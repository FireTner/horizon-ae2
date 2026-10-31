#!/usr/bin/env python3
"""Compare Rust cells with unchanged AE2 BasicCellInventory (JDK 21+).

The real inventory algorithm runs against minimal item/key/component, fastutil,
partition and upgrade stubs. This checks exact partitions and basic/portable
capacity profiles, not Minecraft codecs, actual fuzzy matching, upgrades UI or
world lifecycle. Fake key callbacks exercise blacklist/nested-cell branches.
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
WORK = ROOT / "target/cells-oracle"
ORIGINAL = "appeng/me/cells/BasicCellInventory.java"
CREATIVE_ORIGINAL = "appeng/me/cells/CreativeCellInventory.java"


def generate(creative=False):
    files = {ORIGINAL: (SOURCE / ORIGINAL).read_text(),
        "org/jetbrains/annotations/Nullable.java": "package org.jetbrains.annotations;public @interface Nullable {}",
        "net/minecraft/network/chat/Component.java": "package net.minecraft.network.chat;public class Component {}",
        "appeng/api/config/Actionable.java": "package appeng.api.config;public enum Actionable {MODULATE,SIMULATE}",
        "appeng/api/config/FuzzyMode.java": "package appeng.api.config;public enum FuzzyMode {IGNORE_ALL}",
        "appeng/api/config/IncludeExclude.java": "package appeng.api.config;public enum IncludeExclude {WHITELIST,BLACKLIST}",
        "appeng/api/networking/security/IActionSource.java": "package appeng.api.networking.security;public interface IActionSource {}",
        "appeng/api/ids/AEComponents.java": "package appeng.api.ids;public class AEComponents {public static final Object STORAGE_CELL_INV=new Object();}",
        "appeng/core/definitions/AEItems.java": "package appeng.core.definitions;public enum AEItems {INVERTER_CARD,FUZZY_CARD,EQUAL_DISTRIBUTION_CARD,VOID_CARD}",
        "appeng/api/upgrades/IUpgradeInventory.java": "package appeng.api.upgrades;public interface IUpgradeInventory {boolean isInstalled(Object item);}",
        "net/minecraft/world/item/ItemStack.java": """package net.minecraft.world.item;
public class ItemStack {public Object item;public boolean fits=true;java.util.Map<Object,Object> data=new java.util.HashMap<>();
public ItemStack(Object item){this.item=item;}public Object getItem(){return item;}
@SuppressWarnings("unchecked") public <T>T getOrDefault(Object k,T d){return (T)data.getOrDefault(k,d);}public void remove(Object k){data.remove(k);}public <T>void set(Object k,T v){data.put(k,v);}
public net.minecraft.network.chat.Component getHoverName(){return new net.minecraft.network.chat.Component();}}
""",
        "appeng/api/stacks/AEKeyType.java": """package appeng.api.stacks;public class AEKeyType {
public static final AEKeyType ITEMS=new AEKeyType(8),FLUIDS=new AEKeyType(8000);int per;AEKeyType(int per){this.per=per;}
public boolean contains(AEKey k){return k.kind==this;}public int getAmountPerByte(){return per;}}
""",
        "appeng/api/stacks/AEKey.java": """package appeng.api.stacks;public class AEKey {public final int id;public final AEKeyType kind;
public AEKey(int id,AEKeyType kind){this.id=id;this.kind=kind;}public boolean equals(Object o){return o instanceof AEKey k&&id==k.id&&kind==k.kind;}public int hashCode(){return id*31+(kind==AEKeyType.ITEMS?0:1);}}
""",
        "appeng/api/stacks/AEItemKey.java": """package appeng.api.stacks;public class AEItemKey extends AEKey {public AEItemKey(int id){super(id,AEKeyType.ITEMS);}
public net.minecraft.world.item.ItemStack toStack(){var s=new net.minecraft.world.item.ItemStack(null);s.fits=id!=79;return s;}}
""",
        "appeng/api/stacks/GenericStack.java": "package appeng.api.stacks;public record GenericStack(AEKey what,long amount) {}",
        "appeng/api/stacks/KeyCounter.java": """package appeng.api.stacks;public class KeyCounter {java.util.Map<AEKey,Long> data=new java.util.HashMap<>();
public void add(AEKey k,long a){data.merge(k,a,Long::sum);}public long get(AEKey k){return data.getOrDefault(k,0L);}public boolean isEmpty(){return data.values().stream().allMatch(v->v==0);}}
""",
        "appeng/util/ConfigInventory.java": "package appeng.util;public record ConfigInventory(java.util.Set<appeng.api.stacks.AEKey> keySet) {}",
        "appeng/util/prioritylist/IPartitionList.java": """package appeng.util.prioritylist;import appeng.api.stacks.*;import appeng.api.config.*;
