#!/usr/bin/env python3
"""Compare the source center-part placement target and admission predicates.

Uses verbatim PartPlacement methods. Level/PartHelper/players are controlled
fixtures; side-part segment selection, entities, settings and game events are
separate native evidence.
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
SOURCE = upstream.SOURCE / 'src/main/java/appeng/parts/PartPlacement.java'
WORK = ROOT / 'target/part-placement-oracle'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    parser.add_argument('--cases', type=int, default=50000)
    args = parser.parse_args()
    upstream.verify(upstream.SOURCE)
    methods = '\n'.join(fragment(SOURCE.read_text(), signature) for signature in [
        'public static Placement getPartPlacement(', 'public static boolean canPlacePartOnBlock('])
    java = r'''
import java.io.*;
import java.util.*;
public class PartPlacementOracle {
 @interface Nullable {}
 enum Direction {
  DOWN(0,-1,0),UP(0,1,0),NORTH(0,0,-1),SOUTH(0,0,1),WEST(-1,0,0),EAST(1,0,0);
  final int x,y,z;Direction(int x,int y,int z){this.x=x;this.y=y;this.z=z;}
  Direction getOpposite(){return values()[ordinal()^1];}
 }
 record BlockPos(int x,int y,int z){BlockPos relative(Direction d){return new BlockPos(x+d.x,y+d.y,z+d.z);}}
 record Vec3(double x,double y,double z){}
 record Placement(BlockPos pos,Direction side){}
 static class Player {final boolean ctrl; Player(boolean c){ctrl=c;} boolean getData(Object type){return ctrl;}}
 static class AEAttachmentTypes {static final Object HOLDING_CTRL=new Object();}
 static class ItemStack {}
 static class Host {final int type;Host(int t){type=t;}boolean canAddPart(ItemStack stack,Direction side){return type==3;}}
 static class Level {final BlockPos clicked;final int a,b;Level(BlockPos p,int a,int b){clicked=p;this.a=a;this.b=b;}int at(BlockPos p){return p.equals(clicked)?a:b;}}
 static class PartHelper {
  static Host getPartHost(Level l,BlockPos p){return l.at(p)>=3?new Host(l.at(p)):null;}
  static boolean canPlacePartHost(Player player,Level l,BlockPos p){return l.at(p)==1;}
 }
 static Direction tryReplaceCableSegment(Level level,ItemStack stack,BlockPos pos,Vec3 click){return null;}
 @@METHODS@@
 public static void main(String[] args)throws Exception {
  var reader=new BufferedReader(new InputStreamReader(System.in));String line;
  while((line=reader.readLine())!=null){var s=new StringTokenizer(line);int x=Integer.parseInt(s.nextToken()),y=Integer.parseInt(s.nextToken()),z=Integer.parseInt(s.nextToken());var side=Direction.values()[Integer.parseInt(s.nextToken())];int ctrl=Integer.parseInt(s.nextToken()),a=Integer.parseInt(s.nextToken()),b=Integer.parseInt(s.nextToken());var pos=new BlockPos(x,y,z);var result=getPartPlacement(ctrl<0?null:new Player(ctrl==1),new Level(pos,a,b),new ItemStack(),pos,side,new Vec3(x+.5,y+.5,z+.5));if(result==null)System.out.println("-");else System.out.println(result.pos().x()+" "+result.pos().y()+" "+result.pos().z()+" "+result.side().ordinal());}
 }
}
'''.replace('@@METHODS@@', methods)
    WORK.mkdir(parents=True, exist_ok=True)
    path = WORK/'PartPlacementOracle.java'; path.write_text(java)
    binary = lambda name: str(args.java_home/'bin'/name) if args.java_home else name
    subprocess.run([binary('javac'), str(path)], check=True)
    positions = [(0,0,0),(7,73,-8),(2147483647,2147483647,2147483647),(-2147483648,-2147483648,-2147483648)]
    cases = [(*pos, side, ctrl, a, b) for pos in positions for side in range(6) for ctrl in [-1,0,1] for a in range(6) for b in range(6)]
    rng = random.Random(0xAE2C011)
    cases += [(*(rng.randint(-2147483648,2147483647) for _ in range(3)),rng.randrange(6),rng.randrange(-1,2),rng.randrange(6),rng.randrange(6)) for _ in range(args.cases)]
    data = ''.join(' '.join(map(str,c))+'\n' for c in cases).encode()
    ref = subprocess.run([binary('java'),'-cp',str(WORK),'PartPlacementOracle'],input=data,capture_output=True,check=True).stdout
    port = subprocess.run(['cargo','run','--locked','--quiet','--example','part_placement_oracle'],cwd=ROOT,input=data,capture_output=True,check=True).stdout
    (WORK/'input.txt').write_bytes(data); (WORK/'java.txt').write_bytes(ref); (WORK/'rust.txt').write_bytes(port)
    if ref != port:
        for i,(a,b) in enumerate(zip(ref.splitlines(),port.splitlines())):
            if a!=b: raise AssertionError(f'{cases[i]} Java={a!r} Rust={b!r}')
        raise AssertionError('different output lengths')
    report={'passed':True,'cases':len(cases),'revision':upstream.LOCK['revision'],
            'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
            'port_sha256':hashlib.sha256((ROOT/'mixins/ae2/part_placement.rs').read_bytes()).hexdigest(),
            'verbatim_fragment_sha256':hashlib.sha256(methods.encode()).hexdigest(),
            'input_sha256':hashlib.sha256(data).hexdigest(),'output_sha256':hashlib.sha256(ref).hexdigest(),
            'scope':'Original getPartPlacement and canPlacePartOnBlock for central cable parts, all faces, CTRL/null player, existing hosts, replacement/admission and Java int boundary offsets',
            'limits':['Level/PartHelper/host registry and permission results are fixtures',
                      'Cable segment replacement is irrelevant to central cable admission and stubbed; side parts/facades are not exercised',
                      'Place mutation, collision, security, owner mapping, settings, sound, consumption and world lifecycle require separate native/game evidence']}
    (ROOT/'docs/part-placement-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


if __name__ == '__main__':
    main()
