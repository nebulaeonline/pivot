"""Real local daemon/client integration with bounded process lifetimes."""
import argparse
import concurrent.futures
import json
import os
import pathlib
import queue
import subprocess
import sys
import threading
import uuid

ROOT = pathlib.Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=pathlib.Path, default=ROOT / "artifacts/ipc")
    args = parser.parse_args()
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    subprocess.run(["cargo", "build", "--locked", "-p", "pivotd", "-p", "pivot-cli", "-p", "pivot-transport", "--bins", "--example", "ipc_probe"], cwd=ROOT, check=True, timeout=180)
    # Cargo supplies the real target directory even if overridden in configuration.
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT, text=True))
    binary = pathlib.Path(metadata["target_directory"]) / "debug"
    extension = ".exe" if os.name == "nt" else ""
    name = "pivot-test-" + uuid.uuid4().hex
    private = pathlib.Path("/tmp") / name
    endpoint = "\\\\.\\pipe\\" + name if os.name == "nt" else str(private / "pivot.sock")
    ready = queue.Queue()
    daemon_log = (output / "daemon.log").open("w", encoding="utf-8")
    daemon = subprocess.Popen([str(binary / ("pivotd" + extension)), "--demo", "--endpoint", endpoint, "--shutdown-stdin"], cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    def drain():
        for line in daemon.stdout:
            daemon_log.write(line)
            daemon_log.flush()
            if line.startswith("READY "):
                ready.put(line)
    reader = threading.Thread(target=drain, daemon=True)
    reader.start()
    checks = []
    def cli(*arguments, expected=0):
        result = subprocess.run([str(binary / ("pivot-cli" + extension)), "--endpoint", endpoint, *arguments], cwd=ROOT, text=True, capture_output=True, timeout=12)
        assert result.returncode == expected, (arguments, result.stdout, result.stderr)
        return result.stdout
    try:
        ready.get(timeout=12)
        assert "RestaurantProfit" in cli("models")
        assert "optional default=85000" in cli("describe", "RestaurantProfit")
        full = cli("invoke", "RestaurantProfit", "Revenue=250000", "Costs=85000")
        assert "Net=165000" in full and "Margin=0.66" in full
        defaults = cli("invoke", "RestaurantProfit", "Revenue=250000")
        assert "Net=165000" in defaults and "revision=demo-v1" in defaults
        selected = cli("invoke", "RestaurantProfit", "Revenue=100000.00", "Costs=25000.00", "--outputs", "Margin", "--revision", "demo-v1")
        assert "Margin=0.75" in selected and "Net=" not in selected
        cli("invoke", "RestaurantProfit", expected=1)
        cli("invoke", "RestaurantProfit", "Revenue=1", "Revenue=2", expected=1)
        cli("invoke", "RestaurantProfit", "Unknown=1", expected=1)
        cli("invoke", "RestaurantProfit", "Revenue=1", "--revision", "not-present", expected=1)
        assert "ERROR DivisionByZero" in cli("invoke", "RestaurantProfit", "Revenue=0", expected=2)
        checks.append("discovery, descriptions, default/explicit/required inputs, decimals, outputs, revisions, errors")
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            results = list(pool.map(lambda amount: cli("invoke", "RestaurantProfit", f"Revenue={amount}", "Costs=25"), [100, 200, 300, 400]))
        for amount, result in zip([100, 200, 300, 400], results):
            assert f"Net={amount - 25}" in result
        checks.append("concurrent independent CLI processes")
        probe = subprocess.run([str(binary / "examples" / ("ipc_probe" + extension)), endpoint], cwd=ROOT, text=True, capture_output=True, timeout=20)
        assert probe.returncode == 0, (probe.stdout, probe.stderr)
        checks.append(probe.stdout.strip())
        assert "RestaurantProfit" in cli("models")
        assert daemon.poll() is None
    finally:
        try:
            if daemon.poll() is None:
                daemon.stdin.write("\n")
                daemon.stdin.flush()
                daemon.wait(timeout=8)
        finally:
            if daemon.poll() is None:
                daemon.kill()
                daemon.wait(timeout=5)
            reader.join(timeout=3)
            daemon_log.close()
            if os.name != "nt" and private.exists():
                # Remove only our unique empty directory; never recursively delete.
                private.rmdir()
    assert daemon.returncode == 0, "daemon did not shut down gracefully"
    if os.name != "nt":
        assert not pathlib.Path(endpoint).exists()
    checks.append("graceful daemon shutdown and endpoint cleanup")
    (output / "results.txt").write_text("PASS\n" + "\n".join(checks) + "\n", encoding="utf-8")
    print("IPC integration: PASS")


if __name__ == "__main__":
    main()