public interface IPartitionList {boolean isEmpty();boolean matchesFilter(AEKey k,IncludeExclude mode);static Builder builder(){return new Builder();}
class Builder {java.util.Set<AEKey> keys=new java.util.HashSet<>();public Builder fuzzyMode(FuzzyMode f){return this;}public Builder addAll(java.util.Collection<AEKey> k){keys.addAll(k);return this;}
public IPartitionList build(){return new IPartitionList(){public boolean isEmpty(){return keys.isEmpty();}public boolean matchesFilter(AEKey k,IncludeExclude m){return keys.isEmpty()||keys.contains(k)==(m==IncludeExclude.WHITELIST);}};}}}
""",
        "appeng/util/prioritylist/FuzzyPriorityList.java": "package appeng.util.prioritylist;public abstract class FuzzyPriorityList implements IPartitionList {}",
        "appeng/api/storage/cells/CellState.java": (SOURCE / "appeng/api/storage/cells/CellState.java").read_text(),
        "appeng/api/storage/cells/ISaveProvider.java": "package appeng.api.storage.cells;public interface ISaveProvider {void saveChanges();}",
        "appeng/api/storage/cells/StorageCell.java": """package appeng.api.storage.cells;import appeng.api.stacks.*;import appeng.api.config.*;import appeng.api.networking.security.*;
public interface StorageCell {default boolean canFitInsideCell(){return true;}default void persist(){}default void getAvailableStacks(KeyCounter out){}default KeyCounter getAvailableStacks(){var out=new KeyCounter();getAvailableStacks(out);return out;}
default double getIdleDrain(){return 0;}default CellState getStatus(){return CellState.EMPTY;}default long insert(AEKey k,long a,Actionable m,IActionSource s){return 0;}default long extract(AEKey k,long a,Actionable m,IActionSource s){return 0;}
default net.minecraft.network.chat.Component getDescription(){return new net.minecraft.network.chat.Component();}}
""",
        "appeng/api/storage/StorageCells.java": """package appeng.api.storage;public class StorageCells {public static appeng.api.storage.cells.StorageCell getCellInventory(net.minecraft.world.item.ItemStack s,Object p){return new appeng.api.storage.cells.StorageCell(){public boolean canFitInsideCell(){return s.fits;}};}}
""",
        "appeng/api/storage/cells/IBasicCellItem.java": """package appeng.api.storage.cells;import net.minecraft.world.item.*;import appeng.api.stacks.*;import appeng.api.config.*;
