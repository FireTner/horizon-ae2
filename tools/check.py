#!/usr/bin/env python3
"""Standalone checks, including injected Rust sources Cargo's library does not see."""
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    subprocess.run(args, cwd=ROOT, check=True)


if __name__ == "__main__":
    run("cargo", "fmt", "--all", "--check")
    run("rustfmt", "--edition", "2024", "--check",
        *map(str, sorted((ROOT / "mixins/ae2").glob("*.rs"))))
    run("cargo", "clippy", "--all-targets", "--locked", "--", "-D", "warnings")
    run("cargo", "test", "--locked")
    run(sys.executable, "tools/test_tooling.py")
    run(sys.executable, "tools/upstream.py", "audit", "--check")
