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
    run("cargo", "run", "--locked", "--quiet", "-p", "pivot-protocol", "--example", "authoring_parity", "--", "write", str(output))
    run("dotnet", "build", "tests/protocol-parity/legacy/LegacyProtocol.csproj", "-c", "Release", "--disable-build-servers", "-warnaserror")
    run("dotnet", "tests/protocol-parity/legacy/bin/Release/net10.0/LegacyProtocol.dll", str(output))
    run("cargo", "run", "--locked", "--quiet", "-p", "pivot-host", "--example", "legacy_probe", "--", str(output))
    run("dotnet", "tests/protocol-parity/legacy/bin/Release/net10.0/LegacyProtocol.dll", str(output))
    run("dotnet", "build", "tests/protocol-parity/ProtocolParity.csproj", "-c", "Release", "--disable-build-servers", "-warnaserror")
    run("dotnet", "tests/protocol-parity/bin/Release/net10.0/ProtocolParity.dll", str(output))
    run("cargo", "run", "--locked", "--quiet", "-p", "pivot-protocol", "--example", "protocol_parity", "--", "read", str(output))
    run("cargo", "run", "--locked", "--quiet", "-p", "pivot-protocol", "--example", "authoring_parity", "--", "read", str(output))
    count = len((output / "authoring-manifest.txt").read_text(encoding="utf-8").splitlines())
    (output / "results.txt").write_text(f"PASS: Rust/C# codecs, 58 decimal layouts, {count} authoring fixtures, frozen v1.0 codec/current host and descriptors\n", encoding="utf-8")


if __name__ == "__main__":
    main()
