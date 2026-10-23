"""Stage 3 real-child termination and on-disk fault/recovery research; no UI tests."""
import hashlib
import json
from pathlib import Path
import queue
import shutil
import subprocess
import sys
import threading
import time
import uuid

platform, binary, native, receipts = sys.argv[1:]
binary = str(Path(binary).resolve())
native, receipts = Path(native).resolve(), Path(receipts).resolve()
native.mkdir(parents=True, exist_ok=False)
receipts.mkdir(parents=True, exist_ok=False)
points = ["BeforePrepare", "PartialPrepare", "AfterPrepare", "PartialPayload",
          "AfterPayload", "BeforePayloadSync", "AfterPayloadSync", "PartialDecision",
          "AfterDecisionWrite", "BeforeDecisionSync", "AfterDecisionSync", "BeforeAcknowledge"]
new_points = set(points[8:])
summary = []


def run(args, log, pause=False):
    command = [binary, *map(str, args)]
    start = time.perf_counter()
    if pause:
        child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        messages = queue.Queue()
        thread = threading.Thread(target=lambda: messages.put(child.stdout.readline()), daemon=True)
        thread.start()
        try:
            first = messages.get(timeout=30)
        except queue.Empty:
            child.kill()
            out, err = child.communicate(timeout=10)
            (receipts / log).write_text(out + err, encoding="utf-8")
            raise AssertionError(f"fault worker timeout: {log}")
        assert first.startswith("STAGE3_READY "), (log, first)
        child.kill()  # SIGKILL / TerminateProcess; no Rust cleanup or in-memory recovery.
        out, err = child.communicate(timeout=10)
        out = first + out
        code = child.returncode
        assert code != 0
        assert "ACK " not in out and "FINISH " not in out
    else:
        child = subprocess.run(command, capture_output=True, text=True, timeout=180)
        out, err, code = child.stdout, child.stderr, child.returncode
    (receipts / log).write_text(out + err, encoding="utf-8")
    assert pause or (code == 0 and "FINISH 0" in out), (log, code, out, err)
    return {"command": command, "seconds": time.perf_counter() - start, "returncode": code,
            "terminated_by_parent": pause, "stdout": out, "stderr": err}


def scenario(mode, operation, point, action):
    name = f"{mode}-{operation}-{point}-{action}"
    path = native / (name + ".journal")
    op = str(uuid.uuid4())
    run(["seed", path, mode, "b" if operation == "undo" else "a"], name + "-seed.log")
    attempt = run(["attempt", path, op, operation, point, action], name + "-attempt.log", action == "pause")
    expect = "committed" if action != "budget" and point in new_points else "uncommitted"
    probe = run(["probe", path, op, expect], name + "-probe.log")
    if action == "error":
        assert "ACK " not in attempt["stdout"]
        assert ("RECONCILED " + expect in attempt["stdout"] or
                point == "BeforePrepare" and "DEFINITELY_UNCOMMITTED" in attempt["stdout"])
    summary.append({"scenario": name, "transaction": op, "expected": expect,
                    "attempt": attempt, "probe": probe})
    print(f"PASS {platform} {name} -> {expect}", flush=True)


for operation in ("edit", "undo", "redo"):
    for point in points:
        for action in ("pause", "error"):
            scenario("delta16", operation, point, action)
    for budget in ("0", "128", "256", "257"):
        scenario("delta16", operation, budget, "budget")
for mode in ("full", "delta64", "adaptive64"):
    for operation in ("edit", "undo", "redo"):
        for point in ("BeforePayloadSync", "PartialDecision", "AfterDecisionSync"):
            for action in ("pause", "error"):
                scenario(mode, operation, point, action)

# Intact decisions must not make missing/damaged referenced bytes authoritative.
for mode in ("full", "delta16", "delta64", "adaptive64"):
    for damage in ("payload", "missing-payload", "decision", "metadata", "wrong-ledger", "wrong-parent", "version"):
        name = f"{mode}-corrupt-{damage}"
        path = native / (name + ".journal")
        op = str(uuid.uuid4())
        run(["seed", path, mode, "a"], name + "-seed.log")
        run(["attempt", path, op, "edit", "AfterDecisionSync", "pause"], name + "-attempt.log", True)
        data = bytearray(path.read_bytes())
        # Walk framing solely to select bytes to damage; it is not the recovery oracle.
        offset, last = 128, 0
        while offset < len(data):
            last = offset
            length = int.from_bytes(data[offset + 80:offset + 88], "little")
            offset += 256 + length + 64
        if damage == "payload":
            data[last + 256] ^= 255
        elif damage == "missing-payload":
            data[last + 256:last + 256 + length] = bytes(length)  # Keep decision readable in place.
        elif damage == "decision":
            data[-1] ^= 255
        elif damage == "metadata":
            data[last + 160] ^= 255
        elif damage in ("wrong-ledger", "wrong-parent"):
            field = 160 if damage == "wrong-ledger" else 136
            data[last + field:last + field + 8] = (last if damage == "wrong-parent" else 128).to_bytes(8, "little")
            digest = hashlib.sha256(data[last:last + 224]).digest()
            data[last + 224:last + 256] = digest
            data[-48:-16] = digest  # Decision remains checksum-valid; transition must reject.
        else:
            data[0] ^= 255
        path.write_bytes(data)
        before = hashlib.sha256(data).hexdigest()
        probe = run(["probe", path, op, "reject"], name + "-probe.log")
        assert hashlib.sha256(path.read_bytes()).hexdigest() == before  # No repair/truncation of corruption.
        summary.append({"scenario": name, "expected": "quarantine", "probe": probe})
        print(f"PASS {platform} {name} -> quarantine", flush=True)

for mode, cells in [("full", 16), ("delta16", 16), ("delta64", 16), ("adaptive64", 16), ("delta16", 1024)]:
    name = f"series-{mode}-{cells}"
    result = run(["series", native / (name + ".journal"), mode, 1000, cells], name + ".log")
    summary.append({"scenario": name, "result": result})
    print(f"PASS {platform} {name}", flush=True)

(receipts / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
# Hash original native evidence before copying; preserve journals and exact independent oracles.
manifest = []
for source in sorted(native.rglob("*")):
    if source.is_file():
        relative = source.relative_to(native)
        manifest.append({"path": relative.as_posix(), "sha256": hashlib.sha256(source.read_bytes()).hexdigest()})
        target = receipts / "native-evidence" / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
(receipts / "native-original-hashes.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
print(f"{platform}: {len(summary)} scenarios; {len(manifest)} original hashes", flush=True)
