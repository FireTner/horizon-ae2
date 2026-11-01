#!/usr/bin/env python3
"""Compare pinned cable methods/API and unchanged CableBusStorage in stub hosts.

No Minecraft/NeoForge world, registry/codec, renderer or complete multipart host
is executed. Geometry compares raw addBox calls, not normalized VoxelShapes.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import shutil
import subprocess

import upstream
from controller_oracle import fragment

ROOT = upstream.ROOT
WORK = ROOT / 'target/cable-oracle'
BASE = upstream.SOURCE / 'src/main/java/appeng'


def generate():
    files = ['api/util/' + name + '.java' for name in
             ['AECableType', 'AECableVariant', 'AECableSize', 'AEColor']]
    files.append('parts/CableBusStorage.java')
    src = WORK / 'src'
    src.mkdir(parents=True, exist_ok=True)
    for name in files:
        dest = src / 'appeng' / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(BASE / name, dest)
    stubs = {
        'org/jetbrains/annotations/Nullable.java': 'package org.jetbrains.annotations; import java.lang.annotation.*; @Target({ElementType.TYPE_USE,ElementType.FIELD,ElementType.PARAMETER}) public @interface Nullable {}',
        'net/minecraft/core/Direction.java': 'package net.minecraft.core; public enum Direction { DOWN,UP,NORTH,SOUTH,WEST,EAST; }',
        'appeng/api/parts/IPart.java': 'package appeng.api.parts; public interface IPart {}',
        'appeng/api/parts/IFacadePart.java': 'package appeng.api.parts; public interface IFacadePart {}',
        'appeng/api/implementations/parts/ICablePart.java': 'package appeng.api.implementations.parts; public interface ICablePart extends appeng.api.parts.IPart {}',
        'com/mojang/serialization/Codec.java': 'package com.mojang.serialization; public class Codec<T> {}',
        'net/minecraft/network/FriendlyByteBuf.java': 'package net.minecraft.network; public class FriendlyByteBuf {}',
        'net/minecraft/network/codec/StreamCodec.java': 'package net.minecraft.network.codec; public interface StreamCodec<B,T> {}',
        'net/neoforged/neoforge/network/codec/NeoForgeStreamCodecs.java': 'package net.neoforged.neoforge.network.codec; public class NeoForgeStreamCodecs { public static <T extends Enum<T>> net.minecraft.network.codec.StreamCodec<net.minecraft.network.FriendlyByteBuf,T> enumCodec(Class<T> t){return null;} }',
        'net/minecraft/util/StringRepresentable.java': 'package net.minecraft.util; public interface StringRepresentable {String getSerializedName(); static <T extends Enum<T> & StringRepresentable> com.mojang.serialization.Codec<T> fromEnum(java.util.function.Supplier<T[]> s){return null;} }',
        'net/minecraft/client/renderer/block/model/BakedQuad.java': 'package net.minecraft.client.renderer.block.model; public class BakedQuad { public int getTintIndex(){return 0;} }',
        'net/minecraft/world/item/DyeColor.java': 'package net.minecraft.world.item; public enum DyeColor {WHITE,ORANGE,MAGENTA,LIGHT_BLUE,YELLOW,LIME,PINK,GRAY,LIGHT_GRAY,CYAN,PURPLE,BLUE,BROWN,GREEN,RED,BLACK;}',
    }
    for name, body in stubs.items():
        dest = src / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(body + '\n')
    texts = {name: (BASE / ('parts/networking/' + name + '.java')).read_text()
             for name in ['CablePart', 'GlassCablePart', 'CoveredCablePart', 'SmartCablePart', 'DenseCablePart']}
    signatures = {
        'connection_box': 'protected static void addConnectionBox(',
        'non_dense_boxes': 'protected void addNonDenseBoxes(',
        'length': 'public float getCableConnectionLength(',
        'visual': 'private byte getVisualChannels(',
        'write': 'public void writeToStream(',
        'read': 'public boolean readFromStream(',
        'channels': 'public int getChannelsOnSide(',
    }
    parts = {key: fragment(texts['CablePart'], header) for key, header in signatures.items()}
    for name in ['GlassCablePart', 'CoveredCablePart', 'SmartCablePart', 'DenseCablePart']:
        parts[name] = fragment(texts[name], 'public void getBoxes(')
    parts['compatible'] = fragment((BASE / 'me/InWorldGridNode.java').read_text(), 'private boolean hasCompatibleColor(')
    source = r'''
package oracle;
import java.io.*;
import java.util.*;
import java.util.function.Predicate;
import org.jetbrains.annotations.Nullable;
import net.minecraft.core.Direction;
import appeng.api.util.*;
import appeng.parts.CableBusStorage;
import appeng.api.parts.*;
import appeng.api.implementations.parts.ICablePart;
public class CableOracle {
 enum ChannelMode { DEFAULT,INFINITE }
 static class RegistryFriendlyByteBuf {
  ByteArrayOutputStream out=new ByteArrayOutputStream(); byte[] input; int cursor;
  RegistryFriendlyByteBuf() {} RegistryFriendlyByteBuf(byte[] b){input=b;}
  void writeByte(byte b){out.write(b);} byte readByte(){return input[cursor++];}
 }
 static class Base {boolean baseChanged;void writeToStream(RegistryFriendlyByteBuf b){} boolean readFromStream(RegistryFriendlyByteBuf b){return baseChanged;} }
 record Connection(int channels) {int getUsedChannels(){return channels;} }
 static class Node {
  int max;ChannelMode mode;AEColor color;List<Connection> connections=new ArrayList<>();Map<Direction,Connection> external=new EnumMap<>(Direction.class);
  Node getGrid(){return this;}Node getPathingService(){return this;}ChannelMode getChannelMode(){return mode;}
  int getMaxChannels(){return max;} List<Connection> getConnections(){return connections;}Map<Direction,Connection> getInWorldConnections(){return external;}
  AEColor getGridColor(){return color;}
 }
 static class ColorNode {AEColor color;AEColor getGridColor(){return color;} @@COMPATIBLE@@ }
 static class Part {Double length;Node node;double getCableConnectionLength(AECableType t){return length;}Node getGridNode(){return node;} }
 static class Host {Part[] parts=new Part[6];Part getPart(Direction d){return parts[d.ordinal()];} }
 interface IPartCollisionHelper {void addBox(double a,double b,double c,double d,double e,double f);boolean isBBCollision();}
 static class Boxes implements IPartCollisionHelper {
  boolean bb;List<double[]> calls=new ArrayList<>();public boolean isBBCollision(){return bb;}
  public void addBox(double a,double b,double c,double d,double e,double f){calls.add(new double[]{a,b,c,d,e,f});}
 }
 static abstract class CablePart extends Base {
  int[] channelsOnSide=new int[6];Set<Direction> connections=EnumSet.noneOf(Direction.class);Host host=new Host();Node node;boolean powered=true;int dense;
  abstract AECableType getCableConnectionType();abstract void getBoxes(IPartCollisionHelper b,Predicate<@Nullable Direction> f);
  Node getGridNode(){return node;}Host getHost(){return host;}Set<Direction> getConnections(){return connections;}
  void setConnections(Set<Direction> c){connections=c;}void updateConnections(){}boolean isDense(Direction d){return (dense&(1<<d.ordinal()))!=0;}
  boolean isPowered(){return powered;}void setChannelsOnSide(int i,int ch){channelsOnSide[i]=ch;}
  @@CONNECTION_BOX@@ @@NON_DENSE_BOXES@@ @@LENGTH@@ @@VISUAL@@ @@WRITE@@ @@READ@@ @@CHANNELS@@
 }
 static class Glass extends CablePart {AECableType getCableConnectionType(){return AECableType.GLASS;}@@GLASSCABLEPART@@}
 static class Covered extends CablePart {AECableType getCableConnectionType(){return AECableType.COVERED;}@@COVEREDCABLEPART@@}
 static class Smart extends CablePart {AECableType getCableConnectionType(){return AECableType.SMART;}@@SMARTCABLEPART@@}
 static class Dense extends CablePart {AECableType type;Dense(AECableType t){type=t;}AECableType getCableConnectionType(){return type;}@@DENSECABLEPART@@}
 static class Empty extends CablePart {AECableType getCableConnectionType(){return AECableType.NONE;}void getBoxes(IPartCollisionHelper b,Predicate<@Nullable Direction> f){} }
 static CablePart cable(int t){return switch(t){case 0->new Empty();case 1->new Glass();case 2->new Covered();case 3->new Smart();default->new Dense(AECableType.values()[t]);};}
 static Set<Direction> mask(int mask){var result=EnumSet.noneOf(Direction.class);for(var d:Direction.values())if((mask&(1<<d.ordinal()))!=0)result.add(d);return result;}
 static int mask(Set<Direction> set){int result=0;for(var d:set)result|=1<<d.ordinal();return result;}
 static class Token implements ICablePart,IFacadePart {int id;Token(int id){this.id=id;}}
 static class Storage extends CableBusStorage {
  void center(Token t){setCenter(t);}void part(Direction d,Token t){setPart(d,t);}void remove(Direction d){removePart(d);}
  String snapshot()throws Exception {
   var a=CableBusStorage.class.getDeclaredField("parts");a.setAccessible(true);var b=CableBusStorage.class.getDeclaredField("facades");b.setAccessible(true);
   var s=new StringBuilder();s.append(a.get(this)!=null?1:0).append(' ').append(b.get(this)!=null?1:0).append(' ').append(id(getCenter()));
   for(var d:Direction.values())s.append(' ').append(id(getPart(d)));for(var d:Direction.values())s.append(' ').append(id(getFacade(d)));return s.toString();
  }
 }
 static int id(Object value){return value==null?0:((Token)value).id;}
 static int next(StringTokenizer s){return Integer.parseInt(s.nextToken());}
 static Node node(int max,int mode){var n=new Node();n.max=max;n.mode=mode==1?ChannelMode.INFINITE:ChannelMode.DEFAULT;return n;}
 public static void main(String[] args)throws Exception {
  var reader=new BufferedReader(new InputStreamReader(System.in));String line;var storage=new Storage();
  while((line=reader.readLine())!=null){var s=new StringTokenizer(line);var op=s.nextToken();
   switch(op){
    case "T" -> {int a=next(s),b=next(s);var x=AECableType.values()[a];var y=AECableType.values()[b];System.out.println(AECableType.min(x,y).ordinal()+" "+AECableType.max(x,y).ordinal()+" "+(x.isDense()?1:0)+" "+(x.isSmart()?1:0)+" "+(x.isValid()?1:0)+" "+x.variant().ordinal()+" "+x.size().ordinal()+" "+(int)cable(a).getCableConnectionLength(y));}
    case "C" -> {var c=AEColor.values()[next(s)];int tint=next(s);var other=AEColor.values()[next(s)];var n=new ColorNode();n.color=c;var o=new Node();o.color=other;System.out.println(c.registryPrefix+" "+c.blackVariant+" "+c.mediumVariant+" "+c.whiteVariant+" "+c.contrastTextColor+" "+c.getVariantByTintIndex(tint)+" "+(n.hasCompatibleColor(o)?1:0));}
    case "V" -> {var c=cable(next(s));int channels=next(s),max=next(s),mode=next(s),present=next(s);c.node=present==0?null:node(max,mode);System.out.println(c.getVisualChannels(channels));}
    case "B" -> {var c=cable(next(s));c.connections=mask(next(s));c.dense=next(s);var boxes=new Boxes();boxes.bb=next(s)!=0;int filter=next(s);for(int i=0;i<6;i++){String v=s.nextToken();if(!v.equals("none")){var p=new Part();p.length=Double.valueOf(v);c.host.parts[i]=p;}}c.getBoxes(boxes,d->(filter&(1<<(d==null?6:d.ordinal())))!=0);var out=new StringBuilder().append(boxes.calls.size());for(var box:boxes.calls)for(double v:box)out.append(' ').append(Long.toUnsignedString(Double.doubleToLongBits(v),16));System.out.println(out);}
    case "P" -> {var c=cable(next(s));int conn=next(s),parts=next(s),max=next(s),mode=next(s),present=next(s);c.node=present==0?null:node(max,mode);int[] pc=new int[6],ec=new int[6];for(int i=0;i<6;i++)pc[i]=next(s);for(int i=0;i<6;i++)ec[i]=next(s);for(var d:Direction.values()){int i=d.ordinal();if((parts&(1<<i))!=0){var p=new Part();p.node=node(max,mode);p.node.connections.add(new Connection(pc[i]));c.host.parts[i]=p;}if(c.node!=null&&(conn&(1<<i))!=0)c.node.external.put(d,new Connection(ec[i]));}
     c.connections=mask(next(s));for(int i=0;i<6;i++)c.channelsOnSide[i]=next(s);c.baseChanged=next(s)!=0;
     var out=new RegistryFriendlyByteBuf();c.writeToStream(out);byte[] bytes=out.out.toByteArray();boolean changed=c.readFromStream(new RegistryFriendlyByteBuf(bytes));var result=new StringBuilder(HexFormat.of().formatHex(bytes)).append(' ').append(changed?1:0).append(' ').append(mask(c.connections));for(int ch:c.channelsOnSide)result.append(' ').append(ch);c.powered=false;result.append(' ').append(c.getChannelsOnSide(Direction.DOWN));System.out.println(result);
    }
    case "S" -> {int action=next(s);var d=Direction.values()[next(s)];int v=next(s);var token=v==0?null:new Token(v);switch(action){case 0->storage=new Storage();case 1->storage.center(token);case 2->storage.part(d,token);case 3->storage.remove(d);case 4->storage.setFacade(d,token);case 5->storage.removeFacade(d);}System.out.println(storage.snapshot());}
    default -> throw new AssertionError(op);
   }
  }
 }
}
'''
    # The compatibility method's original IGridNode interface is represented by
    # the minimal Node stub; the method body remains byte-for-byte unchanged.
    source = source.replace('static class ColorNode {', 'interface IGridNode { AEColor getGridColor(); }\n static class ColorNode {')
    source = source.replace('static class Node {', 'static class Node implements IGridNode {')
    source = source.replace('AEColor getGridColor(){return color;}\n }', 'public AEColor getGridColor(){return color;}\n }')
    for key, value in parts.items():
        source = source.replace('@@' + key.upper() + '@@', value)
    (src / 'CableOracle.java').write_text(source)
    source_files = files + ['parts/networking/' + name + '.java' for name in texts] + ['me/InWorldGridNode.java']
    return src, parts, source_files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home', type=Path, default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    parser.add_argument('--cases', type=int, default=20000)
    args = parser.parse_args()
    if args.cases < 1024:
        parser.error('at least 1024 random cases')
    upstream.verify(upstream.SOURCE)
    src, parts, source_files = generate()
    binary = lambda name: str(args.java_home / 'bin' / name) if args.java_home else name
    classes = WORK / 'classes'
    classes.mkdir(exist_ok=True)
    subprocess.run([binary('javac'), '--release', '21', '-d', str(classes), *map(str, sorted(src.rglob('*.java')))], check=True)
    rows = []
    for a in range(6):
        for b in range(6):
            rows.append(f'T {a} {b}')
    for color in range(17):
        for other in range(17):
            for tint in [-1, 0, 1, 2, 3, 4, 5, 2147483647]:
                rows.append(f'C {color} {tint} {other}')
    # All direction masks and filters, both dense box modes, all cable types.
    for kind in range(6):
        for conn in range(64):
            for filter_mask in range(128):
                for dense_mask in [0, 63]:
                    for bb_collision in [0, 1]:
                        rows.append(f'B {kind} {conn} {dense_mask} {bb_collision} {filter_mask} 4 8 0 -1 9 NaN')
    rng = random.Random(0xAE2CAB1E)
    edges = [-2147483648, -2147483647, -1024, -32, -1, 0, 1, 7, 8, 9, 31, 32, 33, 128, 2147483647]
    for i in range(args.cases):
        kind, conn, mask = rng.randrange(6), rng.randrange(64), rng.randrange(64)
        # Negative grid maxima are included to check Java overflow/narrowing;
        # zero is handled before division. MIN/-1 uses Java's wrapping quotient.
        rows.append(f'V {kind} {rng.choice(edges)} {rng.choice(edges)} {rng.randrange(2)} {rng.randrange(2)}')
        pc = [rng.choice([0, 1, 8, 32, 128, 2147483647]) for _ in range(6)]
        ec = [rng.choice(edges) for _ in range(6)]
        old = [rng.randrange(256) for _ in range(6)]
        rows.append(' '.join(map(str, ['P', kind, conn, mask, rng.choice(edges), rng.randrange(2), rng.randrange(2), *pc, *ec, rng.randrange(64), *old, rng.randrange(2)])))
        rows.append(f'S {rng.randrange(6)} {rng.randrange(6)} {rng.randrange(8)}')
        lengths = [rng.choice(['none', '-Infinity', '-0.0', '0.00001', '4.5', '8', '8.00001', 'Infinity', 'NaN']) for _ in range(6)]
        rows.append(' '.join(map(str, ['B', kind, conn, rng.randrange(64), rng.randrange(2), rng.randrange(128), *lengths])))
    data = ('\n'.join(rows) + '\n').encode()
    (WORK / 'input.txt').write_bytes(data)
    reference = subprocess.run([binary('java'), '-cp', str(classes), 'oracle.CableOracle'], input=data, capture_output=True, check=True)
    port = subprocess.run(['cargo', 'run', '--locked', '--quiet', '--example', 'cable_oracle'], cwd=ROOT, input=data, capture_output=True, check=True)
    (WORK / 'java.txt').write_bytes(reference.stdout)
    (WORK / 'rust.txt').write_bytes(port.stdout)
    a, b = reference.stdout.splitlines(), port.stdout.splitlines()
    if a != b:
        for i, (x, y) in enumerate(zip(a, b)):
            if x != y:
                raise AssertionError(f'case {i}: {rows[i]} Java={x!r} Rust={y!r}')
        raise AssertionError(f'output lengths differ: {len(a)} / {len(b)}')
    report = {'passed': True, 'cases': len(rows), 'operations': {op: sum(row.startswith(op + ' ') for row in rows) for op in ['T', 'C', 'V', 'B', 'P', 'S']},
              'revision': upstream.LOCK['revision'], 'input_sha256': hashlib.sha256(data).hexdigest(),
              'source_sha256': {p: hashlib.sha256((BASE / p).read_bytes()).hexdigest() for p in source_files},
              'verbatim_fragment_sha256': {p: hashlib.sha256(v.encode()).hexdigest() for p, v in parts.items()},
              'scope': 'unchanged AECableType/Variant/Size/AEColor and CableBusStorage; verbatim CablePart geometry/connection-length/channel payload methods, subtype geometry and InWorldGridNode color compatibility in stub hosts',
              'limits': ['Minecraft/NeoForge world, codecs/registries, renderer and full multipart host are not executed',
                         'AEBasePart packet prefix is a no-op stub; only CablePart suffix bytes are compared',
                         'grid channels/power/neighbour cable types and side-part lengths are injected, not calculated by Java grid services',
                         'raw addBox calls, not normalized VoxelShapes; updateConnections is a no-op stub',
                         'negative channel counts on part connections are excluded because Java takes max with zero; external connections include signed/overflow cases',
                         'truncated packet transactional rejection is a stricter Rust gate and is not claimed equivalent to partial Java reader mutation']}
    (ROOT / 'docs/cable-oracle.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: report[k] for k in ['passed', 'cases', 'operations']}, indent=2))


if __name__ == '__main__':
    main()
