#!/usr/bin/env python3
"""Enforce the repository's English-only documentation policy."""
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
if (ROOT / '.git').exists():
    paths = subprocess.check_output([
        'git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z',
        '*.md', '*.rst', '*.txt',
    ], cwd=ROOT).decode().split('\0')
else:
    # Corresponding source extracted from a native archive has no Git metadata.
    import os
    paths = []
    for folder, dirs, files in os.walk(ROOT):
        dirs[:] = [name for name in dirs if name not in {'target', '.git', '__pycache__'}]
        paths.extend(str((Path(folder) / name).relative_to(ROOT)) for name in files
                     if Path(name).suffix in {'.md', '.rst', '.txt'})
failures = []
for name in sorted(set(filter(None, paths))):
    path = ROOT / name
    if not path.is_file():
        continue
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if re.search(r'[\u0400-\u052f]', line):
            failures.append(f'{name}:{number}: documentation must be written in English')
if failures:
    raise SystemExit('\n'.join(failures))
print('English-only documentation check passed')
