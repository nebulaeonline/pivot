"""Run the isolated Release scaling experiment; no third-party dependencies."""
import argparse
import ctypes
import datetime
import hashlib
import json
import os
import pathlib
import platform
import subprocess
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]


def rss(pid):
    """OS process peak resident bytes; sampled on Linux, OS high-water on Windows."""
    try:
        if os.name == "nt":
            from ctypes import wintypes
            class Counters(ctypes.Structure):
                _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [(n, ctypes.c_size_t) for n in ["PeakWorkingSetSize", "WorkingSetSize", "QuotaPeakPagedPoolUsage", "QuotaPagedPoolUsage", "QuotaPeakNonPagedPoolUsage", "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage"]]
            kernel = ctypes.WinDLL("kernel32", use_last_error=True)
            kernel.OpenProcess.restype = wintypes.HANDLE
            kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
            kernel.CloseHandle.argtypes = [wintypes.HANDLE]
            psapi = ctypes.WinDLL("psapi", use_last_error=True)
            psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
            handle = kernel.OpenProcess(0x0410, False, pid)
            if not handle:
                return None
            try:
                info = Counters()
                info.cb = ctypes.sizeof(info)
                if psapi.GetProcessMemoryInfo(handle, ctypes.byref(info), info.cb):
                    return info.PeakWorkingSetSize
            finally:
                kernel.CloseHandle(handle)
        elif platform.system() == "Linux":
            for line in pathlib.Path(f"/proc/{pid}/status").read_text().splitlines():
                if line.startswith("VmHWM:"):
                    return int(line.split()[1]) * 1024
    except (OSError, ValueError):
        pass
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=pathlib.Path, default=ROOT / "artifacts/scaling")
    parser.add_argument("--sizes", default="1000,10000,100000")
    parser.add_argument("--shapes", default="independent,chain,fanout,mixed")
    parser.add_argument("--samples", type=int, default=101)
    parser.add_argument("--modes", default="latency,profile")
    parser.add_argument("--timeout-seconds", type=int, default=1800)
    parser.add_argument("--max-rss-mib", type=int, default=8192)
    args = parser.parse_args()
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    output = args.output_dir.resolve() / stamp
    output.mkdir(parents=True)
    environment = {"platform": platform.platform(), "machine": platform.machine(), "processor": platform.processor(), "logical_cpus": os.cpu_count(), "python": platform.python_version(), "cargo_target_dir": os.environ.get("CARGO_TARGET_DIR"), "sizes": args.sizes, "shapes": args.shapes, "samples": args.samples, "release": "Cargo default release opt-level=3; no custom LTO or target-cpu; instrumentation disabled in latency mode"}
    environment["timeout_seconds_per_mode"] = args.timeout_seconds
    environment["max_rss_mib"] = args.max_rss_mib
    for name, command in [("rustc", ["rustc", "-Vv"]), ("cargo", ["cargo", "--version"])]:
        environment[name] = subprocess.check_output(command, cwd=ROOT, text=True)
    if os.name == "nt":
        import winreg
        with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"HARDWARE\DESCRIPTION\System\CentralProcessor\0") as key:
            environment["cpu_name"] = winreg.QueryValueEx(key, "ProcessorNameString")[0]
        class Memory(ctypes.Structure):
            _fields_ = [("length", ctypes.c_ulong), ("load", ctypes.c_ulong)] + [(name, ctypes.c_ulonglong) for name in ["physical", "available", "pagefile", "available_pagefile", "virtual", "available_virtual", "extended"]]
        memory = Memory()
        memory.length = ctypes.sizeof(memory)
        if ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(memory)):
            environment["physical_memory_bytes"] = memory.physical
            environment["available_memory_bytes"] = memory.available
    if platform.system() == "Linux":
        environment["cpuinfo"] = pathlib.Path("/proc/cpuinfo").read_text().split("\n\n")[0]
        environment["meminfo"] = pathlib.Path("/proc/meminfo").read_text()
    manifest = {}
    for directory in [ROOT / "crates/pivot-core", ROOT / "crates/pivot-workbook"]:
        for path in directory.rglob("*"):
            if path.is_file():
                manifest[str(path.relative_to(ROOT))] = hashlib.sha256(path.read_bytes()).hexdigest()
    for path in [ROOT / "Cargo.lock", ROOT / "Cargo.toml", pathlib.Path(__file__).resolve()]:
        manifest[str(path.relative_to(ROOT))] = hashlib.sha256(path.read_bytes()).hexdigest()
    (output / "source-manifest.json").write_text(json.dumps(manifest, indent=2)+"\n", encoding="utf-8")
    (output / "environment.json").write_text(json.dumps(environment, indent=2)+"\n", encoding="utf-8")
    build = ["cargo", "build", "--release", "--locked", "-p", "pivot-workbook", "--features", "scaling-experiment", "--example", "scaling"]
    with (output / "build.log").open("w", encoding="utf-8") as log:
        subprocess.run(build, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT, text=True))
    binary = pathlib.Path(metadata["target_directory"]) / "release/examples" / ("scaling.exe" if os.name == "nt" else "scaling")
    summary = {"status": "running", "environment": "environment.json", "build": build, "runs": []}
    summary_path = output / "summary.json"
    for mode in args.modes.split(","):
        if mode not in ["latency", "profile"]:
            raise ValueError(mode)
        command = [str(binary), "--sizes", args.sizes, "--shapes", args.shapes, "--samples", str(args.samples if mode == "latency" else 5)]
        if mode == "profile":
            command.append("--profile")
        print(f"{mode}: {output}", flush=True)
        peak = None
        stopped = None
        start = time.monotonic()
        with (output / (mode+".csv")).open("w", encoding="utf-8") as log:
            process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
            while process.poll() is None:
                observed = rss(process.pid)
                if observed is not None:
                    peak = max(peak or 0, observed)
                if (peak or 0) > args.max_rss_mib * 1024 * 1024:
                    stopped = "RSS budget exceeded"
                if time.monotonic() - start > args.timeout_seconds:
                    stopped = "time budget exceeded"
                if stopped:
                    process.kill()
                    break
                time.sleep(0.1)
            code = process.wait()
        summary["runs"].append({"mode": mode, "command": command, "exit_code": code, "seconds": time.monotonic()-start, "os_process_peak_rss_bytes": peak, "budget_stop": stopped, "log": mode+".csv"})
        summary["status"] = "passed" if code == 0 else "failed"
        summary_path.write_text(json.dumps(summary, indent=2)+"\n", encoding="utf-8")
        if code:
            raise SystemExit(code)
    print(f"PASS: {summary_path}", flush=True)


if __name__ == "__main__":
    main()
