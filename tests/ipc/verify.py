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
import time
import uuid

ROOT = pathlib.Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=pathlib.Path, default=ROOT / "artifacts/ipc")
    args = parser.parse_args()
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    subprocess.run(["cargo", "build", "--locked", "-p", "pivotd", "-p", "pivot-cli", "-p", "pivot-transport", "--bins", "--example", "ipc_probe", "--example", "repl_peer"], cwd=ROOT, check=True, timeout=180)
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
        # Persistent command mode, including idle time longer than the old five seconds.
        repl = subprocess.Popen([str(binary / ("pivot-cli" + extension)), "--endpoint", endpoint, "repl"], cwd=ROOT, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        repl_lines = []
        described = threading.Event()
        def drain_repl():
            for line in repl.stdout:
                repl_lines.append(line)
                if "output Margin Decimal" in line:
                    described.set()
        repl_reader = threading.Thread(target=drain_repl, daemon=True)
        repl_reader.start()
        try:
            repl.stdin.write("help\nmodels\ndescribe RestaurantProfit\n")
            repl.stdin.flush()
            assert described.wait(timeout=12), "REPL did not complete initial commands"
            time.sleep(6)
            repl.stdin.write("invalid-command\ninvoke RestaurantProfit Revenue=250000\ninvoke RestaurantProfit Revenue=0\ninvoke RestaurantProfit Revenue=100 Costs=25 --revision demo-v1 --outputs Net\nreconnect\nmodels\nexit\n")
            repl.stdin.close()
            repl.wait(timeout=15)
            repl_reader.join(timeout=3)
            transcript = "".join(repl_lines)
            assert repl.returncode == 0, transcript
            assert "unknown or invalid command" in transcript
            assert "Net=165000" in transcript and "Margin=0.66" in transcript
            assert "Net=75" in transcript and "ERROR DivisionByZero" in transcript
            assert "Reconnected." in transcript and "Goodbye." in transcript
            assert "connection lost" not in transcript.lower(), transcript
            (output / "repl.log").write_text(transcript, encoding="utf-8")
        finally:
            if repl.poll() is None:
                repl.kill()
                repl.wait(timeout=5)
            repl_reader.join(timeout=3)
        checks.append("persistent REPL, extended idle, help/history-independent pipe mode, invalid command recovery, explicit reconnect")
        # A peer receives Invoke then closes without a response. The client must
        # not retry; it can explicitly reconnect to the real daemon afterward.
        fake_endpoint = endpoint + "-ambiguous" if os.name == "nt" else str(private / "ambiguous.sock")
        audit = output / "ambiguous-invocation.txt"
        peer = subprocess.Popen([str(binary / "examples" / ("repl_peer" + extension)), fake_endpoint, str(audit)], cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        peer_ready = queue.Queue()
        peer_reader = threading.Thread(target=lambda: peer_ready.put(peer.stdout.readline()), daemon=True)
        peer_reader.start()
        try:
            assert peer_ready.get(timeout=8).startswith("READY")
            script = f"invoke RestaurantProfit Revenue=100\nmodels\nreconnect --endpoint {endpoint}\ninvoke RestaurantProfit Revenue=100 Costs=25 --outputs Net\nexit\n"
            result = subprocess.run([str(binary / ("pivot-cli" + extension)), "--endpoint", fake_endpoint, "repl"], input=script, cwd=ROOT, text=True, capture_output=True, timeout=15)
            transcript = result.stdout + result.stderr
            assert result.returncode == 0, transcript
            assert "outcome may be unknown" in transcript and "No command was sent" in transcript
            assert "Net=75" in transcript and "Reconnected." in transcript
            peer.wait(timeout=6)
            assert peer.returncode == 0 and audit.read_text() == "invocations=1\n"
            (output / "repl-disconnect.log").write_text(transcript, encoding="utf-8")
        finally:
            if peer.poll() is None:
                peer.kill()
                peer.wait(timeout=5)
            peer_reader.join(timeout=2)
        checks.append("ambiguous invocation sent once, disconnected command not sent, explicit reconnect to healthy daemon")
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
