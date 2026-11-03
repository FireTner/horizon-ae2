#!/usr/bin/env python3
"""Verified, ignored Minecraft 1.21.1 Java reference dependencies for differential checks.

Downloads only artifacts pinned by the official version metadata. Never runs the
Minecraft client entry point or modifies an installed game directory.
"""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT/'target/minecraft-1.21.1-reference'
META_SHA1 = '018296bcb207bea6fd25b05f08eee3942dcec406'
META_URL = 'https://piston-meta.mojang.com/v1/packages/'+META_SHA1+'/1.21.1.json'


def verify(path, sha1, size=None):
    return path.is_file() and (size is None or path.stat().st_size == size) and hashlib.sha1(path.read_bytes()).hexdigest() == sha1


def fetch(url, path, sha1, size=None):
    if verify(path, sha1, size):
        return
    if not url.startswith('https://'):
        raise ValueError('reference downloads require HTTPS')
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name+'.download')
    with urllib.request.urlopen(url, timeout=60) as response, temporary.open('wb') as output:
        while chunk := response.read(1024*1024):
            output.write(chunk)
    if not verify(temporary, sha1, size):
        temporary.unlink(missing_ok=True)
        raise ValueError('reference hash/size mismatch: '+path.name)
    temporary.replace(path)


def prepare():
    WORK.mkdir(parents=True, exist_ok=True)
    fetch(META_URL, WORK/'version.json', META_SHA1)
    version = json.loads((WORK/'version.json').read_text())
    if version['id'] != '1.21.1':
        raise ValueError('unexpected reference version')
    for key, name in [('client','client.jar'), ('client_mappings','client_mappings.txt')]:
        artifact = version['downloads'][key]
        fetch(artifact['url'], WORK/name, artifact['sha1'], artifact['size'])
    artifacts = [lib['downloads']['artifact'] for lib in version['libraries']
                 if 'artifact' in lib.get('downloads', {}) and ':natives-' not in lib['name']]
    def library(artifact):
        path = WORK/'libraries'/artifact['path']
        if not path.resolve().is_relative_to((WORK/'libraries').resolve()):
            raise ValueError('reference artifact path escapes its cache')
        fetch(artifact['url'], path, artifact['sha1'], artifact['size'])
        return path
    with ThreadPoolExecutor(max_workers=6) as pool:
        jars = list(pool.map(library, artifacts))
    return os.pathsep.join(map(str, [WORK/'client.jar', *jars]))


if __name__ == '__main__':
    print(prepare())
