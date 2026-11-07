#!/usr/bin/env python3
"""Compile and run crater, fallout, and lake tests without Cargo or the mixin manifest."""

import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "mixins" / "ae2" / "meteorite_fallout.rs"


def main():
    with tempfile.TemporaryDirectory(prefix="meteorite-fallout-") as temporary:
        binary = Path(temporary) / "meteorite_fallout"
        compiled = subprocess.run(
            [
                "/usr/bin/rustc",
                "--edition",
                "2024",
                "--test",
                str(SOURCE),
                "-o",
                str(binary),
            ],
            check=False,
        )
        if compiled.returncode != 0:
            return compiled.returncode
        tested = subprocess.run([str(binary)], check=False)
        return tested.returncode


if __name__ == "__main__":
    sys.exit(main())
