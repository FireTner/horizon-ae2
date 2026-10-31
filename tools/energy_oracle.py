#!/usr/bin/env python3
"""Differential traces against unmodified pinned AE2 energy classes (JDK 21+).

EnergyService, GridEnergyStorage, StoredEnergyAmount and EnergyOverlayGrid are
compiled unchanged. Game/event/collection infrastructure and finite power-store
hosts are stubs; the hosts delegate amount operations to original StoredEnergyAmount.
This does not launch Minecraft, test native lifecycle or prove client parity.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import random
import subprocess

import upstream

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "target/upstream/ae2/src/main/java"
WORK = ROOT / "target/energy-oracle"
ORIGINALS = [
    "appeng/me/service/EnergyService.java", "appeng/me/service/EnergyOverlayGrid.java",
    "appeng/me/energy/GridEnergyStorage.java", "appeng/me/energy/StoredEnergyAmount.java",
    "appeng/me/energy/EnergyThreshold.java", "appeng/api/config/AccessRestriction.java",
    "appeng/api/config/PowerMultiplier.java",
]


def generate():
    files = {name: (SOURCE / name).read_text() for name in ORIGINALS}
    files.update({
        "org/jetbrains/annotations/Nullable.java": "package org.jetbrains.annotations; public @interface Nullable {}",
        "org/jetbrains/annotations/VisibleForTesting.java": "package org.jetbrains.annotations; public @interface VisibleForTesting {}",
        "com/google/common/base/Preconditions.java": "package com.google.common.base; public class Preconditions {public static void checkArgument(boolean v,String m){if(!v)throw new IllegalArgumentException(m);} public static void checkState(boolean v,String m){if(!v)throw new IllegalStateException(m);}}",
        "com/google/common/collect/Sets.java": "package com.google.common.collect; public class Sets {public static <T> java.util.NavigableSet<T> newTreeSet(){return new java.util.TreeSet<T>();}}",
        "com/google/common/collect/Multiset.java": "package com.google.common.collect; public interface Multiset<T> extends java.util.Collection<T> {}",
        "com/google/common/collect/HashMultiset.java": "package com.google.common.collect; public class HashMultiset<T> extends java.util.ArrayList<T> implements Multiset<T> {public static <T> Multiset<T> create(){return new HashMultiset<T>();}}",
        "it/unimi/dsi/fastutil/objects/ObjectRBTreeSet.java": "package it.unimi.dsi.fastutil.objects; public class ObjectRBTreeSet<T> extends java.util.TreeSet<T> {public ObjectRBTreeSet(java.util.Comparator<T> c){super(c);}}",
        "it/unimi/dsi/fastutil/objects/ObjectArrayList.java": "package it.unimi.dsi.fastutil.objects; public class ObjectArrayList<T> extends java.util.ArrayList<T> {public T pop(){return remove(size()-1);}}",
        "it/unimi/dsi/fastutil/objects/ReferenceOpenHashSet.java": "package it.unimi.dsi.fastutil.objects; public class ReferenceOpenHashSet<T> extends java.util.AbstractSet<T> {private java.util.Set<T> set=java.util.Collections.newSetFromMap(new java.util.IdentityHashMap<>());public boolean add(T v){return set.add(v);}public int size(){return set.size();}public java.util.Iterator<T> iterator(){return set.iterator();}}",
        "net/minecraft/util/Mth.java": "package net.minecraft.util; public class Mth {public static double clamp(double v,double a,double b){return Math.min(Math.max(v,a),b);}}",
        "net/minecraft/nbt/Tag.java": "package net.minecraft.nbt; public class Tag {public static final int TAG_DOUBLE=6;}",
        "net/minecraft/nbt/CompoundTag.java": "package net.minecraft.nbt; public class CompoundTag {private java.util.Map<String,Double> values=new java.util.HashMap<>();public boolean contains(String k,int t){return t==Tag.TAG_DOUBLE&&values.containsKey(k);}public double getDouble(String k){return values.getOrDefault(k,0.0);}public void putDouble(String k,double v){values.put(k,v);}}",
        "appeng/core/AEConfig.java": "package appeng.core; public class AEConfig {public static AEConfig instance(){return new AEConfig();}public double getGridEnergyStoragePerNode(){return 25;}}",
        "appeng/core/AELog.java": "package appeng.core; public class AELog {public static void error(String msg,Object...args){throw new IllegalStateException(msg);}}",
        "appeng/api/config/Actionable.java": "package appeng.api.config; public enum Actionable {MODULATE,SIMULATE}",
        "appeng/api/networking/IGrid.java": "package appeng.api.networking; public interface IGrid {void postEvent(Object event);}",
        "appeng/api/networking/IGridNode.java": "package appeng.api.networking; public interface IGridNode {<T>T getService(Class<T> cls);double getIdlePowerUsage();}",
        "appeng/api/networking/IGridNodeListener.java": "package appeng.api.networking; public interface IGridNodeListener {enum State {POWER}}",
        "appeng/api/networking/IGridServiceProvider.java": "package appeng.api.networking; public interface IGridServiceProvider {default void onServerStartTick(){}default void onServerEndTick(){}default void addNode(IGridNode n,net.minecraft.nbt.CompoundTag data){}default void removeNode(IGridNode n){}default void saveNodeData(IGridNode n,net.minecraft.nbt.CompoundTag data){}}",
        "appeng/api/networking/GridHelper.java": "package appeng.api.networking; public class GridHelper {public static <E,S>void addGridServiceEventHandler(Class<E> ev,Class<S> service,java.util.function.BiConsumer<S,E> handler){}}",
        "appeng/api/networking/pathing/IPathingService.java": "package appeng.api.networking.pathing; public interface IPathingService {}",
        "appeng/me/service/PathingService.java": "package appeng.me.service; public class PathingService implements appeng.api.networking.pathing.IPathingService {public double channel;public double getChannelPowerUsage(){return channel;}}",
        "appeng/api/networking/energy/IEnergyService.java": """package appeng.api.networking.energy;import appeng.api.config.*;
