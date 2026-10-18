"""Generated Rust/C# Protobuf interoperability without a transport or daemon."""
import argparse
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]


def run(*command):
    subprocess.run(command, cwd=ROOT, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=pathlib.Path, default=ROOT / "artifacts/protocol-parity")
    args = parser.parse_args()
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    run("cargo", "run", "--locked", "--quiet", "-p", "pivot-protocol", "--example", "protocol_parity", "--", "write", str(output))
    run("dotnet", "build", "tests/protocol-parity/ProtocolParity.csproj", "-c", "Release", "--disable-build-servers", "-warnaserror")
    run("dotnet", "tests/protocol-parity/bin/Release/net10.0/ProtocolParity.dll", str(output))
    run("cargo", "run", "--locked", "--quiet", "-p", "pivot-protocol", "--example", "protocol_parity", "--", "read", str(output))
    (output / "results.txt").write_text("PASS: generated Rust/C# protobuf fixtures and 58 decimal layouts\n", encoding="utf-8")


if __name__ == "__main__":
    main()
