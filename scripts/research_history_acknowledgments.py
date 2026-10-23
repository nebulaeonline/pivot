"""Stage 3 successful client acknowledgment followed by independent process reopen."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

platform, binary, native, receipts = sys.argv[1:]
binary = str(Path(binary).resolve())
native, receipts = Path(native).resolve(), Path(receipts).resolve()
native.mkdir(parents=True, exist_ok=False)
receipts.mkdir(parents=True, exist_ok=False)
summary = []


def run(args, name):
    r = subprocess.run([binary, *map(str, args)], capture_output=True, text=True, timeout=60)
    (receipts / name).write_text(r.stdout + r.stderr, encoding="utf-8")
    assert r.returncode == 0 and "FINISH 0" in r.stdout, (name, r.stdout, r.stderr)
    return r.stdout


for mode in ("full", "delta16", "delta64", "adaptive64"):
    for operation in ("edit", "undo", "redo"):
        name = f"{mode}-{operation}"
        path = native / (name + ".journal")
        op = str(uuid.uuid4())
        run(["seed", path, mode, "b" if operation == "undo" else "a"], name + "-seed.log")
        # Non-binding 1 MiB write allowance: no injected failure, ordinary synced success.
        ack = run(["attempt", path, op, operation, "1048576", "budget"], name + "-ack.log")
        assert f"ACK {op} " in ack and "RECONCILED" not in ack
        recovery = run(["probe", path, op, "committed"], name + "-reopen.log")
        summary.append({"case": name, "transaction": op, "acknowledgment": ack, "recovery": recovery})
        print(f"PASS {platform} acknowledged {name}", flush=True)
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
print(f"{platform}: {len(summary)} acknowledged scenarios, {len(manifest)} original hashes", flush=True)