public interface IEnergyService {double extractAEPower(double a,Actionable m,PowerMultiplier p);double getIdlePowerUsage();double getChannelPowerUsage();double getAvgPowerUsage();double getAvgPowerInjection();boolean isNetworkPowered();double injectPower(double a,Actionable m);double getStoredPower();double getMaxStoredPower();double getEnergyDemand(double a);}""",
        "appeng/api/networking/energy/IAEPowerStorage.java": """package appeng.api.networking.energy;import appeng.api.config.*;
public interface IAEPowerStorage {double injectAEPower(double a,Actionable m);double extractAEPower(double a,Actionable m,PowerMultiplier p);boolean isAEPublicPowerStorage();AccessRestriction getPowerFlow();double getAEMaxPower();double getAECurrentPower();int getPriority();}""",
        "appeng/api/networking/energy/IPassiveEnergyGenerator.java": "package appeng.api.networking.energy; public interface IPassiveEnergyGenerator {double getRate();void setSuppressed(boolean v);}",
        "appeng/api/networking/energy/IEnergyWatcher.java": "package appeng.api.networking.energy; public interface IEnergyWatcher {void reset();}",
        "appeng/api/networking/energy/IEnergyWatcherNode.java": "package appeng.api.networking.energy; public interface IEnergyWatcherNode {void updateWatcher(IEnergyWatcher w);}",
        "appeng/me/energy/EnergyWatcher.java": "package appeng.me.energy; public class EnergyWatcher implements appeng.api.networking.energy.IEnergyWatcher {public EnergyWatcher(appeng.me.service.EnergyService s,appeng.api.networking.energy.IEnergyWatcherNode n){}public void reset(){}public void post(appeng.me.service.EnergyService s){}}",
        "appeng/me/energy/IEnergyOverlayGridConnection.java": "package appeng.me.energy; public interface IEnergyOverlayGridConnection {java.util.Collection<appeng.me.service.EnergyService> connectedEnergyServices();}",
        "appeng/api/networking/events/GridPowerStorageStateChanged.java": "package appeng.api.networking.events; public class GridPowerStorageStateChanged {public enum PowerEventType {PROVIDE_POWER,RECEIVE_POWER} public final appeng.api.networking.energy.IAEPowerStorage storage;public final PowerEventType type;public GridPowerStorageStateChanged(appeng.api.networking.energy.IAEPowerStorage s,PowerEventType t){storage=s;type=t;}}",
        "appeng/api/networking/events/GridPowerIdleChange.java": "package appeng.api.networking.events; public class GridPowerIdleChange {public final appeng.api.networking.IGridNode node;public GridPowerIdleChange(appeng.api.networking.IGridNode n){node=n;}}",
        "appeng/api/networking/events/GridPowerStatusChange.java": "package appeng.api.networking.events; public class GridPowerStatusChange {}",
        "appeng/me/Grid.java": """package appeng.me;import appeng.api.networking.*;import appeng.api.networking.events.*;
