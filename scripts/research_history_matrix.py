"""Run only the ADR-0023 Stage 2 local-filesystem research matrix."""
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import time

platform, binary, samples, receipts = sys.argv[1:5]
adaptive = sys.argv[5:] == ["adaptive"]
binary = str(pathlib.Path(binary).resolve())
samples, receipts = pathlib.Path(samples).resolve(), pathlib.Path(receipts).resolve()
samples.mkdir(parents=True, exist_ok=True)
receipts.mkdir(parents=True, exist_ok=True)
runs = [(case, mode, 10000, False, "replace")
        for case in ("small", "medium", "formulas", "near")
        for mode in ("full", "delta1", "delta16", "delta64", "objects")]
runs += [(case, mode, 1000, False, "mixed")
         for case in ("small", "medium", "formulas", "near")
         for mode in ("full", "delta16", "objects")]
runs += [(case, mode, 1000, True, "replace")
         for case in ("small", "near")
         for mode in ("full", "delta16", "objects")]
if adaptive:
    runs = [(case, "bounded64", 10000, False, "replace")
            for case in ("small", "medium", "formulas", "near")]
    runs += [(case, "bounded64", 1000, False, "mixed")
             for case in ("small", "medium", "formulas", "near")]
    runs += [(case, "bounded64", 1000, True, "replace") for case in ("small", "near")]
    runs += [(case, "delta64", 1000, False, "replace") for case in ("small", "near")]
summary = []
for case, mode, count, sync, workload in runs:
    name = f"{case}-{mode}-{count}-{'sync' if sync else workload}"
    output = samples / workload / ("sync" if sync else "buffered")
    output.mkdir(parents=True, exist_ok=True)
    stdout, stderr = receipts / (name + ".jsonl"), receipts / (name + ".stderr.log")
    # Refuse to overwrite earlier failed or passing evidence.
    with stdout.open("x", encoding="utf-8") as out, stderr.open("x", encoding="utf-8") as err:
        start = time.perf_counter()
        result = subprocess.run([binary, case, mode, str(count), str(output),
                                 "sync" if sync else "buffered", workload], stdout=out, stderr=err)
    elapsed = time.perf_counter() - start
    if result.returncode:
        raise SystemExit(f"FAILED {platform} {name}: exit {result.returncode}; {stderr}")
    rows = [json.loads(line) for line in stdout.read_text(encoding="utf-8-sig").splitlines()]
    assert rows[-1]["remaining_charge"] == 0 and rows[-1]["finished_transactions"] == count
    assert rows[-2]["transactions"] == count and rows[-2]["semantic_samples_passed"] >= 3
    summary.append({"run": name, "seconds": elapsed, "last": rows[-2]})
    print(f"PASS {platform} {name} {elapsed:.2f}s", flush=True)
(receipts / "matrix-summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
# Hash original native-filesystem evidence before copying it to the shared workspace.
manifest = []
for source in sorted(samples.rglob("*")):
    if source.is_file():
        relative = source.relative_to(samples)
        manifest.append({"path": relative.as_posix(), "sha256": hashlib.sha256(source.read_bytes()).hexdigest()})
        target = receipts / "samples" / relative
        if source.resolve() != target.resolve():
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, target)
(receipts / "original-sample-hashes.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
print(f"{platform}: {len(summary)} runs, {len(manifest)} original sample hashes", flush=True)
