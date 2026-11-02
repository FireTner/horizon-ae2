#!/usr/bin/env python3
"""Differential PartPlacement.place control-flow comparison with declared host stubs."""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import random
import subprocess
import upstream
from controller_oracle import fragment
ROOT = upstream.ROOT
SOURCE = upstream.SOURCE/'src/main/java/appeng/parts/PartPlacement.java'
WORK = ROOT/'target/part-use-oracle'

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--java-home',type=Path,default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    args=p.parse_args();upstream.verify(upstream.SOURCE)
    method=fragment(SOURCE.read_text(),'public static InteractionResult place(')
    java=r'''
import java.io.*;
import java.util.*;
public class PartUseOracle {
 enum InteractionResult {PASS,FAIL,SUCCESS,CONSUME; static InteractionResult sidedSuccess(boolean client){return client?SUCCESS:CONSUME;}}
 static class BlockPos {} static class Direction {} static class Vec3 {}
 interface IPartItem<T> {} static class PartItem implements IPartItem<Object> {}
 static class ItemStack {static final ItemStack EMPTY=new ItemStack(false,0);Object item;int count;UseOnContext context;
  ItemStack(boolean part,int count){item=part?new PartItem():new Object();this.count=count;}
  Object getItem(){return item;}Object getComponents(){return null;}int getCount(){return count;}
  void shrink(int n){count-=n;context.events.append('C');}
 }
 static class Player {boolean creative;UseOnContext context;boolean isCreative(){return creative;}void setItemInHand(Object hand,ItemStack item){context.empty=true;context.events.append('E');}}
 static class Level {boolean isClientSide;UseOnContext context;boolean isClientSide(){return isClientSide;}}
 static class UseOnContext {Player player;Level level=new Level();ItemStack stack;boolean target,added,empty;StringBuilder events=new StringBuilder();
  Player getPlayer(){return player;}Level getLevel(){return level;}BlockPos getClickedPos(){return new BlockPos();}ItemStack getItemInHand(){return stack;}Direction getClickedFace(){return new Direction();}Vec3 getClickLocation(){return new Vec3();}Object getHand(){return null;}
 }
 static class Platform {static UseOnContext context;static void sendImmediateBlockEntityUpdate(Player player,BlockPos pos){context.events.append('U');}}
 record Placement(BlockPos pos,Direction side) {}
 static class PartPlacement {
  static Placement getPartPlacement(Player p,Level l,ItemStack s,BlockPos pos,Direction side,Vec3 v){l.context.events.append('T');return l.context.target?new Placement(pos,side):null;}
  static Object placePart(Player p,Level l,IPartItem<?> item,Object components,BlockPos pos,Direction side){l.context.events.append('P');return l.context.added?new Object():null;}
  @@METHOD@@
 }
 public static void main(String[] args)throws Exception {var reader=new BufferedReader(new InputStreamReader(System.in));String line;
  while((line=reader.readLine())!=null){var a=Arrays.stream(line.split(" ")).mapToInt(Integer::parseInt).toArray();var c=new UseOnContext();c.level.context=c;c.level.isClientSide=a[1]!=0;c.stack=new ItemStack(a[0]!=0,a[4]);c.stack.context=c;c.target=a[5]!=0;c.added=a[6]!=0;
   if(a[2]!=0){c.player=new Player();c.player.creative=a[3]!=0;c.player.context=c;}Platform.context=c;
   var r=PartPlacement.place(c);System.out.println(r+" "+c.stack.count+" "+(c.empty?1:0)+" "+(c.events.length()==0?"-":c.events)+" "+(r!=InteractionResult.PASS?1:0)+" "+(r==InteractionResult.SUCCESS?1:0));
  }
 }
}
'''.replace('@@METHOD@@',method)
    WORK.mkdir(parents=True,exist_ok=True);path=WORK/'PartUseOracle.java';path.write_text(java)
    binary=lambda name:str(args.java_home/'bin'/name) if args.java_home else name
    subprocess.run([binary('javac'),str(path)],check=True)
    cases=[(part,client,player,creative,count,target,added) for part,client,player,creative,target,added in itertools.product(range(2),repeat=6) for count in [1,2,64,99,2147483647]]
    rng=random.Random(0xAE2CA11)
    cases += [(rng.randrange(2),rng.randrange(2),rng.randrange(2),rng.randrange(2),rng.randrange(1,2147483648),rng.randrange(2),rng.randrange(2)) for _ in range(10000)]
    data=''.join(' '.join(map(str,c))+'\n' for c in cases).encode()
    ref=subprocess.run([binary('java'),'-cp',str(WORK),'PartUseOracle'],input=data,capture_output=True,check=True).stdout
    port=subprocess.run(['cargo','run','--locked','--quiet','--example','part_use_oracle'],cwd=ROOT,input=data,capture_output=True,check=True).stdout
    for name,content in [('input.txt',data),('java.txt',ref),('rust.txt',port)]:(WORK/name).write_bytes(content)
    if ref!=port:
        for i,(a,b) in enumerate(zip(ref.splitlines(),port.splitlines())):
            if a!=b:raise AssertionError(f'{cases[i]} Java={a!r} Rust={b!r}')
        raise AssertionError('output length differs')
    report={'passed':True,'cases':len(cases),'revision':upstream.LOCK['revision'],
        'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),'verbatim_fragment_sha256':hashlib.sha256(method.encode()).hexdigest(),
        'port_sha256':hashlib.sha256((ROOT/'mixins/ae2/part_use.rs').read_bytes()).hexdigest(),'input_sha256':hashlib.sha256(data).hexdigest(),'output_sha256':hashlib.sha256(ref).hexdigest(),
        'scope':'Verbatim PartPlacement.place result, target/place/update order, server-only nullable-player/creative consumption and empty-hand replacement',
        'limits':['World target lookup and part mutation are boolean stubs; geometry, registration, security and networking require separate evidence',
                  'ItemStack uses positive input counts and a stub part marker; real item/registry normalization is excluded',
                  'Immediate update records the Platform method call; the platform transport and native client prediction are separate']}
    (ROOT/'docs/part-use-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':main()
