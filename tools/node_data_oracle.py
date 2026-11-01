#!/usr/bin/env python3
"""Compare GridNode's verbatim owner-ignoring saved-data equality.

CompoundTag and tag-value equality are stubbed maps/strings, not Minecraft NBT.
This does not execute readiness/destroy/Grid services or full loadFromNBT.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import subprocess

import upstream
from controller_oracle import fragment

ROOT = upstream.ROOT
WORK = ROOT / 'target/node-data-oracle'
SOURCE = upstream.SOURCE / 'src/main/java/appeng/me/GridNode.java'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    parser.add_argument('--cases', type=int, default=20000)
    args = parser.parse_args()
    upstream.verify(upstream.SOURCE)
    method = fragment(SOURCE.read_text(), 'private boolean areTagsEqualIgnoringPlayerId(')
    source = r'''
import java.io.*;
import java.util.*;
public class NodeDataOracle {
 static class CompoundTag {
  Map<String,String> data=new TreeMap<>();Set<String> getAllKeys(){return data.keySet();}
  Object get(String k){return data.get(k);}
 }
 @@METHOD@@
 static CompoundTag read(StringTokenizer s){int n=Integer.parseInt(s.nextToken());if(n<0)return null;var c=new CompoundTag();for(int i=0;i<n;i++)c.data.put(s.nextToken(),s.nextToken());return c;}
 public static void main(String[] args)throws Exception {
  var reader=new BufferedReader(new InputStreamReader(System.in));var host=new NodeDataOracle();String line;
  while((line=reader.readLine())!=null){var s=new StringTokenizer(line);var a=read(s);var b=read(s);System.out.println(host.areTagsEqualIgnoringPlayerId(a,b)?1:0);if(s.hasMoreTokens())throw new AssertionError();}
 }
}
'''.replace('@@METHOD@@', method)
    WORK.mkdir(parents=True, exist_ok=True)
    java = WORK / 'NodeDataOracle.java'
    java.write_text(source)
    binary = lambda name: str(args.java_home / 'bin' / name) if args.java_home else name
    subprocess.run([binary('javac'), '--release', '21', '-d', str(WORK), str(java)], check=True)
    rng = random.Random(0xAE20DA7A)
    special = [None, {}, {'p': 'i:1'}, {'p': 'i:2'}, {'e': 'd:0'}, {'e': 'd:-0'}, {'e': 'i:0'}, {'p': 'i:1', 'e': 'd:0'}]
    maps = [(a, b) for a in special for b in special]
    for _ in range(args.cases):
        a = None if rng.randrange(10) == 0 else {k: f'{rng.choice(["i", "d", "s"])}:{rng.randrange(-2,3)}' for k in ['p', 'e', 'a', 'b', 'extra'] if rng.randrange(2)}
        if rng.randrange(2) and a is not None:
            b = dict(a)
            if rng.randrange(2):
                b['p'] = f'i:{rng.randrange(100)}'
            else:
                b.pop(rng.choice(list(b)), None) if b else None
        else:
            b = rng.choice(special)
        maps.append((a, b))
    def encode(data):
        return '-1' if data is None else str(len(data)) + ''.join(f' {k} {v}' for k, v in sorted(data.items()))
    data = ''.join(encode(a) + ' ' + encode(b) + '\n' for a, b in maps).encode()
    (WORK / 'input.txt').write_bytes(data)
    reference = subprocess.run([binary('java'), '-cp', str(WORK), 'NodeDataOracle'], input=data, capture_output=True, check=True)
    port = subprocess.run(['cargo', 'run', '--locked', '--quiet', '--example', 'node_data_oracle'], cwd=ROOT, input=data, capture_output=True, check=True)
    (WORK / 'java.txt').write_bytes(reference.stdout)
    (WORK / 'rust.txt').write_bytes(port.stdout)
    if reference.stdout != port.stdout:
        for i, (a, b) in enumerate(zip(reference.stdout.splitlines(), port.stdout.splitlines())):
            if a != b:
                raise AssertionError(f'case {i}: {maps[i]} Java={a!r} Rust={b!r}')
        raise AssertionError('output lengths differ')
    report = {'passed': True, 'cases': len(maps), 'revision': upstream.LOCK['revision'],
              'input_sha256': hashlib.sha256(data).hexdigest(),
              'output_sha256': hashlib.sha256(reference.stdout).hexdigest(),
              'source_sha256': hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
              'verbatim_fragment_sha256': hashlib.sha256(method.encode()).hexdigest(),
              'scope': 'verbatim GridNode.areTagsEqualIgnoringPlayerId; missing/empty/owner-only maps, changed/missing non-owner keys and typed-value identity',
              'limits': ['CompoundTag and tag-value equality are map/string stubs, not Minecraft NBT implementations',
                         'readiness/destroy/Grid service callbacks and full GridNode.loadFromNBT are not executed',
                         'native typed NBT codec/lifecycle tests are separate evidence; nonfinite/nested tag equality is not proved']}
    (ROOT / 'docs/node-data-oracle.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
