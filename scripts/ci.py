"""Local CI: python scripts/ci.py (Python 3.8+, no third-party packages)."""
import argparse
import datetime
import json
import pathlib
import shutil
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]


def run_step(name, command, output):
    """Run one gate, recording its complete output and actual exit status."""
    log = output / f"{name}.log"
    started = time.monotonic()
    print(f"[{name}] {' '.join(command)}", flush=True)
    try:
        # No shell interpolation; arguments and paths remain separate.
        with log.open("w", encoding="utf-8") as stream:
            process = subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE,
                                       stderr=subprocess.STDOUT, text=True,
                                       encoding="utf-8", errors="replace")
            try:
                for line in process.stdout:
                    stream.write(line)
                    stream.flush()
                    print(line, end="", flush=True)
                code = process.wait()
            except BaseException:
                process.terminate()
                process.wait()
                raise
    except OSError as error:
        log.write_text(str(error) + "\n", encoding="utf-8")
        print(error, flush=True)
        code = 127
    return {"name": name, "command": command, "exit_code": code,
            "status": "passed" if code == 0 else "failed",
            "seconds": round(time.monotonic() - started, 3), "log": str(log)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=pathlib.Path, default=ROOT / "artifacts/ci")
    args = parser.parse_args()
    output = args.output_dir.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output.mkdir(parents=True)
    print(f"CI reports: {output}", flush=True)
    missing = [tool for tool in ["cargo", "rustc", "dotnet"] if not shutil.which(tool)]
    steps = []
    if missing:
        print(f"Missing required tools: {', '.join(missing)}")
    else:
        commands = [
            ("rust-version", ["rustc", "--version"]),
            ("cargo-version", ["cargo", "--version"]),
            ("dotnet-version", ["dotnet", "--version"]),
            ("rust-format", ["cargo", "fmt", "--all", "--", "--check"]),
            ("rust-lint", ["cargo", "clippy", "--locked", "--workspace", "--all-targets", "--", "-D", "warnings"]),
            ("rust-tests", ["cargo", "test", "--locked", "--workspace"]),
            ("dotnet-build", ["dotnet", "build", "Pivot.slnx", "-c", "Release", "--disable-build-servers", "-warnaserror"]),
            ("decimal-parity", [sys.executable, "tests/decimal-parity/verify.py", "--configuration", "Release", "--output-dir", str(output / "decimal-parity")]),
            ("protocol-parity", [sys.executable, "tests/protocol-parity/verify.py", "--output-dir", str(output / "protocol-parity")]),
            ("local-ipc", [sys.executable, "tests/ipc/verify.py", "--output-dir", str(output / "ipc")]),
        ]
        for name, command in commands:
            steps.append(run_step(name, command, output))
            if steps[-1]["exit_code"]:
                break
    passed = not missing and all(step["exit_code"] == 0 for step in steps)
    summary = {"status": "passed" if passed else "failed", "python": sys.version,
               "platform": sys.platform, "missing_tools": missing, "steps": steps}
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"CI {summary['status'].upper()}: {output / 'summary.json'}", flush=True)
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
