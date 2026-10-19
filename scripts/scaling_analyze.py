"""Normalize one scaling run and write a reviewable artifact summary."""
import argparse
import csv
import json
import pathlib


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run", type=pathlib.Path)
    args = parser.parse_args()
    rows = []
    phases = []
    for mode in ["latency", "profile"]:
        with (args.run / (mode + ".csv")).open(encoding="utf-8") as stream:
            header = None
            for record in csv.reader(stream):
                if not record:
                    continue
                if record[0] == "HEADER":
                    header = record[1:]
                elif record[0] == "ROW":
                    assert header and len(record[1:]) == len(header)
                    row = dict(zip(header, record[1:]))
                    for key in header:
                        if key not in ["implementation", "shape", "operation", "mode"]:
                            row[key] = int(row[key])
                    batch = 1000 if row["operation"].endswith("_1000") else 1
                    row["capture_batch"] = batch
                    if batch != 1:
                        row["operation"] = row["operation"][:-5]
                        for key in ["p50_ns", "p95_ns", "worst_ns", "allocated_bytes_mean", "allocation_calls_mean", "instrumented_payload_copy_bytes_mean", "retained_delta_mean_bytes"]:
                            row[key] /= batch
                    if mode == "latency":
                        for key in header[9:]:
                            row[key] = None
                    rows.append(row)
                elif record[0] == "PHASE":
                    phases.append(dict(zip(["implementation", "shape", "cells", "operation", "phase", "mean_ns", "allocated_bytes_mean"], record[1:])))
    (args.run / "metrics.json").write_text(json.dumps({"rows": rows, "phases": phases}, indent=2)+"\n", encoding="utf-8")
    lookup = {(r["implementation"], r["shape"], r["cells"], r["operation"]): r for r in rows if r["mode"] == "latency"}
    text = ["# Optimized scaling observations", "", "All times below are observed p95 milliseconds, not release guarantees.", "", "| Shape | Cells | Operation | Baseline | Shared | Baseline/shared |", "| --- | --- | --- | --- | --- | --- |"]
    for shape in ["independent", "chain", "fanout", "mixed"]:
        for size in [1000, 10000, 100000]:
            for op in ["isolated_input_edit", "capture", "selected_evaluate", "update_capture_evaluate", "viewport_100x50", "formula_replace", "affected_capture_evaluate"]:
                a = lookup.get(("baseline", shape, size, op))
                b = lookup.get(("shared", shape, size, op))
                if a and b:
                    ratio = a["p95_ns"] / b["p95_ns"] if b["p95_ns"] else 0
                    text.append(f"| {shape} | {size} | {op} | {a['p95_ns']/1e6:.6f} | {b['p95_ns']/1e6:.6f} | {ratio:.2f} |")
    text += ["", "Capture sharing is normalized from batches of 1,000. Structural formula replacement eagerly validates the shared topology but baseline validation occurs at capture, so that row is not a matching validation-cost speed comparison.", "", "Profile-mode latency is excluded from this table. Zero memory counters in raw latency logs mean unmeasured, not zero allocation. See metrics.json and profile.csv for instrumented memory/phase data.", ""]
    (args.run / "analysis.md").write_text("\n".join(text), encoding="utf-8")
    print(args.run / "analysis.md")


if __name__ == "__main__":
    main()
