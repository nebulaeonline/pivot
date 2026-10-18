"""Build both implementations and compare exact decimal values and scales.

Run from repository root: python tests/decimal-parity/verify.py
Only the Python standard library is required. No float conversions.
"""
import decimal
import pathlib
import random
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]


def run(*args):
    return subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=True).stdout


def vectors():
    rows = []
    expected = {}

    def add(op, a, b="-", mode="-", want=None):
        key = f"v{len(rows):05}"
        rows.append("\t".join((key, op, a, b, mode)))
        if want is not None:
            expected[key] = want

    modes = {
        "ToEven": decimal.ROUND_HALF_EVEN,
        "MidpointAwayFromZero": decimal.ROUND_HALF_UP,
        "MidpointTowardZero": decimal.ROUND_HALF_DOWN,
        "ToZero": decimal.ROUND_DOWN,
        "AwayFromZero": decimal.ROUND_UP,
        "ToNegativeInfinity": decimal.ROUND_FLOOR,
        "ToPositiveInfinity": decimal.ROUND_CEILING,
    }
    decimal.getcontext().prec = 100
    samples = ["0", "-0.00", "1.2300", "2.5", "3.5", "2.499999", "2.500001",
               "1.245", "1.255", "0.0000000000000000000000000001",
               "0.0000000000000000000000000015",
               "79228162514264337593543950335",
               "7922816251426433759354395033.5"]
    for a in samples + ["-" + s for s in samples if not s.startswith("-")]:
        for places in [0, 1, 2, 27, 28]:
            quantum = decimal.Decimal(1).scaleb(-places)
            for mode, strategy in modes.items():
                want = decimal.Decimal(a).quantize(quantum, rounding=strategy)
                add("round", a, str(places), mode, str(want))
            add("default", a, str(places), want=str(decimal.Decimal(a).quantize(
                quantum, rounding=decimal.ROUND_HALF_UP)))
        add("round", a, "29", "ToEven", "InvalidScale")
        add("normalize", a, want=str(decimal.Decimal(a)))
        add("identity", a, want=str(decimal.Decimal(a)))

    values = ["0", "-0.00", "1", "-1", "2", "3", "7", "0.1", "1.00", "1.2345678901234567890123456789",
              "0.0000000000000000000000000001", "0.0000000000000000000000000005",
              "79228162514264337593543950335", "-79228162514264337593543950335",
              "7922816251426433759354395033.5", "0.9999999999999999999999999999"]
    for a in values:
        for b in values:
            for op in ["add", "sub", "mul", "div"]:
                add(op, a, b)
    rng = random.Random(20261018)
    for _ in range(2000):
        pair = []
        for _ in range(2):
            coefficient = rng.randrange(-(2**96 - 1), 2**96)
            scale = rng.randrange(29)
            pair.append(format(decimal.Decimal(coefficient).scaleb(-scale), "f"))
        for op in ["add", "sub", "mul", "div"]:
            add(op, *pair)
    return rows, expected


def parse(output):
    return {f[0]: f[1:] for line in output.splitlines() if (f := line.split("\t"))[0].startswith("v")}


def main():
    run("cargo", "build", "-p", "pivot-types", "--example", "decimal_parity")
    run("dotnet", "build", "tests/decimal-parity/DecimalParity.csproj", "--disable-build-servers", "-v:q")
    rows, expected = vectors()
    with tempfile.TemporaryDirectory() as tmp:
        path = pathlib.Path(tmp) / "vectors.tsv"
        path.write_text("\n".join(rows), encoding="utf-8")
        rust = parse(run("cargo", "run", "--quiet", "-p", "pivot-types", "--example", "decimal_parity", "--", str(path)))
        csharp = parse(run("dotnet", "tests/decimal-parity/bin/Debug/net10.0/DecimalParity.dll", str(path)))
        raw_rust = parse(run("cargo", "run", "--quiet", "-p", "pivot-types", "--example", "decimal_parity", "--", str(path), "--native"))
        raw_csharp = parse(run("dotnet", "tests/decimal-parity/bin/Debug/net10.0/DecimalParity.dll", str(path), "--native"))
    raw_differences = [f"{row}: Rust={raw_rust[key]}, .NET={raw_csharp[key]}"
                       for row in rows if (key := row.split("\t")[0]) and raw_rust[key] != raw_csharp[key]]
    (ROOT / "tests/decimal-parity/backend-baseline.txt").write_text(
        f"Native backend representation differences across {len(rows)} vectors: {len(raw_differences)}\n"
        + "\n".join(raw_differences) + "\n", encoding="utf-8")
    failures, scales = [], []
    for row in rows:
        key = row.split("\t")[0]
        r, c = rust.get(key), csharp.get(key)
        if not r or not c or r[0] != c[0] or (r[0] == "ok" and decimal.Decimal(r[1]) != decimal.Decimal(c[1])):
            failures.append(f"{row}: Rust={r}, .NET={c}")
        elif r[0] == "ok" and r[2] != c[2]:
            scales.append(f"{row}: Rust={r}, .NET={c}")
        if key in expected:
            want = expected[key]
            for label, result in [("Rust", r), (".NET", c)]:
                if want == "InvalidScale":
                    good = result and result[0] == want
                else:
                    good = result and result[0] == "ok" and decimal.Decimal(result[1]) == decimal.Decimal(want)
                if not good:
                    failures.append(f"{row}: {label}={result}, expected={want}")
    report = [f"Vectors: {len(rows)}", f"Value/error failures: {len(failures)}",
              f"Scale differences: {len(scales)}", *failures, *scales]
    (ROOT / "tests/decimal-parity/results.txt").write_text("\n".join(report) + "\n", encoding="utf-8")
    print("\n".join(report[:40]))
    if failures or scales:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