public interface IBasicCellItem {int getBytes(ItemStack s);int getBytesPerType(ItemStack s);int getTotalTypes(ItemStack s);AEKeyType getKeyType();appeng.util.ConfigInventory getConfigInventory(ItemStack s);appeng.api.upgrades.IUpgradeInventory getUpgrades(ItemStack s);
default FuzzyMode getFuzzyMode(ItemStack s){return FuzzyMode.IGNORE_ALL;}default boolean storableInStorageCell(){return false;}default boolean isStorageCell(ItemStack s){return true;}default double getIdleDrain(){return 0.5;}default boolean isBlackListed(ItemStack s,AEKey k){return k.id==78;}}
""",
        "it/unimi/dsi/fastutil/objects/Object2LongMap.java": """package it.unimi.dsi.fastutil.objects;public interface Object2LongMap<K> {interface Entry<K>{K getKey();long getLongValue();}
int size();long getLong(Object k);long put(K k,long v);boolean remove(Object k,long v);boolean containsKey(Object k);java.util.Collection<Long> values();java.lang.Iterable<Entry<K>> object2LongEntrySet();}
""",
        "it/unimi/dsi/fastutil/objects/Object2LongOpenHashMap.java": """package it.unimi.dsi.fastutil.objects;public class Object2LongOpenHashMap<K> implements Object2LongMap<K> {
java.util.Map<K,Long> data=new java.util.HashMap<>();public int size(){return data.size();}public long getLong(Object k){return data.getOrDefault(k,0L);}public long put(K k,long v){Long old=data.put(k,v);return old==null?0:old;}public boolean remove(Object k,long v){return data.remove(k,v);}public boolean containsKey(Object k){return data.containsKey(k);}public java.util.Collection<Long> values(){return data.values();}
public Iterable<Object2LongMap.Entry<K>> object2LongEntrySet(){var out=new java.util.ArrayList<Object2LongMap.Entry<K>>();for(var e:data.entrySet())out.add(new Object2LongMap.Entry<K>(){public K getKey(){return e.getKey();}public long getLongValue(){return e.getValue();}});return out;}}
""",
        "it/unimi/dsi/fastutil/objects/Object2LongMaps.java": "package it.unimi.dsi.fastutil.objects;public class Object2LongMaps {public static <K>Iterable<Object2LongMap.Entry<K>> fastIterable(Object2LongMap<K> m){return m.object2LongEntrySet();}}",
        "Main.java": """import java.util.*;import java.io.*;import appeng.api.stacks.*;import appeng.api.config.*;import appeng.api.storage.cells.*;import appeng.me.cells.*;import appeng.util.*;import net.minecraft.world.item.*;
public class Main {
static AEKey key(AEKeyType kind,int id){if(id==81)kind=kind==AEKeyType.ITEMS?AEKeyType.FLUIDS:AEKeyType.ITEMS;return kind==AEKeyType.ITEMS?new AEItemKey(id):new AEKey(id,kind);}
record Cell(int bytes,int overhead,int types,AEKeyType kind,int flags,ConfigInventory config) implements IBasicCellItem {
public int getBytes(ItemStack s){return bytes;}public int getBytesPerType(ItemStack s){return overhead;}public int getTotalTypes(ItemStack s){return types;}public AEKeyType getKeyType(){return kind;}public ConfigInventory getConfigInventory(ItemStack s){return config;}
public appeng.api.upgrades.IUpgradeInventory getUpgrades(ItemStack s){return item->switch((appeng.core.definitions.AEItems)item){case INVERTER_CARD->(flags&1)!=0;case EQUAL_DISTRIBUTION_CARD->(flags&2)!=0;case VOID_CARD->(flags&4)!=0;default->false;};}}
public static void main(String[] args) throws Exception {var in=new Scanner(new BufferedInputStream(System.in));var out=new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.out)));int cases=in.nextInt();
for(int c=0;c<cases;c++){int tier=in.nextInt();AEKeyType kind=in.nextInt()==0?AEKeyType.ITEMS:AEKeyType.FLUIDS;boolean portable=in.nextInt()!=0;int flags=in.nextInt(),n=in.nextInt();var keys=new HashSet<AEKey>();for(int k=0;k<n;k++)keys.add(key(kind,in.nextInt()));
int base=1024<<(2*tier),types=kind==AEKeyType.ITEMS?(portable?63-(tier+1)*9:63):18;var item=new Cell(portable?base/2:base,base/128,types,kind,flags,new ConfigInventory(keys));var stack=new ItemStack(item);var inv=BasicCellInventory.createInventory(stack,null);int operations=in.nextInt();
for(int step=0;step<operations;step++){int op=in.nextInt(),id=in.nextInt();long amount=in.nextLong();Actionable mode=in.nextInt()==0?Actionable.SIMULATE:Actionable.MODULATE;long result=0;
switch(op){case 0->result=inv.insert(key(kind,id),amount,mode,null);case 1->result=inv.extract(key(kind,id),amount,mode,null);case 2->{inv.persist();inv=BasicCellInventory.createInventory(stack,null);}default->throw new IllegalArgumentException();}
out.print(result+" "+inv.getStoredItemCount()+" "+inv.getStoredItemTypes()+" "+inv.getUsedBytes()+" "+inv.getFreeBytes()+" "+inv.getRemainingItemTypes()+" "+inv.getRemainingItemCount()+" "+inv.getStatus().ordinal());var counts=inv.getAvailableStacks();for(int k=0;k<82;k++)out.print(" "+counts.get(key(kind,k)));out.println();}}
out.flush();}}
"""}
    if creative:
        files[CREATIVE_ORIGINAL] = (SOURCE / CREATIVE_ORIGINAL).read_text()
        storage = "appeng/api/storage/cells/StorageCell.java"
        files[storage] = files[storage].replace("public interface StorageCell {", "public interface StorageCell {default boolean isPreferredStorageFor(AEKey k,IActionSource s){return false;}")
        files["appeng/items/contents/CellConfig.java"] = """package appeng.items.contents;public class CellConfig {public static appeng.util.ConfigInventory create(net.minecraft.world.item.ItemStack s){return s.getOrDefault("config",new appeng.util.ConfigInventory(java.util.Set.of()));}}"""
        files["appeng/me/cells/CreativeFactory.java"] = """package appeng.me.cells;public class CreativeFactory {public static appeng.api.storage.cells.StorageCell create(net.minecraft.world.item.ItemStack s){return new CreativeCellInventory(s);}}"""
        files["Main.java"] = """import java.util.*;import java.io.*;import appeng.api.stacks.*;import appeng.api.config.*;import appeng.api.storage.cells.*;import appeng.me.cells.*;import appeng.util.*;import net.minecraft.world.item.*;
