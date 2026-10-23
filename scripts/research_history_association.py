"""Stage 4 actual native file/history association, process termination and lifecycle research."""
import hashlib
import json
from pathlib import Path
import queue
import shutil
import subprocess
import sys
import threading

platform, binary, native, receipts = sys.argv[1:]
binary = str(Path(binary).resolve())
native, receipts = Path(native).resolve(), Path(receipts).resolve()
native.mkdir(parents=True, exist_ok=False)
receipts.mkdir(parents=True, exist_ok=False)
summary = []


def run(args, name, pause=False):
    command = [binary, *map(str, args)]
    if pause:
        child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        messages = queue.Queue()
        threading.Thread(target=lambda: messages.put(child.stdout.readline()), daemon=True).start()
        try:
            first = messages.get(timeout=30)
        except queue.Empty:
            child.kill()
            raise AssertionError(f"hook timeout: {name}")
        assert first.startswith("STAGE4_READY "), (name, first)
        child.kill()
        out, err = child.communicate(timeout=10)
        out = first + out
        code = child.returncode
        assert code != 0 and "FINISH" not in out
    else:
        child = subprocess.run(command, capture_output=True, text=True, timeout=120)
        out, err, code = child.stdout, child.stderr, child.returncode
    (receipts / name).write_text(out + err, encoding="utf-8")
    assert pause or (code == 0 and "FINISH 0" in out), (name, code, out, err)
    return {"command": command, "returncode": code, "killed": pause, "stdout": out, "stderr": err}


def hashes(root):
    return {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(root.rglob("*")) if p.is_file()}


for case in ("matching", "late-save", "late-clear", "cancel", "retained", "session-only",
             "rename", "copy", "replace", "save-as", "relocate", "unavailable", "unretained", "historical-export-limit"):
    result = run(["exercise", native / case, case], case + ".log")
    summary.append({"case": case, "result": result})
    print(f"PASS {platform} {case}", flush=True)

control = ["BeforeControl", "PartialControl", "BeforeControlDecision", "PartialControlDecision", "AfterControlDecision"]
for operation, points in [("save", control + ["AfterInstall", "BeforeSaveReceipt"]),
                          ("clear", control), ("close", control),
                          ("native", ["BeforeWrite", "PartialWrite", "BeforeInstall", "AfterInstall", "BeforeVerify"]),
                          ("move", ["BeforeMove", "AfterMove"])]:
    for point in points:
        for action in ("pause", "error"):
            name = f"{operation}-{point}-{action}"
            base = native / name
            run(["seed", base, "prepared" if operation in ("save", "native", "clear") else "normal"], name + "-seed.log")
            attempt = run(["operate", base, operation, point, action], name + "-attempt.log", action == "pause")
            probe = run(["probe", base, operation, point], name + "-probe.log")
            summary.append({"case": name, "attempt": attempt, "probe": probe})
            print(f"PASS {platform} {name}", flush=True)

# Ordinary saved Open and explicit decline never open or mutate history.
base = native / "decline-restart"
run(["seed", base], "decline-seed.log")
before = hashes(base / "history")
for i in range(2):
    run(["probe", base, "decline", "none"], f"decline-{i}.log")
    assert before == hashes(base / "history")
recovered = run(["probe", base, "recover", "none"], "decline-later-recover.log")
assert before == hashes(base / "history")
summary.append({"case": "decline-restart", "probe": recovered, "unchanged_history_hashes": before})
print(f"PASS {platform} decline-restart", flush=True)

for damage in ("missing", "control-version", "control-decision", "journal", "copied-store"):
    base = native / damage
    run(["seed", base], damage + "-seed.log")
    history = base / "history"
    if damage == "missing":
        history.rename(base / "preserved-history")
    elif damage == "copied-store":
        shutil.copytree(history, base / "copy")
        history.rename(base / "preserved-history")
        (base / "copy").rename(history)
    else:
        path = next(history.glob("epoch-*.journal")) if damage == "journal" else history / "control"
        data = bytearray(path.read_bytes())
        at = 384 if damage == "journal" else (0 if damage == "control-version" else len(data) - 1)
        data[at] ^= 255
        path.write_bytes(data)
    before = hashes(history) if history.exists() else {}
    probe = run(["probe", base, "unavailable", "none"], damage + "-probe.log")
    assert before == (hashes(history) if history.exists() else {})
    summary.append({"case": damage, "probe": probe})
    print(f"PASS {platform} ordinary-open-{damage}", flush=True)

# Two distinct processes: held owner does not prevent ordinary WB5 Open, but excludes attachment.
base = native / "two-processes"
run(["seed", base], "two-processes-seed.log")
child = subprocess.Popen([binary, "operate", str(base), "hold", "none", "pause"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
try:
    assert child.stdout.readline().startswith("STAGE4_READY locked")
    result = run(["probe", base, "unavailable", "none"], "two-processes-contender.log")
finally:
    child.kill()
    child.communicate(timeout=10)
assert child.returncode != 0
summary.append({"case": "two-processes", "holder_returncode": child.returncode, "probe": result})
print(f"PASS {platform} two-processes", flush=True)

(receipts / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
manifest = []
for source in sorted(native.rglob("*")):
    if source.is_file():
        relative = source.relative_to(native)
        manifest.append({"path": relative.as_posix(), "sha256": hashlib.sha256(source.read_bytes()).hexdigest()})
        target = receipts / "native-evidence" / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
(receipts / "native-original-hashes.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
print(f"{platform}: {len(summary)} cases, {len(manifest)} native-original hashes", flush=True)
