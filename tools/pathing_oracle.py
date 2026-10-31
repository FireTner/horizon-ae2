#!/usr/bin/env python3
"""Compare Rust with ORIGINAL AE2 PathingCalculation + extracted path methods.

This is a rule-level Java oracle, not a running NeoForge server. Only game,
logging and collection infrastructure is stubbed. No routing/counting method is
rewritten in Java: they are copied verbatim from the pinned reference checkout.
Requires JDK 21+, --java-home or JAVA_HOME. Generated Java stays under target/.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import subprocess

import upstream

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "target/upstream/ae2/src/main/java"
WORK = ROOT / "target/pathing-oracle"


def method(source, signature):
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 1
    end = brace + 1
    while depth:
        if source[end] == "{":
            depth += 1
        elif source[end] == "}":
            depth -= 1
        end += 1
    return source[start:end]


def generate():
    files = {}
    for name in ["appeng/api/networking/GridFlags.java", "appeng/api/networking/pathing/ChannelMode.java",
                 "appeng/me/pathfinding/IPathItem.java", "appeng/me/pathfinding/PathingCalculation.java"]:
        files[name] = (SOURCE / name).read_text()
    node_methods = ["public IPathItem getControllerRoute()", "public @Nullable GridNode getHighestSimilarAncestor()",
                    "public boolean getSubtreeAllowsCompressedChannels()", "public void setControllerRoute(IPathItem fast)",
                    "public int getUsedChannels()", "public int getMaxChannels()", "public Iterable<IPathItem> getPossibleOptions()",
                    "public int propagateChannelsUpwards(boolean consumesChannel)", "public void incrementChannelCount(int usedChannels)",
                    "public void finalizeChannels()", "public void setAdHocChannels(int channels)"]
    connection_methods = ["public IGridNode getOtherSide(IGridNode gridNode)", "public GridNode a()", "public GridNode b()",
                          "public int getUsedChannels()", "public void setAdHocChannels(int channels)",
                          "public GridNode getControllerRoute()", "public void setControllerRoute(IPathItem fast)",
                          "public int getMaxChannels()", "public Iterable<IPathItem> getPossibleOptions()",
                          "public boolean hasFlag(GridFlags flag)", "public int propagateChannelsUpwards()", "public void finalizeChannels()"]
    imports = """import java.util.*; import org.jetbrains.annotations.Nullable;
import appeng.api.networking.*; import appeng.api.networking.pathing.*;
import appeng.me.pathfinding.*; import appeng.blockentity.networking.ControllerBlockEntity;
import com.google.common.collect.ImmutableList;
"""
    node_source = (SOURCE / "appeng/me/GridNode.java").read_text()
    files["appeng/me/GridNode.java"] = "package appeng.me;\n" + imports + """
