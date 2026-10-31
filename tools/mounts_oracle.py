#!/usr/bin/env python3
"""Compare mounted Rust storage with ORIGINAL AE2 NetworkStorage (JDK 21+).

The original class is copied unchanged. Host inventories are bounded test bins;
AEKey, KeyCounter, Component and action-source infrastructure are minimal stubs.
The oracle exercises mount order, callbacks, queued mutations and recursion. It
does not verify cell bytes/codecs, grid provider lifecycle, security or world UI.
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
WORK = ROOT / "target/mounts-oracle"
ORIGINAL = "appeng/me/storage/NetworkStorage.java"


def generate():
    files = {ORIGINAL: (SOURCE / ORIGINAL).read_text(),
        "org/jetbrains/annotations/Nullable.java": "package org.jetbrains.annotations;public @interface Nullable {}",
        "com/google/common/base/Preconditions.java": "package com.google.common.base;public class Preconditions {public static void checkState(boolean ok){if(!ok)throw new IllegalStateException();}}",
        "net/minecraft/network/chat/Component.java": "package net.minecraft.network.chat;public class Component {}",
        "appeng/core/localization/GuiText.java": "package appeng.core.localization;public enum GuiText {MENetworkStorage;public net.minecraft.network.chat.Component text(){return new net.minecraft.network.chat.Component();}}",
        "appeng/api/config/Actionable.java": "package appeng.api.config;public enum Actionable {MODULATE,SIMULATE}",
        "appeng/api/networking/security/IActionSource.java": "package appeng.api.networking.security;public interface IActionSource {}",
        "appeng/api/stacks/AEKey.java": "package appeng.api.stacks;public record AEKey(int id) {}",
        "appeng/api/stacks/KeyCounter.java": "package appeng.api.stacks;public class KeyCounter {private final java.util.Map<AEKey,Long> values=new java.util.HashMap<>();public void add(AEKey k,long a){values.put(k,get(k)+a);}public long get(AEKey k){return values.getOrDefault(k,0L);}}",
        "appeng/api/storage/MEStorage.java": """package appeng.api.storage;import appeng.api.config.*;import appeng.api.stacks.*;import appeng.api.networking.security.*;
public interface MEStorage {default boolean isPreferredStorageFor(AEKey k,IActionSource s){return false;}default long insert(AEKey k,long a,Actionable m,IActionSource s){return 0;}default long extract(AEKey k,long a,Actionable m,IActionSource s){return 0;}default void getAvailableStacks(KeyCounter out){}net.minecraft.network.chat.Component getDescription();}""",
        "Main.java": """import java.util.*;import appeng.me.storage.*;import appeng.api.storage.*;import appeng.api.stacks.*;import appeng.api.config.*;import appeng.api.networking.security.*;import net.minecraft.network.chat.*;