public class Grid implements IGrid {public appeng.me.service.EnergyService energy;public void postEvent(Object ev){if(ev instanceof GridPowerStorageStateChanged p)energy.storagePowerChangeHandler(p);if(ev instanceof GridPowerIdleChange i)energy.nodeIdlePowerChangeHandler(i);}public void setImportantFlag(int flag,boolean v){}public void notifyAllNodes(IGridNodeListener.State s){}}""",
        "appeng/me/GridNode.java": """package appeng.me;import appeng.api.networking.*;
public class GridNode implements IGridNode {public double idle,previous;public java.util.Map<Class<?>,Object> services=new java.util.HashMap<>();public <T>T getService(Class<T> cls){return cls.cast(services.get(cls));}public double getIdlePowerUsage(){return idle;}public double getPreviousDraw(){return previous;}public void setPreviousDraw(double v){previous=v;}}""",
        "Main.java": """import java.util.*;import appeng.api.config.*;import appeng.api.networking.energy.*;import appeng.api.networking.events.*;import appeng.me.*;import appeng.me.energy.*;import appeng.me.service.*;import net.minecraft.nbt.*;
public class Main {
 static class State {
  final Grid grid=new Grid(); final PathingService path=new PathingService();
  final EnergyService service=new EnergyService(grid,path);final TreeMap<Integer,GridNode> nodes=new TreeMap<>();
  State(){grid.energy=service;}
 }
 static class Generator implements IPassiveEnergyGenerator {
  final double rate;boolean suppressed;Generator(double rate){this.rate=rate;}
  public double getRate(){return rate;}public void setSuppressed(boolean v){suppressed=v;}
 }
 static class Store implements IAEPowerStorage {
  final StoredEnergyAmount amount;final Grid grid;final int priority;final boolean pub;final AccessRestriction flow;
  Store(Grid g,double a,double max,int p,int f,boolean pub){grid=g;priority=p;flow=AccessRestriction.values()[f];this.pub=pub;amount=new StoredEnergyAmount(a,max,t->g.postEvent(new GridPowerStorageStateChanged(this,t)));}
  public double injectAEPower(double a,Actionable m){return a-amount.insert(a,m==Actionable.MODULATE);}public double extractAEPower(double a,Actionable m,PowerMultiplier p){return amount.extract(a,m==Actionable.MODULATE);}
  public double getAECurrentPower(){return amount.getAmount();}public double getAEMaxPower(){return amount.getMaximum();}public int getPriority(){return priority;}public boolean isAEPublicPowerStorage(){return pub;}public AccessRestriction getPowerFlow(){return flow;}
 }
 static double value(Scanner in){return in.nextDouble();}
 public static void main(String[] args)throws Exception{Locale.setDefault(Locale.ROOT);Scanner in=new Scanner(System.in);int cases=in.nextInt();
 var bufferField=EnergyService.class.getDeclaredField("localStorage");bufferField.setAccessible(true);
 for(int c=0;c<cases;c++){TreeMap<Integer,State> all=new TreeMap<>();all.put(0,new State());int selected=0;
 Map<Integer,Deque<IEnergyOverlayGridConnection>> links=new HashMap<>();int ops=in.nextInt();
 for(int step=0;step<ops;step++){int op=in.nextInt();double result=0;State active=all.get(selected);
 Grid grid=active.grid;PathingService path=active.path;EnergyService service=active.service;TreeMap<Integer,GridNode> nodes=active.nodes;
 switch(op){
 case 0->{int id=in.nextInt();GridNode n=new GridNode();n.idle=value(in);Store store=new Store(grid,value(in),value(in),in.nextInt(),in.nextInt(),in.nextInt()!=0);double saved=value(in);n.services.put(IAEPowerStorage.class,store);nodes.put(id,n);CompoundTag data=new CompoundTag();data.putDouble("e",saved);service.addNode(n,data);}
 case 1->{int id=in.nextInt();GridNode n=nodes.remove(id);CompoundTag data=new CompoundTag();service.saveNodeData(n,data);result=data.getDouble("e");service.removeNode(n);}
 case 2->{double a=value(in);Actionable m=in.nextInt()!=0?Actionable.MODULATE:Actionable.SIMULATE;result=service.injectPower(a,m);}
 case 3->{double a=value(in);Actionable m=in.nextInt()!=0?Actionable.MODULATE:Actionable.SIMULATE;PowerMultiplier.CONFIG.multiplier=value(in);result=service.extractAEPower(a,m,PowerMultiplier.CONFIG);}
 case 4->{GridNode n=nodes.get(in.nextInt());((Store)n.getService(IAEPowerStorage.class)).amount.setStored(value(in));}
 case 5->{GridNode n=nodes.get(in.nextInt());n.idle=value(in);service.nodeIdlePowerChangeHandler(new GridPowerIdleChange(n));}
 case 6->{int ticks=in.nextInt();path.channel=value(in);PowerMultiplier.CONFIG.multiplier=value(in);for(int t=0;t<ticks;t++){for(State s:all.values())s.service.onServerStartTick();service.onServerEndTick();}}
 case 7->service.refreshPower();case 8->result=service.getEnergyDemand(value(in));
 case 9->{selected=in.nextInt();all.computeIfAbsent(selected,k->new State());}
 case 10->{int a=in.nextInt(),b=in.nextInt();EnergyService sa=all.get(a).service,sb=all.get(b).service;IEnergyOverlayGridConnection link=()->List.of(sa,sb);links.computeIfAbsent(Math.min(a,b)*100+Math.max(a,b),k->new ArrayDeque<>()).push(link);sa.getOverlayGridConnections().add(link);sb.getOverlayGridConnections().add(link);sa.invalidateOverlayEnergyGrid();sb.invalidateOverlayEnergyGrid();}
 case 11->{int a=in.nextInt(),b=in.nextInt();EnergyService sa=all.get(a).service,sb=all.get(b).service;var link=links.get(Math.min(a,b)*100+Math.max(a,b)).pop();sa.getOverlayGridConnections().remove(link);sb.getOverlayGridConnections().remove(link);sa.invalidateOverlayEnergyGrid();sb.invalidateOverlayEnergyGrid();}
 case 12->{int id=in.nextInt();GridNode n=new GridNode();n.services.put(IPassiveEnergyGenerator.class,new Generator(value(in)));nodes.put(id,n);service.addNode(n,null);}
 case 13->{path.channel=value(in);PowerMultiplier.CONFIG.multiplier=value(in);service.onServerEndTick();}
 default->throw new IllegalArgumentException();
 }
 active=all.get(selected);service=active.service;nodes=active.nodes;
 GridEnergyStorage local=(GridEnergyStorage)bufferField.get(service);
 System.out.print(result+" "+local.getAECurrentPower()+" "+local.getNodeEnergyShare()+" "+service.getMaxStoredPower()+" "+service.getStoredPower()+" "+service.getAvgPowerUsage()+" "+service.getAvgPowerInjection()+" "+(service.isNetworkPowered()?1:0));
 for(GridNode n:nodes.values()){var store=n.getService(IAEPowerStorage.class);System.out.print(" "+(store==null?0:store.getAECurrentPower()));}System.out.println();
 }
 }
 }
}
""",
    })
    root = WORK / "java"
    for name, content in files.items():
        dest = root / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(content)
    return [root / name for name in sorted(files)]


def cases(count):
    rng = random.Random(0xAE219218E)
    traces = []
    for case in range(count):
        ops = []
        nodes = {}
        def add(id):
            maximum = rng.choice([1.0, 100.0, 200000.0, 1600000.0])
            nodes[id] = maximum
            amount = rng.choice([0.0, maximum / 2, maximum])
            # No priority ties: Java identityHashCode is intentionally not reproduced.
            ops.append([0, id, rng.choice([0, .00001, .25, 1, 4]), amount, maximum,
                        id * 17 - 71, rng.randrange(4), int(rng.random() > .2), rng.choice([0, 10, 25])])
        for id in range(rng.randrange(1, 9)):
            add(id)
        for _ in range(200):
            op = rng.choices(range(9), [4, 3, 20, 20, 10, 4, 15, 5, 8])[0]
            if op == 0:
                missing = sorted(set(range(10)) - nodes.keys())
                if missing:
                    add(rng.choice(missing))
                continue
            if op in [1, 4, 5] and not nodes:
                continue
            amount = rng.choice([0, .0000005, .000001, .05, .1, 1, 25, 125, 1600, 2000000])
            if op == 1:
                id = rng.choice(sorted(nodes)); del nodes[id]; ops.append([op, id])
            elif op == 2:
                ops.append([op, amount, rng.randrange(2)])
            elif op == 3:
                ops.append([op, amount, rng.randrange(2), rng.choice([.5, 1, 2, 3.5])])
            elif op == 4:
                id = rng.choice(sorted(nodes)); ops.append([op, id, rng.choice([0, .05, .1, nodes[id]])])
            elif op == 5:
                ops.append([op, rng.choice(sorted(nodes)), rng.choice([0, .00001, .25, 1, 4])])
            elif op == 6:
                ops.append([op, rng.choice([1, 2, 30, 31, 32, 91]), rng.choice([0, .25, 1]), rng.choice([.5, 1, 2])])
            elif op == 7:
                ops.append([op])
            elif op == 8:
                ops.append([op, amount])
        traces.append(ops)
    lines = [str(len(traces))]
    for ops in traces:
        lines.append(str(len(ops)))
        lines.extend(" ".join(map(str, op)) for op in ops)
    return "\n".join(lines) + "\n", sum(map(len, traces))


def overlay_cases(count):
    rng = random.Random(0xAE219218F)
    traces = []
    for _ in range(count):
        ops = []
        generators = {s: set() for s in range(4)}
        links = {}
        current = 0
        for s in range(4):
            ops.append([9, s]); current = s
            # Unique store capacities/priorities and generator rates avoid
            # identity hash ties in both original sorted sets and overlay sort.
            for i in range(3):
                maximum = (s * 3 + i + 1) * 1000
                ops.append([0, i, rng.choice([0, .25, 1]), rng.choice([0, maximum]),
                            maximum, (s * 3 + i + 1) * 10, 3, 1, rng.choice([0, 25])])
            ops.append([12, 100, s * 3 + .25]); generators[s].add(100)
        for step in range(250):
            choice = rng.randrange(12)
            if choice == 0:
                current = rng.randrange(4); ops.append([9, current])
            elif choice in [1, 2]:
                a, b = sorted(rng.sample(range(4), 2)); pair = (a, b)
                if choice == 1:
                    links[pair] = links.get(pair, 0) + 1; ops.append([10, a, b])
                elif links.get(pair, 0):
                    links[pair] -= 1; ops.append([11, a, b])
            elif choice == 3:
                ops.append([2, rng.choice([0, .1, 10, 1000, 100000]), rng.randrange(2)])
            elif choice == 4:
                ops.append([3, rng.choice([0, .1, 10, 1000, 100000]), rng.randrange(2), rng.choice([.5, 1, 2])])
            elif choice == 5:
                ops.append([6, rng.choice([1, 2, 32, 91]), rng.choice([0, .25]), rng.choice([.5, 1, 2])])
            elif choice == 6:
                id = rng.choice([100, 101])
                if id in generators[current]:
                    ops.append([1, id]); generators[current].remove(id)
                else:
                    ops.append([12, id, current * 3 + id - 99.75]); generators[current].add(id)
            elif choice == 7:
                # End-tick immediately after removal/link change must not
                # inject from a stale active passive generator.
                ops.append([13, .25, 1])
            elif choice == 8:
                ops.append([4, rng.randrange(3), rng.choice([0, .1, 1000])])
            elif choice == 9:
                ops.append([7])
            elif choice == 10:
                ops.append([8, rng.choice([0, 1, 1000, 100000])])
            else:
                ops.append([5, rng.randrange(3), rng.choice([0, .25, 1])])
        traces.append(ops)
    lines = [str(len(traces))]
    for ops in traces:
        lines.append(str(len(ops)))
        lines.extend(" ".join(map(str, op)) for op in ops)
    return "\n".join(lines) + "\n", sum(map(len, traces))


def main():
    global WORK
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, default=Path(os.environ.get("JAVA_HOME", "/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home")))
    parser.add_argument("--cases", type=int, default=200)
    parser.add_argument("--overlay", action="store_true", help="four grids, refcounted links and passive generators")
    args = parser.parse_args()
    if args.cases < 1:
        parser.error("--cases must be positive")
    WORK = WORK / ("overlay" if args.overlay else "single")
    upstream.verify(ROOT / "target/upstream/ae2")
    sources = generate()
    classes = WORK / "classes"; classes.mkdir(exist_ok=True)
    subprocess.run([str(args.java_home / "bin/javac"), "-d", str(classes), *map(str, sources)], check=True)
    inputs, operations = (overlay_cases if args.overlay else cases)(args.cases)
    (WORK / "inputs.txt").write_text(inputs)
    reference = subprocess.run([str(args.java_home / "bin/java"), "-cp", str(classes), "Main"], input=inputs, text=True, capture_output=True, check=True)
    (WORK / "java-stderr.txt").write_text(reference.stderr)
    subprocess.run(["cargo", "build", "--locked", "--example", "energy_oracle"], cwd=ROOT, check=True)
    actual = subprocess.run([str(ROOT / "target/debug/examples/energy_oracle")], input=inputs, text=True, capture_output=True, check=True)
    (WORK / "java.txt").write_text(reference.stdout); (WORK / "rust.txt").write_text(actual.stdout)
    java = reference.stdout.splitlines(); rust = actual.stdout.splitlines()
    if len(java) != operations or len(rust) != operations:
        raise ValueError("oracle output line count mismatch")
    failures = []
    for i, (a, b) in enumerate(zip(java, rust)):
        av = list(map(float, a.split())); bv = list(map(float, b.split()))
        if len(av) != len(bv) or any(not math.isclose(x, y, rel_tol=1e-11, abs_tol=1e-8) for x, y in zip(av, bv)):
            failures.append(i)
    report = {"passed": not failures, "cases": args.cases, "operations": operations,
              "failed_operations": len(failures), "first_failures": failures[:20],
              "upstream": upstream.LOCK["revision"], "input_sha256": hashlib.sha256(inputs.encode()).hexdigest(),
              "source_sha256": {name: hashlib.sha256((SOURCE / name).read_bytes()).hexdigest() for name in ORIGINALS},
              "scope": "unmodified original energy classes; single grid finite stores, unique priorities, all flow/public modes, node add/remove, stored buffers, events, simulate/commit, multipliers, cached totals, averages, idle/channel drain, warm-up; game/event infrastructure stubbed",
              "not_covered": ["watcher callbacks", "addon/creative storage handlers", "world lifecycle, client state and NeoForge"],
              "tolerance": {"relative": 1e-11, "absolute": 1e-8}}
    if args.overlay:
        report["scope"] = "unmodified original energy classes; four grids, unique provider priorities/capacities/generator rates, reference-counted overlay links, passive generators, removal between tick phases, capacity order, simulate/commit/multipliers and source totals; game infrastructure stubbed"
    else:
        report["not_covered"].insert(0, "overlay links and passive generators")
    (WORK / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