public class GridNode implements IGridNode, IPathItem {
 public static final org.slf4j.Logger LOG=org.slf4j.LoggerFactory.getLogger(GridNode.class);
 public ArrayList<GridConnection> connections=new ArrayList<>();
 public EnumSet<GridFlags> flags=EnumSet.noneOf(GridFlags.class);
 public Object owner; public StubGrid myGrid; public int usedChannels=0;
 private int lastUsedChannels=0; private GridNode highestSimilarAncestor=null;
 private int subtreeMaxChannels; private boolean subtreeAllowsCompressedChannels;
 public List<IGridNode> multi=new ArrayList<>();
 public Object getOwner(){return owner;} public StubGrid getGrid(){return myGrid;}
 public StubGrid getInternalGrid(){return myGrid;} public boolean hasFlag(GridFlags flag){return flags.contains(flag);}
 public List<IGridConnection> getConnections(){return new ArrayList<IGridConnection>(connections);}
 public <T> T getService(Class<T> cls){return multi.isEmpty()?null:cls.cast((IGridMultiblock)()->multi.iterator());}
 public void notifyStatusChange(IGridNodeListener.State reason){}
""" + "\n".join(method(node_source, sig) for sig in node_methods) + "\n}"
    connection_source = (SOURCE / "appeng/me/GridConnection.java").read_text()
    files["appeng/me/GridConnection.java"] = "package appeng.me;\n" + imports + """
import net.minecraft.core.Direction;
public class GridConnection implements IGridConnection, IPathItem {
 public int usedChannels=0; private int lastUsedChannels=0;
 private GridNode sideA,sideB; private Direction fromAtoB;
 public GridConnection(GridNode a,GridNode b){sideA=a;sideB=b; a.connections.add(this);b.connections.add(this);}
""" + "\n".join(method(connection_source, sig) for sig in connection_methods) + "\n}"
    files.update({
        "org/jetbrains/annotations/Nullable.java": "package org.jetbrains.annotations; public @interface Nullable {}",
        "org/slf4j/Logger.java": "package org.slf4j; public class Logger {public void error(String msg,Object... args){} }",
        "org/slf4j/LoggerFactory.java": "package org.slf4j; public class LoggerFactory {public static Logger getLogger(Class<?> c){return new Logger();}}",
        "com/google/common/collect/ImmutableList.java": "package com.google.common.collect; public class ImmutableList {public static <T> java.util.List<T> copyOf(java.util.Collection<? extends T> v){return java.util.List.copyOf(v);} @SafeVarargs public static <T> java.util.List<T> of(T... v){return java.util.List.of(v);} }",
        "net/minecraft/core/Direction.java": "package net.minecraft.core; public enum Direction {DOWN,UP,NORTH,SOUTH,WEST,EAST;public Direction getOpposite(){return values()[ordinal()^1];}}",
        "it/unimi/dsi/fastutil/objects/Reference2IntOpenHashMap.java": "package it.unimi.dsi.fastutil.objects; public class Reference2IntOpenHashMap<T> extends java.util.IdentityHashMap<T,Integer>{public void addTo(T k,int amount){put(k,getOrDefault(k,0)+amount);} }",
        "appeng/api/networking/IGridConnection.java": "package appeng.api.networking; public interface IGridConnection {IGridNode getOtherSide(IGridNode node);}",
        "appeng/api/networking/IGridNode.java": "package appeng.api.networking; public interface IGridNode {Object getOwner(); java.util.List<IGridConnection> getConnections(); <T> T getService(Class<T> cls);}",
        "appeng/api/networking/IGridMultiblock.java": "package appeng.api.networking; public interface IGridMultiblock {java.util.Iterator<IGridNode> getMultiblockNodes();}",
        "appeng/api/networking/IGridNodeListener.java": "package appeng.api.networking; public interface IGridNodeListener {enum State {CHANNEL} }",
        "appeng/api/networking/IGrid.java": "package appeng.api.networking; public interface IGrid {Iterable<IGridNode> getMachineNodes(Class<?> ownerType);}",
        "appeng/blockentity/networking/ControllerBlockEntity.java": "package appeng.blockentity.networking; public class ControllerBlockEntity {}",
        "appeng/me/StubGrid.java": """package appeng.me;
import appeng.api.networking.*; import appeng.api.networking.pathing.*;
public class StubGrid implements IGrid {
 public java.util.List<GridNode> nodes=new java.util.ArrayList<>(); public ChannelMode mode;
 public StubGrid getPathingService(){return this;} public ChannelMode getChannelMode(){return mode;}
 public Iterable<IGridNode> getMachineNodes(Class<?> cls){return nodes.stream().filter(n->cls.isInstance(n.owner)).map(n->(IGridNode)n).toList();}
} """,
        "Main.java": """import java.util.*;
import appeng.me.*;import appeng.me.pathfinding.*;import appeng.api.networking.*;import appeng.api.networking.pathing.*;
import appeng.blockentity.networking.ControllerBlockEntity;
public class Main {
 public static void main(String[] args){Scanner in=new Scanner(System.in);int tests=in.nextInt();
 for(int t=0;t<tests;t++){
  StubGrid grid=new StubGrid();grid.mode=ChannelMode.values()[in.nextInt()];int n=in.nextInt(),m=in.nextInt();
  List<int[]> multis=new ArrayList<>();
  for(int i=0;i<n;i++){GridNode node=new GridNode();node.myGrid=grid;int bits=in.nextInt();
   for(var f:GridFlags.values())if((bits&(1<<f.ordinal()))!=0)node.flags.add(f);
   node.owner=in.nextInt()!=0?new ControllerBlockEntity():new Object();for(int axis=0;axis<3;axis++)in.nextInt();
   int k=in.nextInt();int[] multi=new int[k];for(int j=0;j<k;j++)multi[j]=in.nextInt();multis.add(multi);grid.nodes.add(node);
  }
  for(int i=0;i<n;i++)for(int id:multis.get(i))grid.nodes.get(i).multi.add(grid.nodes.get(id));
  List<GridConnection> edges=new ArrayList<>();
  for(int i=0;i<m;i++){var edge=new GridConnection(grid.nodes.get(in.nextInt()),grid.nodes.get(in.nextInt()));in.nextInt();edges.add(edge);}
  PathingCalculation calc=new PathingCalculation(grid);calc.compute();
  StringBuilder out=new StringBuilder();out.append(calc.getChannelsInUse()).append(' ').append(calc.getChannelsByBlocks());
  for(var node:grid.nodes){node.finalizeChannels();out.append(' ').append(node.getUsedChannels());}
  for(var edge:edges){edge.finalizeChannels();out.append(' ').append(edge.getUsedChannels());}System.out.println(out);
 }
 }
} """,
    })
    root = WORK / "java"
    for path, content in files.items():
        dest = root / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(content)
    return sorted(root.rglob("*.java"))


def cases(count):
    rng = random.Random(0xAE219218)
    result = []
    for _ in range(count):
        n = rng.randrange(2, 85)
        nodes = [(0, True, [])]
        for i in range(1, n):
            flags = rng.choice([0, 1, 1, 1, 4, 8, 16, 16, 64, 65, 3, 24])
            nodes.append((flags, False, []))
        if n > 8 and rng.random() < 0.6:
            group = rng.sample(range(1,n), rng.randrange(2,6))
            for id in group:
                nodes[id] = (nodes[id][0] | 1 | 32, False, group)
        edges = [(rng.randrange(i), i, -1) for i in range(1, n)]
        pairs = {tuple(sorted((a,b))) for a,b,_ in edges}
        for _ in range(n // 2):
            a,b = rng.sample(range(n),2)
            if tuple(sorted((a,b))) not in pairs:
                pairs.add(tuple(sorted((a,b))));edges.append((a,b,-1))
        result.append((rng.randrange(5), nodes, edges))
    lines = [str(len(result))]
    for mode,nodes,edges in result:
        lines.append(f"{mode} {len(nodes)} {len(edges)}")
        for flags,controller,multi in nodes:
            lines.append(" ".join(map(str,[flags,int(controller),0,0,0,len(multi),*multi])))
        lines.extend(f"{a} {b} {direction}" for a,b,direction in edges)
    return "\n".join(lines)+"\n"


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home",type=Path,default=Path(os.environ.get("JAVA_HOME","/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home")))
    parser.add_argument("--cases",type=int,default=500)
    args=parser.parse_args()
    upstream.verify(ROOT/"target/upstream/ae2")
    sources=generate();classes=WORK/"classes";classes.mkdir(exist_ok=True)
    subprocess.run([str(args.java_home/"bin/javac"),"-d",str(classes),*map(str,sources)],check=True)
    input_data=cases(args.cases);(WORK/"inputs.txt").write_text(input_data)
    reference=subprocess.run([str(args.java_home/"bin/java"),"-cp",str(classes),"Main"],input=input_data,text=True,capture_output=True,check=True).stdout
    subprocess.run(["cargo","build","--locked","--example","grid_oracle"],cwd=ROOT,check=True)
    actual=subprocess.run([str(ROOT/"target/debug/examples/grid_oracle")],input=input_data,text=True,capture_output=True,check=True).stdout
    (WORK/"java.txt").write_text(reference);(WORK/"rust.txt").write_text(actual)
    java_lines=reference.splitlines();rust_lines=actual.splitlines()
    failures=[i for i,(a,b) in enumerate(zip(java_lines,rust_lines)) if a!=b]
    if len(java_lines)!=args.cases or len(rust_lines)!=args.cases:raise ValueError("oracle output line count mismatch")
    report={"passed":not failures,"cases":args.cases,"failures":failures,"upstream":upstream.LOCK["revision"],
            "scope":"original PathingCalculation and extracted GridNode/GridConnection path methods; runtime services stubbed; controller-online graphs only",
            "input_sha256":hashlib.sha256(input_data.encode()).hexdigest()}
    (WORK/"report.json").write_text(json.dumps(report,indent=2)+"\n");print(json.dumps(report,indent=2))
    if failures:raise SystemExit(1)


if __name__=="__main__":main()
