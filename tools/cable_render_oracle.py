#!/usr/bin/env python3
"""Compare original cable builders and verbatim cable-only model composition.

Sprites use a controlled affine atlas; packed vanilla vertices, AO, GPU,
attachment models/facades and world render-state collection are not executed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import shutil
import struct
import subprocess

import upstream
import cable_oracle
from controller_oracle import fragment

ROOT = upstream.ROOT
WORK = ROOT / 'target/cable-render-oracle'
BASE = upstream.SOURCE / 'src/main/java/appeng'
FILES = ['client/render/cablebus/' + n + '.java' for n in ['CubeBuilder', 'CableBuilder', 'CableCoreType', 'SmartCableTextures']]
MODEL = 'client/render/cablebus/CableBusBakedModel.java'
EMITTER = 'thirdparty/fabric/QuadEmitter.java'


def generate():
    api, _, _ = cable_oracle.generate()
    src = WORK / 'src'
    shutil.copytree(api, src, dirs_exist_ok=True, ignore=shutil.ignore_patterns('CableOracle.java'))
    for name in FILES:
        dest = src / 'appeng' / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(BASE / name, dest)
    square = fragment((BASE / EMITTER).read_text(), 'default QuadEmitter square(')
    model = (BASE / MODEL).read_text()
    straight = fragment(model, 'private static boolean isStraightLine(')
    compose = fragment(model, 'private void addCableQuads(')
    stubs = {
        'com/google/common/base/Preconditions.java': 'package com.google.common.base; public class Preconditions {public static void checkArgument(boolean b,String s){if(!b)throw new IllegalArgumentException(s);}}',
        'com/google/common/collect/ImmutableMap.java': 'package com.google.common.collect; public class ImmutableMap {public static <K,V> java.util.Map<K,V> copyOf(java.util.Map<K,V> m){return java.util.Map.copyOf(m);}}',
        'org/joml/Vector4f.java': 'package org.joml; public record Vector4f(float x,float y,float z,float w) {}',
        'net/minecraft/core/Direction.java': '''package net.minecraft.core; public enum Direction {
 DOWN(0,-1,0,Axis.Y),UP(0,1,0,Axis.Y),NORTH(0,0,-1,Axis.Z),SOUTH(0,0,1,Axis.Z),WEST(-1,0,0,Axis.X),EAST(1,0,0,Axis.X);
 public enum Axis {X,Y,Z} final int x,y,z;final Axis axis;Direction(int x,int y,int z,Axis a){this.x=x;this.y=y;this.z=z;axis=a;}
 public Direction getOpposite(){return values()[ordinal()^1];}public Axis getAxis(){return axis;}public int getStepX(){return x;}public int getStepY(){return y;}public int getStepZ(){return z;}
}''',
        'net/minecraft/resources/ResourceLocation.java': 'package net.minecraft.resources; public record ResourceLocation(String value) {public String toString(){return value;}}',
        'appeng/core/AppEng.java': 'package appeng.core; public class AppEng {public static net.minecraft.resources.ResourceLocation makeId(String s){return new net.minecraft.resources.ResourceLocation("ae2:"+s);}}',
        'net/minecraft/client/renderer/texture/TextureAtlas.java': 'package net.minecraft.client.renderer.texture; public class TextureAtlas {public static final Object LOCATION_BLOCKS=new Object();}',
        'net/minecraft/client/resources/model/Material.java': 'package net.minecraft.client.resources.model; public record Material(Object atlasLocation,net.minecraft.resources.ResourceLocation texture) {}',
        'net/minecraft/client/renderer/texture/TextureAtlasSprite.java': '''package net.minecraft.client.renderer.texture; public class TextureAtlasSprite {
 public final String id;final float[] rect;public TextureAtlasSprite(String id,float[] r){this.id=id;rect=r;}
 public float getU(float v){return rect[0]+(rect[2]-rect[0])*v;}public float getV(float v){return rect[1]+(rect[3]-rect[1])*v;}}
''',
        'net/minecraft/client/renderer/LightTexture.java': 'package net.minecraft.client.renderer; public class LightTexture {public static int pack(int b,int s){return (b<<4)|(s<<20);}}',
        'net/minecraft/client/renderer/block/model/BakedQuad.java': '''package net.minecraft.client.renderer.block.model; import net.minecraft.core.Direction;
 public record BakedQuad(String texture,Direction face,Direction cull,float[][] positions,float[][] uv,int color,boolean emissive){public int getTintIndex(){return -1;}}
''',
        'appeng/thirdparty/fabric/EncodingFormat.java': 'package appeng.thirdparty.fabric; public class EncodingFormat {public static final int TOTAL_STRIDE=32;}',
        'appeng/thirdparty/fabric/QuadEmitter.java': '''package appeng.thirdparty.fabric;import net.minecraft.core.Direction;
public interface QuadEmitter {float CULL_FACE_EPSILON=0.00001f;QuadEmitter cullFace(Direction f);QuadEmitter nominalFace(Direction f);QuadEmitter pos(int v,float x,float y,float z);QuadEmitter emit();void uv(int i,float u,float v);
@@SQUARE@@
}'''.replace('@@SQUARE@@', square),
        'appeng/thirdparty/fabric/MutableQuadViewImpl.java': '''package appeng.thirdparty.fabric;
import net.minecraft.core.Direction;import net.minecraft.client.renderer.texture.TextureAtlasSprite;import net.minecraft.client.renderer.block.model.BakedQuad;
public abstract class MutableQuadViewImpl implements QuadEmitter {
 float[][] p=new float[4][3],uv=new float[4][2];int color;Direction face,cull;boolean bright;
 protected void begin(int[] data,int index){}public void colorIndex(int index){if(index!=-1)throw new AssertionError();}
 public void color(int a,int b,int c,int d){if(a!=b||b!=c||c!=d)throw new AssertionError();color=a;}
 public void normal(int i,float x,float y,float z){}public void uv(int i,float u,float v){uv[i][0]=u;uv[i][1]=v;}
 public QuadEmitter cullFace(Direction d){cull=d;return this;}public QuadEmitter nominalFace(Direction d){face=d;return this;}
 public QuadEmitter pos(int i,float x,float y,float z){p[i]=new float[]{x,y,z};return this;}
 public void lightmap(int a,int b,int c,int d){if(a!=15728880||a!=b||b!=c||c!=d)throw new AssertionError();bright=true;}
 public BakedQuad toBakedQuad(TextureAtlasSprite t){return new BakedQuad(t.id,face,cull,p,uv,color,bright);}
}''',
    }
    for name, body in stubs.items():
        dest = src / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(body + '\n')
    source = r'''
package appeng.client.render.cablebus;
import java.io.*;import java.util.*;import java.util.Map.Entry;
import net.minecraft.core.Direction;import appeng.api.util.*;
import net.minecraft.client.renderer.block.model.BakedQuad;
import net.minecraft.client.renderer.texture.TextureAtlasSprite;
public class RenderOracle {
 interface IPartModel {boolean requireCableConnection();}
 static class CableBusRenderState {
  AECableType type;AEColor color;CableCoreType core;boolean require;Set<Direction> adjacent=EnumSet.noneOf(Direction.class);
  EnumMap<Direction,AECableType> connections=new EnumMap<>(Direction.class);EnumMap<Direction,Integer> channels=new EnumMap<>(Direction.class),attachments=new EnumMap<>(Direction.class);
  AECableType getCableType(){return type;}AEColor getCableColor(){return color;}CableCoreType getCoreType(){return core;}
  EnumMap<Direction,AECableType> getConnectionTypes(){return connections;}EnumMap<Direction,Integer> getChannelsOnSide(){return channels;}
  EnumMap<Direction,Integer> getAttachmentConnections(){return attachments;}Set<Direction> getCableBusAdjacent(){return adjacent;}
  Map<Direction,IPartModel> getAttachments(){return require?Map.of(Direction.DOWN,()->true):Map.of();}
 }
 CableBuilder cableBuilder;RenderOracle(CableBuilder b){cableBuilder=b;}
 @@STRAIGHT@@
 @@COMPOSE@@
 static int n(StringTokenizer s){return Integer.parseInt(s.nextToken());}
 static float f(StringTokenizer s){return Float.intBitsToFloat(Integer.parseUnsignedInt(s.nextToken(),16));}
 static float[] rect(StringTokenizer s){return new float[]{f(s),f(s),f(s),f(s)};}
 static void piece(CableBuilder b,AECableType kind,AEColor color,Direction face,int op,AECableType other,boolean adj,int ch,int dist,CableCoreType core,List<BakedQuad> out){
  if(op==0){b.addCableCore(core,color,out);return;}
  if(kind==AECableType.NONE)return;
  if(op==1){switch(kind){case GLASS->b.addGlassConnection(face,color,other,adj,out);case COVERED->b.addCoveredConnection(face,color,other,adj,out);case SMART->b.addSmartConnection(face,color,other,adj,ch,out);case DENSE_COVERED->b.addDenseCoveredConnection(face,color,other,adj,out);case DENSE_SMART->b.addDenseSmartConnection(face,color,other,adj,ch,out);}}
  if(op==2){switch(kind){case GLASS->b.addStraightGlassConnection(face,color,out);case COVERED->b.addStraightCoveredConnection(face,color,out);case SMART->b.addStraightSmartConnection(face,color,ch,out);case DENSE_COVERED->b.addStraightDenseCoveredConnection(face,color,out);case DENSE_SMART->b.addStraightDenseSmartConnection(face,color,ch,out);}}
  if(op==3){switch(kind){case GLASS->b.addConstrainedGlassConnection(face,color,dist,out);case COVERED->b.addConstrainedCoveredConnection(face,color,dist,out);case SMART->b.addConstrainedSmartConnection(face,color,dist,ch,out);}}
 }
 static void print(List<BakedQuad> quads){var s=new StringBuilder().append(quads.size());for(var q:quads){s.append(' ').append(q.texture()).append(' ').append(q.face().ordinal()).append(' ').append(q.cull()==null?-1:q.cull().ordinal()).append(' ').append(Integer.toUnsignedString(q.color(),16)).append(' ').append(q.emissive()?1:0);for(int i=0;i<4;i++){for(float v:q.positions()[i])s.append(' ').append(Integer.toUnsignedString(Float.floatToIntBits(v),16));for(float v:q.uv()[i])s.append(' ').append(Integer.toUnsignedString(Float.floatToIntBits(v),16));}}System.out.println(s);}
 public static void main(String[] args)throws Exception {var reader=new BufferedReader(new InputStreamReader(System.in));String line;
  while((line=reader.readLine())!=null){var s=new StringTokenizer(line);String op=s.nextToken();List<BakedQuad> out=new ArrayList<>();
   if(op.equals("C")){int faces=n(s),fu=n(s),fv=n(s),rot=n(s),custom=n(s);boolean bright=n(s)!=0;int color=Integer.parseUnsignedInt(s.nextToken(),16),mode=n(s),face=n(s);float[] bounds=new float[6];for(int i=0;i<6;i++)bounds[i]=f(s);float[] uv=rect(s),atlas=rect(s);var cube=new CubeBuilder(out);var draw=EnumSet.noneOf(Direction.class);for(var d:Direction.values()){int i=d.ordinal();cube.setTexture(d,new TextureAtlasSprite("probe/"+i,atlas));if((faces&(1<<i))!=0)draw.add(d);cube.setFlipU(d,(fu&(1<<i))!=0);cube.setFlipV(d,(fv&(1<<i))!=0);cube.setUvRotation(d,(rot>>(i*2))&3);if((custom&(1<<i))!=0)cube.setCustomUv(d,uv[0],uv[1],uv[2],uv[3]);}cube.setDrawFaces(draw);cube.setColor(color);cube.setEmissiveMaterial(bright);if(mode==0)cube.addCube(bounds[0],bounds[1],bounds[2],bounds[3],bounds[4],bounds[5]);else cube.addQuad(Direction.values()[face],bounds[0],bounds[1],bounds[2],bounds[3],bounds[4],bounds[5]);}
   else if(op.equals("P")){var kind=AECableType.values()[n(s)];var color=AEColor.values()[n(s)];var face=Direction.values()[n(s)];int action=n(s);var other=AECableType.values()[n(s)];boolean adjacent=n(s)!=0;int channels=n(s),distance=n(s);var core=CableCoreType.values()[n(s)];float[] atlas=rect(s);var builder=new CableBuilder(m->new TextureAtlasSprite(m.texture().toString(),atlas));piece(builder,kind,color,face,action,other,adjacent,channels,distance,core,out);}
   else if(op.equals("R")){var state=new CableBusRenderState();state.type=AECableType.values()[n(s)];state.color=AEColor.values()[n(s)];state.core=CableCoreType.values()[n(s)];state.require=n(s)!=0;int adjacent=n(s);for(var d:Direction.values())state.channels.put(d,n(s));for(var d:Direction.values()){int type=n(s);if(type>=0)state.connections.put(d,AECableType.values()[type]);if((adjacent&(1<<d.ordinal()))!=0)state.adjacent.add(d);}for(var d:Direction.values()){int dist=n(s);if(dist!=Integer.MIN_VALUE)state.attachments.put(d,dist);}float[] atlas=rect(s);new RenderOracle(new CableBuilder(m->new TextureAtlasSprite(m.texture().toString(),atlas))).addCableQuads(state,out);}
   else throw new AssertionError(op);
   if(s.hasMoreTokens())throw new AssertionError(line);print(out);
  }
 }
}
'''
    source = source.replace('@@STRAIGHT@@', straight).replace('@@COMPOSE@@', compose)
    (src / 'RenderOracle.java').write_text(source)
    return src, {'square': square, 'is_straight': straight, 'compose': compose}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--java-home', type=Path, default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    p.add_argument('--cases', type=int, default=4000)
    args = p.parse_args()
    upstream.verify(upstream.SOURCE)
    src, methods = generate()
    classes = WORK / 'classes'; classes.mkdir(exist_ok=True)
    binary = lambda n: str(args.java_home / 'bin' / n) if args.java_home else n
    subprocess.run([binary('javac'), '--release', '21', '-d', str(classes), *map(str, sorted(src.rglob('*.java')))], check=True)
    rng = random.Random(0xAE2CA8E)
    bits = lambda v: format(struct.unpack('>I', struct.pack('>f', v))[0], 'x')
    atlases = [[0, 0, 1, 1], [.125, .375, .25, .875], [0.376281, .7715, .398012, .79351]]
    rows = []
    def append(values, atlas):
        rows.append(' '.join(map(str, values)) + ' ' + ' '.join(bits(v) for v in atlas))
    # Every cable/color/direction/neighbor and both machine/bus adjacency modes.
    for kind in range(1, 6):
        for color in range(17):
            for face in range(6):
                for other in range(6):
                    for adjacent in range(2):
                        append(['P', kind, color, face, 1, other, adjacent, rng.randrange(-1, 35), 0, 0], rng.choice(atlases))
                append(['P', kind, color, face, 2, 0, 0, rng.choice([-2147483648, -1, 0, 1, 4, 5, 8, 9, 31, 32, 33, 2147483647]), 0, 0], rng.choice(atlases))
                for distance in [-2147483648, -1, 0, 4, 5, 6, 8, 2147483647]:
                    append(['P', kind, color, face, 3, 0, 0, rng.randrange(-1, 35), distance, 0], rng.choice(atlases))
            for core in range(3):
                append(['P', kind, color, 0, 0, 0, 0, 0, 0, core], rng.choice(atlases))
    # All common world connection masks, with actual core type and no attached
    # parts, for every color/type. This includes every straight-line shortcut.
    for kind in range(1, 6):
        for color in range(17):
            for mask in range(64):
                conn = [kind if mask & (1 << side) else -1 for side in range(6)]
                core = 0 if kind == 1 else 1 if kind < 4 else 2
                append(['R', kind, color, core, 0, 63, *[side * 7 for side in range(6)], *conn, *([-2147483648] * 6)], rng.choice(atlases))
    for i in range(args.cases):
        values = [-.01, 0, .000001, .5, 1, 4, 5, 6, 10, 11, 12, 13, 16, 16.01]
        bounds = [rng.choice(values) for _ in range(6)]
        uv = [rng.choice([0, .3, .5, 1, 1.5]) for _ in range(4)]
        append(['C', rng.randrange(64), rng.randrange(64), rng.randrange(64), rng.randrange(4096), rng.randrange(64), rng.randrange(2), format(rng.randrange(1 << 32), 'x'), rng.randrange(2), rng.randrange(6), *map(bits, bounds), *map(bits, uv)], rng.choice(atlases))
        kind, color = rng.randrange(6), rng.randrange(17)
        conn = [rng.randrange(-1, 6) for _ in range(6)]
        if i % 4 == 0:
            axis = rng.randrange(3);conn = [-1] * 6;conn[axis * 2] = conn[axis * 2 + 1] = kind
        attach = [rng.choice([-2147483648, -1, 0, 4, 5, 6, 8]) for _ in range(6)]
        append(['R', kind, color, rng.randrange(3), rng.randrange(2), rng.randrange(64), *[rng.randrange(-1, 35) for _ in range(6)], *conn, *attach], rng.choice(atlases))
    data = ('\n'.join(rows) + '\n').encode(); (WORK / 'input.txt').write_bytes(data)
    with (WORK / 'java.txt').open('wb') as output:
        subprocess.run([binary('java'), '-cp', str(classes), 'appeng.client.render.cablebus.RenderOracle'], input=data, stdout=output, check=True)
    with (WORK / 'rust.txt').open('wb') as output:
        subprocess.run(['cargo', 'run', '--locked', '--quiet', '--example', 'cable_render_oracle'], cwd=ROOT, input=data, stdout=output, check=True)
    counts = {'cases': len(rows), 'quads': 0, 'float_fields': 0}
    with (WORK / 'java.txt').open() as a, (WORK / 'rust.txt').open() as b:
        for i, row in enumerate(rows):
            expected, actual = a.readline(), b.readline()
            if expected != actual:
                raise AssertionError(f'case {i}: {row}\nJava={expected[:500]}\nRust={actual[:500]}')
            n = int(expected.split(' ', 1)[0]);counts['quads'] += n;counts['float_fields'] += n * 20
        assert not a.read() and not b.read(), 'unexpected trailing output'
    sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    report = {'passed': True, **counts, 'operations': {op: sum(r.startswith(op + ' ') for r in rows) for op in ['C', 'P', 'R']},
              'revision': upstream.LOCK['revision'], 'input_sha256': hashlib.sha256(data).hexdigest(),
              'output_sha256': sha(WORK / 'java.txt'), 'source_sha256': {name: sha(BASE / name) for name in [*FILES, MODEL, EMITTER]},
              'verbatim_fragment_sha256': {k: hashlib.sha256(v.encode()).hexdigest() for k, v in methods.items()},
              'scope': 'unchanged CubeBuilder/CableBuilder/CableCoreType/SmartCableTextures; verbatim QuadEmitter.square and CableBusBakedModel cable-only composition; exact float bits, vertex colors/fullbright, sprite IDs, face/cull and quad order',
              'limits': ['sprites use three controlled affine atlas rectangles, not a live Minecraft/Horizon atlas',
                         'MutableQuadViewImpl captures emitter values; vanilla packed vertices/normals/AO/shading conversion is not executed',
                         'cull is emitter metadata, not a claim that returned unculled BakedQuads can be assigned side-culling',
                         'render-state attachment/connection/neighbor/channel inputs are stubbed, not collected from a world',
                         'attachment models, facades, particles, caches, GPU and native renderer integration remain open']}
    (ROOT / 'docs/cable-render-oracle.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({k: report[k] for k in ['passed', 'cases', 'quads', 'float_fields', 'operations']}, indent=2))


if __name__ == '__main__':
    main()
