#!/usr/bin/env python3
"""Verbatim AE2 Energy Cell display/comparator methods in a minimal Java host.

World lookup supplies only local stored/max values; ticking, grid and NeoForge
lifecycle are outside this method-level oracle.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import random
import struct
import subprocess

import upstream
from controller_oracle import fragment

ROOT = upstream.ROOT
WORK = ROOT / 'target/energy-cell-oracle'
BLOCK = 'block/networking/EnergyCellBlock.java'
ENTITY = 'blockentity/networking/EnergyCellBlockEntity.java'


def bits(value):
    return struct.unpack('>Q', struct.pack('>d', value))[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    parser.add_argument('--cases', type=int, default=100000)
    args = parser.parse_args()
    if args.cases < 1000:
        parser.error('at least 1000 cases required')
    upstream.verify(upstream.SOURCE)
    base = upstream.SOURCE / 'src/main/java/appeng'
    parts = {'display': fragment((base/ENTITY).read_text(), 'public static int getStorageLevelFromFillFactor('),
             'signal': fragment((base/BLOCK).read_text(), 'public int getAnalogOutputSignal(')}
    source = '''
import java.io.*;
import java.util.*;
public class EnergyCellOracle {
 static class Mth {
  static double clamp(double v,double a,double b){return v<a?a:Math.min(v,b);}
  static int floor(double v){int i=(int)v;return v<(double)i?i-1:i;}
 }
 static class BlockState {} static class Level {} static class BlockPos {}
 static class EnergyCellBlockEntity {
  double current,maximum;
  double getAECurrentPower(){return current;} double getAEMaxPower(){return maximum;}
  @@DISPLAY@@
 }
 static class EnergyCellBlock {
  static final int MAX_FULLNESS=4; EnergyCellBlockEntity cell;
  EnergyCellBlockEntity getBlockEntity(Level l,BlockPos p){return cell;}
  @@SIGNAL@@
 }
 public static void main(String[] args)throws Exception {
  var input=new BufferedReader(new InputStreamReader(System.in));
  var out=new StringBuilder();var block=new EnergyCellBlock();block.cell=new EnergyCellBlockEntity();
  String line;while((line=input.readLine())!=null){
   var row=new StringTokenizer(line);
   double current=Double.longBitsToDouble(Long.parseUnsignedLong(row.nextToken()));
   double maximum=Double.longBitsToDouble(Long.parseUnsignedLong(row.nextToken()));
   block.cell.current=current;block.cell.maximum=maximum;
   out.append(EnergyCellBlockEntity.getStorageLevelFromFillFactor(current/maximum)).append(' ')
      .append(block.getAnalogOutputSignal(null,null,null)).append('\\n');
  }System.out.print(out);
 }
}
'''
    for key,value in parts.items():
        source=source.replace('@@'+key.upper()+'@@',value)
    WORK.mkdir(parents=True,exist_ok=True)
    java=WORK/'EnergyCellOracle.java';java.write_text(source)
    classes=WORK/'classes';classes.mkdir(exist_ok=True)
    binary=lambda name:str(args.java_home/'bin'/name) if args.java_home else name
    subprocess.run([binary('javac'),'--release','21','-d',str(classes),str(java)],check=True)
    rows=[]
    for maximum in [200000.0,1600000.0]:
        for value in [0.,math.ulp(0.),1e-6,maximum,maximum*2,-1.,math.inf,-math.inf,math.nan]:
            rows.append((bits(value),bits(maximum)))
        for ratio in [.24,.49,.74,.99,*[i/14 for i in range(15)]]:
            value=maximum*ratio
            for v in [math.nextafter(value,-math.inf),value,math.nextafter(value,math.inf)]:
                rows.append((bits(v),bits(maximum)))
    rng=random.Random(0xAE2CE11)
    while len(rows)<args.cases:
        maximum=rng.choice([200000.,1600000.])
        # Exercise normal storage, exact/near boundaries, and IEEE/narrowing cases.
        current=rng.uniform(0,maximum) if len(rows)%3 else struct.unpack('>d',rng.getrandbits(64).to_bytes(8,'big'))[0]
        rows.append((bits(current),bits(maximum)))
    data=''.join(f'{a} {b}\n' for a,b in rows).encode()
    (WORK/'input.txt').write_bytes(data)
    reference=subprocess.run([binary('java'),'-cp',str(classes),'EnergyCellOracle'],input=data,capture_output=True,check=True)
    port=subprocess.run(['cargo','run','--locked','--quiet','--example','energy_cell_oracle'],cwd=ROOT,input=data,capture_output=True,check=True)
    (WORK/'java.txt').write_bytes(reference.stdout);(WORK/'rust.txt').write_bytes(port.stdout)
    if reference.stdout!=port.stdout:
        for i,(a,b) in enumerate(zip(reference.stdout.splitlines(),port.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i} bits={rows[i]} Java={a!r} Rust={b!r}')
        raise AssertionError('oracle output lengths differ')
    report={'passed':True,'cases':len(rows),'top_level_method_comparisons':len(rows)*2,
            'revision':upstream.LOCK['revision'],'input_sha256':hashlib.sha256(data).hexdigest(),
            'source_sha256':{p:hashlib.sha256((base/p).read_bytes()).hexdigest() for p in [BLOCK,ENTITY]},
            'verbatim_fragment_sha256':{p:hashlib.sha256(v.encode()).hexdigest() for p,v in parts.items()},
            'scope':'verbatim EnergyCellBlock comparator and EnergyCellBlockEntity fullness methods; local energy/max stub host',
            'limits':['Mth clamp/floor supplied by minimal host','no grid/tick manager/chunk/event lifecycle','not a Java/NeoForge world oracle']}
    (ROOT/'docs/energy-cell-visual-oracle.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__=='__main__':
    main()
