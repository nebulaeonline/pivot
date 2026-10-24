"""Local CI: python scripts/ci.py (Python 3.8+, no third-party packages)."""
import argparse
from contextlib import nullcontext
from execution_policy import Blocked, ExecutionClass, blocked_exit, desktop_environment, desktop_session, headless_environment
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
            environment = desktop_environment() if name == "desktop-ui" else headless_environment()
            process = subprocess.Popen(command, cwd=ROOT, env=environment, stdout=subprocess.PIPE,
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
    parser.add_argument("--desktop", action="store_true", help="Include actual-window/clipboard/close checks; requires an interactive desktop.")
    args = parser.parse_args()
    try:
        with desktop_session("CI including desktop acceptance") if args.desktop else nullcontext():
            return execute(args)
    except Blocked as error:
        return blocked_exit(error)


def execute(args):
    output = args.output_dir.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output.mkdir(parents=True)
    print(f"CI reports: {output}", flush=True)
    print("Execution classes: headless, fixture-contained-fault" + (", gui-isolated-or-interactive" if args.desktop else ""), flush=True)
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
            ("execution-safety", [sys.executable, "tests/execution-safety/verify.py"]),
            ("aggregate-reference", [sys.executable, "tests/decimal-parity/aggregate_reference.py", "--check"]),
            ("decimal-parity", [sys.executable, "tests/decimal-parity/verify.py", "--configuration", "Release", "--output-dir", str(output / "decimal-parity")]),
            ("protocol-parity", [sys.executable, "tests/protocol-parity/verify.py", "--output-dir", str(output / "protocol-parity")]),
            ("local-ipc", [sys.executable, "tests/ipc/verify.py", "--output-dir", str(output / "ipc")]),
            ("application-session", [sys.executable, "tests/application-session/verify.py", "--output-dir", str(output / "application-session")]),
        ]
        if args.desktop:
            commands.append(("desktop-lock-tests", [sys.executable, "tests/application-session/test_desktop_lock.py"]))
            commands.append(("desktop-ui", [sys.executable, "tests/application-session/ui_smoke.py", "--output-dir", str(output / "desktop-ui")]))
        for name, command in commands:
            steps.append(run_step(name, command, output))
            if steps[-1]["exit_code"]:
                break
    passed = not missing and all(step["exit_code"] == 0 for step in steps)
    summary = {"status": "passed" if passed else "failed", "python": sys.version,
               "platform": sys.platform, "desktop": args.desktop, "execution_classes": [ExecutionClass.HEADLESS.value, ExecutionClass.FIXTURE_FAULT.value] + ([ExecutionClass.GUI.value] if args.desktop else []), "missing_tools": missing, "steps": steps}
    (output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"CI {summary['status'].upper()}: {output / 'summary.json'}", flush=True)
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