public class Main {
static AEKey key(AEKeyType kind,int id){if(id==81)kind=kind==AEKeyType.ITEMS?AEKeyType.FLUIDS:AEKeyType.ITEMS;return kind==AEKeyType.ITEMS?new AEItemKey(id):new AEKey(id,kind);}
public static void main(String[] args) throws Exception {var in=new Scanner(new BufferedInputStream(System.in));var out=new PrintWriter(new BufferedWriter(new OutputStreamWriter(System.out)));int cases=in.nextInt();
for(int c=0;c<cases;c++){in.nextInt();AEKeyType kind=in.nextInt()==0?AEKeyType.ITEMS:AEKeyType.FLUIDS;in.nextInt();in.nextInt();int n=in.nextInt();var keys=new HashSet<AEKey>();for(int k=0;k<n;k++)keys.add(key(kind,in.nextInt()));var stack=new ItemStack(null);stack.set("config",new ConfigInventory(keys));StorageCell inv=CreativeFactory.create(stack);int operations=in.nextInt();
for(int step=0;step<operations;step++){int op=in.nextInt(),id=in.nextInt();long amount=in.nextLong();Actionable mode=in.nextInt()==0?Actionable.SIMULATE:Actionable.MODULATE;long result=0;
switch(op){case 0->result=inv.insert(key(kind,id),amount,mode,null);case 1->result=inv.extract(key(kind,id),amount,mode,null);case 2->{inv.persist();inv=CreativeFactory.create(stack);}default->throw new IllegalArgumentException();}
out.print(result+" "+inv.getStatus().ordinal()+" "+Double.doubleToRawLongBits(inv.getIdleDrain())+" "+(inv.canFitInsideCell()?1:0)+" "+(inv.isPreferredStorageFor(key(kind,id),null)?1:0));var counts=inv.getAvailableStacks();for(int k=0;k<82;k++)out.print(" "+counts.get(key(kind,k)));out.println();}}
out.flush();}}
"""
    root = WORK / "java"
    for name, content in files.items():
        path = root / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text(content)
    return [root / name for name in sorted(files)]


def cases(count, creative=False):
    rng = random.Random(0xAE219218CE)
    lines = [str(count)]; operations = 0
    for case in range(count):
        tier = case % 5; kind = case // 5 % 2; portable = case // 10 % 2; flags = case // 20 % 8
        filters = rng.sample(range(82), rng.choice([0, 1, 3, 20, 63])) if creative else rng.sample(range(82 - 1), rng.choice([0, 1, 3, 20, 70]))
        lines.append(f"{tier} {kind} {portable} {flags} {len(filters)} " + " ".join(map(str, filters)))
        ops = [[0, key, 1, 1] for key in range(82)]
        # Fill, reject, extract and reopen at byte/type boundaries, then mix
        # small/large amounts and simulation without allowing invalid counts.
        ops += [[0, 0, 10**12, 0], [0, 0, 10**12, 1], [0, 81, 1, 1], [2, 0, 0, 0]]
        for _ in range(200):
            op = rng.choices([0, 1, 2], [48, 48, 4])[0]
            ops.append([op, rng.randrange(82), rng.choice([0, 1, 7, 8, 9, 64, 1000, 4032, 8128, 10**12]), rng.randrange(2)])
        lines.append(str(len(ops))); lines.extend(" ".join(map(str, op)) for op in ops); operations += len(ops)
    return "\n".join(lines) + "\n", operations


def main():
    global WORK
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", type=Path, default=Path(os.environ.get("JAVA_HOME", "/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home")))
    parser.add_argument("--cases", type=int, default=200)
    parser.add_argument("--creative", action="store_true", help="compare original CreativeCellInventory instead")
    args = parser.parse_args()
    if args.cases < 1:
        parser.error("--cases must be positive")
    upstream.verify(ROOT / "target/upstream/ae2")
    if args.creative:
        WORK = ROOT / "target/creative-cell-oracle"
    original = CREATIVE_ORIGINAL if args.creative else ORIGINAL
    sources = generate(args.creative); classes = WORK / "classes"; classes.mkdir(exist_ok=True)
    subprocess.run([str(args.java_home / "bin/javac"), "-d", str(classes), *map(str, sources)], check=True)
    inputs, operations = cases(args.cases, args.creative); (WORK / "inputs.txt").write_text(inputs)
    reference = subprocess.run([str(args.java_home / "bin/java"), "-cp", str(classes), "Main"], input=inputs, text=True, capture_output=True, check=True).stdout
    example = "creative_oracle" if args.creative else "cells_oracle"
    subprocess.run(["cargo", "build", "--locked", "--example", example], cwd=ROOT, check=True)
    actual = subprocess.run([str(ROOT / "target/debug/examples" / example)], input=inputs, text=True, capture_output=True, check=True).stdout
    (WORK / "java.txt").write_text(reference); (WORK / "rust.txt").write_text(actual)
    java = reference.splitlines(); rust = actual.splitlines()
    if len(java) != operations or len(rust) != operations:
        raise ValueError("oracle output line count mismatch")
    failures = [i for i, (a, b) in enumerate(zip(java, rust)) if a != b]
    report = {"passed": not failures, "cases": args.cases, "operations": operations,
              "failed_operations": len(failures), "first_failures": failures[:20],
              "upstream": upstream.LOCK["revision"], "input_sha256": hashlib.sha256(inputs.encode()).hexdigest(),
              "source_sha256": {name: hashlib.sha256((SOURCE / name).read_bytes()).hexdigest() for name in [original, "appeng/api/storage/cells/CellState.java"]},
              "profile_source_sha256": {name: hashlib.sha256((SOURCE / name).read_bytes()).hexdigest() for name in [
                  "appeng/items/tools/powered/PortableCellItem.java", "appeng/core/definitions/AEItems.java", "appeng/items/storage/StorageTier.java"]},
              "scope": "unchanged BasicCellInventory; independently specified basic/portable profiles, all tiers/key kinds, exact partitions, inverter/distribution/void, simulated/committed insertion/extraction, reload, byte/type limits, fake blacklist/nested-cell callbacks; exact per-key and metadata integer outputs",
              "not_covered": ["actual Minecraft AEKey/component codecs", "actual fuzzy matcher and upgrade validation", "missing/malformed components", "creative/tool cells", "world lifecycle, native capability API and NeoForge"]}
    if args.creative:
        report["profile_source_sha256"] = {name: hashlib.sha256((SOURCE / name).read_bytes()).hexdigest() for name in [
            "appeng/items/contents/CellConfig.java", "appeng/items/storage/CreativeCellItem.java"]}
        report["scope"] = "unchanged CreativeCellInventory; exact configured keys, mixed key kinds, simulate/commit insert/extract, Integer.MAX_VALUE listing independent of requested amount, preferred/nested flags, status, zero idle drain and reopen; keys and configuration infrastructure stubbed"
        report["not_covered"] = ["actual Minecraft component/config codecs", "workbench and tooltip UI", "addon key types", "world lifecycle, capability API and NeoForge"]
    report["not_covered"].append("Java hash/identity iteration and serialized list order")
    (WORK / "report.json").write_text(json.dumps(report, indent=2) + "\n"); print(json.dumps(report, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
