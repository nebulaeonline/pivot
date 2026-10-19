"""Resolve perf report DSO-relative program offsets with local binutils."""
import argparse
import json
import pathlib
import re
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=pathlib.Path)
    parser.add_argument("binary", type=pathlib.Path)
    parser.add_argument("output", type=pathlib.Path)
    args = parser.parse_args()
    samples = []
    for line in args.report.read_text().splitlines():
        match = re.match(r"\s*([0-9.]+)%\s+scaling\s+\[.\]\s+(0x[0-9a-f]+)", line)
        if match:
            samples.append({"percent": float(match[1]), "offset": match[2]})
    addresses = [row["offset"] for row in samples]
    result = subprocess.run(["addr2line", "-a", "-f", "-C", "-i", "-e", str(args.binary), *addresses], check=True, capture_output=True, text=True)
    decoded = {}
    current = None
    for line in result.stdout.splitlines():
        if re.fullmatch(r"0x[0-9a-f]+", line):
            current = int(line, 16)
            decoded[current] = []
        elif current is not None:
            decoded[current].append(line)
    for row in samples:
        lines = decoded[int(row["offset"], 16)]
        row["inline_frames"] = [{"function": lines[i], "location": lines[i+1]} for i in range(0, len(lines)-1, 2)]
    args.output.with_suffix(".json").write_text(json.dumps(samples, indent=2)+"\n", encoding="utf-8")
    text = ["# Offline sampled-address resolution", "", "Individual user-CPU IP samples, not reconstructed non-inline call stacks. Percentages describe the complete mixed baseline/prototype profiling workload, not individual operations. Optimized/folded generic inline frames require cautious interpretation.", ""]
    for row in samples[:25]:
        text.append(f"## {row['percent']:.2f}% at {row['offset']}")
        text.append("")
        for frame in row["inline_frames"]:
            text.append(f"- `{frame['function']}` — `{frame['location']}`")
        text.append("")
    args.output.with_suffix(".md").write_text("\n".join(text), encoding="utf-8")
    print(f"Resolved {len(samples)} sampled program offsets")


if __name__ == "__main__":
    main()