public class Main {
 static AEKey key=new AEKey(1);static IActionSource source=new IActionSource(){};
 record Action(int phase,MEStorage target,int flags,int priority){}
 static class Bin implements MEStorage {
  long count,capacity;boolean preferred;NetworkStorage net;Action action;
  void trigger(int phase){if(action!=null&&action.phase==phase){Action a=action;action=null;if((a.flags&1)!=0)net.unmount(a.target);if((a.flags&2)!=0)net.mount(a.priority,a.target);if((a.flags&4)!=0&&net.insert(key,1,Actionable.MODULATE,source)!=0)throw new IllegalStateException("recursion");}}
  public boolean isPreferredStorageFor(AEKey k,IActionSource s){trigger(1);return preferred;}
  public long insert(AEKey k,long a,Actionable m,IActionSource s){trigger(2);long v=Math.min(a,capacity-count);if(m==Actionable.MODULATE)count+=v;return v;}
  public long extract(AEKey k,long a,Actionable m,IActionSource s){trigger(3);long v=Math.min(a,count);if(m==Actionable.MODULATE)count-=v;return v;}
  public void getAvailableStacks(KeyCounter out){trigger(4);out.add(key,count);}public Component getDescription(){return new Component();}
 }
 public static void main(String[] args){Scanner in=new Scanner(System.in);int cases=in.nextInt();
 for(int c=0;c<cases;c++){int n=in.nextInt(),ops=in.nextInt();NetworkStorage net=new NetworkStorage();Bin[] bins=new Bin[n];MEStorage[] inventories=new MEStorage[n+1];
 for(int i=0;i<n;i++){Bin b=new Bin();b.capacity=in.nextLong();b.count=in.nextLong();b.preferred=in.nextInt()!=0;b.net=net;bins[i]=b;inventories[i]=b;}inventories[n]=net;
 for(int step=0;step<ops;step++){int op=in.nextInt();long result=0;
 switch(op){case 0->{int p=in.nextInt();net.mount(p,inventories[in.nextInt()]);}case 1->net.unmount(inventories[in.nextInt()]);
 case 2->{long a=in.nextLong();result=net.insert(key,a,in.nextInt()!=0?Actionable.MODULATE:Actionable.SIMULATE,source);}
 case 3->{long a=in.nextLong();result=net.extract(key,a,in.nextInt()!=0?Actionable.MODULATE:Actionable.SIMULATE,source);}
 case 4->{KeyCounter out=new KeyCounter();net.getAvailableStacks(out);result=out.get(key);}
 case 5->bins[in.nextInt()].preferred=in.nextInt()!=0;
 case 6->{int id=in.nextInt(),phase=in.nextInt(),target=in.nextInt(),flags=in.nextInt(),priority=in.nextInt();bins[id].action=new Action(phase,inventories[target],flags,priority);}
 case 7->bins[in.nextInt()].count=in.nextLong();default->throw new IllegalArgumentException();}
 System.out.print(result);for(Bin b:bins)System.out.print(" "+b.count);System.out.println();
 }
 }
 }
}"""}
    root = WORK / "java"
    for name, content in files.items():
        path = root / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text(content)
    return [root / name for name in sorted(files)]


def cases(count):
    rng = random.Random(0xAE219218D)
    traces = []
    for _ in range(count):
        n = rng.randrange(2, 10)
        capacities = [rng.choice([10, 100, 1000, 10000000000]) for _ in range(n)]
        bins = [(cap, rng.choice([0, cap // 2, cap]), rng.randrange(2)) for cap in capacities]
        ops = [[0, rng.choice([-10, 0, 0, 10]), id] for id in range(n)]
        for _ in range(200):
            op = rng.choices(range(8), [8, 8, 20, 20, 15, 5, 20, 4])[0]
            if op == 0:
                ops.append([op, rng.choice([-100, -10, 0, 10, 100]), rng.randrange(n + 1)])
            elif op == 1:
                ops.append([op, rng.randrange(n + 1)])
            elif op in [2, 3]:
                ops.append([op, rng.choice([0, 1, 10, 10000, 100000000000]), rng.randrange(2)])
            elif op == 4:
                ops.append([op])
            elif op == 5:
                ops.append([op, rng.randrange(n), rng.randrange(2)])
            elif op == 6:
                ops.append([op, rng.randrange(n), rng.randrange(1, 5), rng.randrange(n + 1),
                            rng.randrange(1, 8), rng.choice([-10, 0, 10])])
            else:
                id = rng.randrange(n); ops.append([op, id, rng.choice([0, capacities[id]])])
        traces.append((bins, ops))
    lines = [str(count)]
    for bins, ops in traces:
        lines.append(f"{len(bins)} {len(ops)}")
        lines.extend(" ".join(map(str, bin)) for bin in bins)
        lines.extend(" ".join(map(str, op)) for op in ops)
    return "\n".join(lines) + "\n", sum(len(ops) for _, ops in traces)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, default=Path(os.environ.get("JAVA_HOME", "/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home")))
    parser.add_argument("--cases", type=int, default=200)
    args = parser.parse_args()
    if args.cases < 1:
        parser.error("--cases must be positive")
    upstream.verify(ROOT / "target/upstream/ae2")
    sources = generate(); classes = WORK / "classes"; classes.mkdir(exist_ok=True)
    subprocess.run([str(args.java_home / "bin/javac"), "-d", str(classes), *map(str, sources)], check=True)
    inputs, operations = cases(args.cases); (WORK / "inputs.txt").write_text(inputs)
    reference = subprocess.run([str(args.java_home / "bin/java"), "-cp", str(classes), "Main"], input=inputs, text=True, capture_output=True, check=True).stdout
    subprocess.run(["cargo", "build", "--locked", "--example", "mounts_oracle"], cwd=ROOT, check=True)
    actual = subprocess.run([str(ROOT / "target/debug/examples/mounts_oracle")], input=inputs, text=True, capture_output=True, check=True).stdout
    (WORK / "java.txt").write_text(reference); (WORK / "rust.txt").write_text(actual)
    java = reference.splitlines(); rust = actual.splitlines()
    if len(java) != operations or len(rust) != operations:
        raise ValueError("oracle output line count mismatch")
    failures = [i for i, (a, b) in enumerate(zip(java, rust)) if a != b]
    report = {"passed": not failures, "cases": args.cases, "operations": operations,
              "failed_operations": len(failures), "first_failures": failures[:20],
              "upstream": upstream.LOCK["revision"], "input_sha256": hashlib.sha256(inputs.encode()).hexdigest(),
              "source_sha256": {ORIGINAL: hashlib.sha256((SOURCE / ORIGINAL).read_bytes()).hexdigest()},
              "scope": "unmodified NetworkStorage; bounded host bins, mount/unmount, duplicate mounts, high/low priorities, preferred two-pass insertion, simulate/commit, recursive self-mounts, queued mutations in all provider operations, deferred listing queues; exact integer outputs",
              "not_covered": ["cell internals/codecs", "StorageService and provider lifecycle", "security/energy charging", "world Drive, terminal UI, NeoForge"]}
    (WORK / "report.json").write_text(json.dumps(report, indent=2) + "\n"); print(json.dumps(report, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
