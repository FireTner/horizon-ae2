#!/usr/bin/env python3
"""Compare unchanged Controller visual methods in a ready/loaded stub host.

Geometry, signed coordinate parity and offline/online/conflict precedence use the
verbatim pinned Java methods. Grid power/conflict/ready/lifecycle are stubbed;
this is not a Java/NeoForge world or ControllerValidator oracle.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import subprocess

import upstream

ROOT = upstream.ROOT
WORK = ROOT / 'target/controller-oracle'
BLOCK = 'block/networking/ControllerBlock.java'
ENTITY = 'blockentity/networking/ControllerBlockEntity.java'


def fragment(text, header):
    start = text.index(header)
    end = text.index('{', start) + 1
    depth = 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[start:end]


def generate():
    base = upstream.SOURCE / 'src/main/java/appeng'
    block = (base / BLOCK).read_text()
    entity = (base / ENTITY).read_text()
    parts = {
        'block_state': fragment(block, 'public enum ControllerBlockState'),
        'render_type': fragment(block, 'public enum ControllerRenderType'),
        'get_controller_type': fragment(block, 'private BlockState getControllerType('),
        'is_controller': fragment(block, 'private static boolean isController('),
        'update_state': fragment(entity, 'public void updateState()'),
        'check_controller': fragment(entity, 'private boolean checkController('),
    }
    source = r'''
package oracle;
import java.io.*;
import java.util.*;
import oracle.ControllerOracle.ControllerBlock.ControllerBlockState;
public class ControllerOracle {
 interface StringRepresentable { String getSerializedName(); }
 static final class Property<T> {}
 record BlockPos(int x,int y,int z) { int getX(){return x;} int getY(){return y;} int getZ(){return z;} }
 static class Block { static final int UPDATE_CLIENTS=2; }
 static class AEBlocks {
  static final Object TOKEN=new Object(); static final Definition CONTROLLER=new Definition();
  static class Definition { Object block(){return TOKEN;} }
 }
 static class ControllerBlock {
 @@BLOCK_STATE@@
 @@RENDER_TYPE@@
 static final Property<ControllerBlockState> CONTROLLER_STATE=new Property<>();
 static final Property<ControllerRenderType> CONTROLLER_TYPE=new Property<>();
 @@GET_CONTROLLER_TYPE@@
 @@IS_CONTROLLER@@
 }
 static class BlockState {
  Object block; ControllerBlock.ControllerBlockState state; ControllerBlock.ControllerRenderType type;
  BlockState(Object b,ControllerBlock.ControllerBlockState s,ControllerBlock.ControllerRenderType t){block=b;state=s;type=t;}
  boolean is(Object b){return block==b;}
  @SuppressWarnings("unchecked") <T> T getValue(Property<T> p){return (T)(p==ControllerBlock.CONTROLLER_STATE?state:type);}
  <T> BlockState setValue(Property<T> p,T value){return new BlockState(block,
    p==ControllerBlock.CONTROLLER_STATE?(ControllerBlock.ControllerBlockState)value:state,
    p==ControllerBlock.CONTROLLER_TYPE?(ControllerBlock.ControllerRenderType)value:type);}
 }
 interface LevelAccessor { BlockState getBlockState(BlockPos p); }
 static class Level implements LevelAccessor {
  BlockPos center; Set<BlockPos> neighbours=new HashSet<>(); BlockState state; ControllerBlockEntity entity;
  public BlockState getBlockState(BlockPos p){return p.equals(center)?state:new BlockState(neighbours.contains(p)?AEBlocks.TOKEN:null,state.state,state.type);}
  void setBlock(BlockPos p,BlockState s,int flags){if(!p.equals(center)||flags!=Block.UPDATE_CLIENTS)throw new AssertionError();state=s;}
 }
 enum ControllerState { NO_CONTROLLER,CONTROLLER_ONLINE,CONTROLLER_CONFLICT }
 static class Energy { boolean powered; boolean isNetworkPowered(){return powered;} }
 static class Pathing { boolean conflict; ControllerState getControllerState(){return conflict?ControllerState.CONTROLLER_CONFLICT:ControllerState.CONTROLLER_ONLINE;} }
 static class Grid { Energy energy=new Energy();Pathing pathing=new Pathing();Energy getEnergyService(){return energy;}Pathing getPathingService(){return pathing;} }
 static class MainNode { Grid grid=new Grid();boolean isReady(){return true;}Grid getGrid(){return grid;} }
 static class Platform { static Object getTickingBlockEntity(Level level,BlockPos pos){return level.entity;} }
 static class ControllerBlockEntity {
  Level level;BlockPos worldPosition;MainNode node=new MainNode();
  MainNode getMainNode(){return node;}Level getLevel(){return level;}
  @@UPDATE_STATE@@
  @@CHECK_CONTROLLER@@
 }
 public static void main(String[] args)throws Exception {
  var reader=new BufferedReader(new InputStreamReader(System.in));String line;
  int[][] offsets={{0,-1,0},{0,1,0},{0,0,-1},{0,0,1},{-1,0,0},{1,0,0}};
  while((line=reader.readLine())!=null){
   var tokens=new StringTokenizer(line);int[] v=new int[7];for(int i=0;i<7;i++)v[i]=Integer.parseInt(tokens.nextToken());
   var level=new Level();level.center=new BlockPos(v[0],v[1],v[2]);
   level.state=new BlockState(AEBlocks.TOKEN,ControllerBlock.ControllerBlockState.values()[v[6]],ControllerBlock.ControllerRenderType.block);
   for(int i=0;i<6;i++)if((v[3]&(1<<i))!=0)level.neighbours.add(new BlockPos(v[0]+offsets[i][0],v[1]+offsets[i][1],v[2]+offsets[i][2]));
   level.state=new ControllerBlock().getControllerType(level.state,level,level.center);
   var entity=new ControllerBlockEntity();entity.level=level;entity.worldPosition=level.center;level.entity=entity;
   entity.node.grid.energy.powered=v[4]!=0;entity.node.grid.pathing.conflict=v[5]!=0;
   entity.updateState();
   System.out.println(level.state.type.getSerializedName()+" "+level.state.state.getSerializedName());
  }
 }
}
'''
    for key, value in parts.items():
        source = source.replace('@@' + key.upper() + '@@', value)
    dest = WORK / 'src/ControllerOracle.java'
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(source)
    return dest, parts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    parser.add_argument('--cases', type=int, default=4000)
    args = parser.parse_args()
    if args.cases < 64:
        parser.error('at least 64 cases are needed for every neighbour mask')
    upstream.verify(upstream.SOURCE)
    java, parts = generate()
    binary = lambda name: str(args.java_home / 'bin' / name) if args.java_home else name
    classes = WORK / 'classes'; classes.mkdir(exist_ok=True)
    subprocess.run([binary('javac'), '--release', '21', '-d', str(classes), str(java)], check=True)
    rng = random.Random(0xAE2C017)
    edges = [-2147483648, -2147483647, -1024, -1, 0, 1, 1024, 2147483647]
    rows = []
    for i in range(args.cases):
        pos = [0,0,0] if i < 64 else [rng.choice(edges) if rng.randrange(3)==0 else rng.randint(-30000000,30000000) for _ in range(3)]
        rows.append([*pos, i % 64, rng.randrange(2), rng.randrange(2), rng.randrange(3)])
    data = ''.join(' '.join(map(str, row))+'\n' for row in rows).encode()
    (WORK / 'input.txt').write_bytes(data)
    reference = subprocess.run([binary('java'), '-cp', str(classes), 'oracle.ControllerOracle'], input=data, capture_output=True, check=True)
    port = subprocess.run(['cargo','run','--locked','--quiet','--example','controller_oracle'], cwd=ROOT, input=data, capture_output=True, check=True)
    (WORK / 'java.txt').write_bytes(reference.stdout); (WORK / 'rust.txt').write_bytes(port.stdout)
    if reference.stdout != port.stdout:
        for i,(a,b) in enumerate(zip(reference.stdout.splitlines(),port.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i}: {rows[i]} Java={a!r} Rust={b!r}')
        raise AssertionError('oracle output lengths differ')
    base = upstream.SOURCE / 'src/main/java/appeng'
    report = {'passed':True,'cases':len(rows),'top_level_method_comparisons':len(rows)*2,
              'revision':upstream.LOCK['revision'],'input_sha256':hashlib.sha256(data).hexdigest(),
              'source_sha256':{p:hashlib.sha256((base/p).read_bytes()).hexdigest() for p in [BLOCK,ENTITY]},
              'verbatim_fragment_sha256':{p:hashlib.sha256(v.encode()).hexdigest() for p,v in parts.items()},
              'scope':'verbatim ControllerBlock getControllerType and ControllerBlockEntity updateState in ready/loaded/grid-present stub host',
              'limits':['power/conflict injected through stub services','grid validation/pathing not executed by this oracle',
                        'world events/chunks/spatial IO/UI, full block entity and Java/NeoForge world parity remain open']}
    (ROOT/'docs/controller-visual-oracle.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__=='__main__':
    main()
