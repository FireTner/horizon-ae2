#!/usr/bin/env python3
"""Compare verbatim pinned Charger.doWork and unchanged powered-item/energy classes.

The doWork method is compiled in a minimal host, not in the full Minecraft block
entity. Grid/slot/config/lifecycle infrastructure is stubbed. Only the charging
branch and common refill run; the recipe branch is retained but returns no recipe.
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct
import subprocess

import item_energy_oracle as item_oracle
import upstream

ROOT = item_oracle.ROOT
SOURCE = item_oracle.SOURCE
WORK = ROOT / 'target/charger-oracle'
CHARGER = 'appeng/blockentity/misc/ChargerBlockEntity.java'
EXTRA = ['appeng/me/energy/StoredEnergyAmount.java', 'appeng/api/config/PowerMultiplier.java']


def original_method():
    text = (SOURCE / CHARGER).read_text()
    start = text.index('    private void doWork(int ticksSinceLastCall) {')
    opening = text.index('{', start)
    depth = 1; end = opening + 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[start:end]


def generate():
    previous = item_oracle.WORK
    item_oracle.WORK = WORK
    try:
        sources = item_oracle.generate()
    finally:
        item_oracle.WORK = previous
    def write(name, text):
        path = WORK / 'src' / name; path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        if str(path) not in sources:
            sources.append(str(path))
    for name in EXTRA:
        write(name, (SOURCE / name).read_text())
    write('appeng/api/networking/events/GridPowerStorageStateChanged.java', 'package appeng.api.networking.events; public class GridPowerStorageStateChanged {public enum PowerEventType {PROVIDE_POWER,RECEIVE_POWER}}')
    write('net/minecraft/util/Mth.java', 'package net.minecraft.util; public class Mth {public static int clamp(int a,int b,int c){return Math.min(c,Math.max(b,a));}public static double clamp(double a,double b,double c){return a<b?b:Math.min(a,c);}public static int hsvToRgb(float a,float b,float c){return 0;}}')
    write('net/minecraft/world/item/ItemStack.java', '''package net.minecraft.world.item; public class ItemStack {
public final java.util.Map<String,Double> data=new java.util.HashMap<>();private final Object item;private final int count;public ItemStack(Object item,int count){this.item=item;this.count=count;}
public double getOrDefault(String key,double value){return data.getOrDefault(key,value);}public void set(String key,double v){data.put(key,v);}public void remove(String key){data.remove(key);}public static boolean isSameItem(ItemStack a,ItemStack b){return true;}public Object getItem(){return item;}public boolean isEmpty(){return count<=0;}public ItemStack copy(){return this;}}''')
    main = '''import java.util.*;import java.util.function.*;import appeng.items.tools.powered.powersink.*;import appeng.api.config.*;import appeng.api.ids.*;import appeng.api.implementations.items.*;import appeng.me.energy.*;import net.minecraft.world.item.*;
public class Main {
static class Item extends AEBasePoweredItem {final double rate;Item(double max,double rate){super(()->max,new Properties());this.rate=rate;}public double getChargeRate(ItemStack s){return rate;}}
static class AEConfig {static double factor;static AEConfig instance(){return new AEConfig();}double getChargerChargeRate(){return factor;}}
static class Platform {static boolean isChargeable(ItemStack s){return !s.isEmpty()&&s.getItem() instanceof IAEItemPowerStorage ps&&ps.getAEMaxPower(s)>0&&ps.getPowerFlow(s)!=AccessRestriction.READ;}}
enum PowerUnit {AE}
static class Recipe {ItemStack result;}
static class Level {Random getRandom(){return new Random(0);}}
static class ChargerRecipes {static Recipe findRecipe(Level l,ItemStack s){return null;}}
static class Inv {ItemStack item;Inv(ItemStack item){this.item=item;}ItemStack getStackInSlot(int s){return item;}void setItemDirect(int s,ItemStack value){item=value;}}
static class Grid {final StoredEnergyAmount energy;final ArrayList<Double> requests=new ArrayList<>();Grid(double amount){energy=new StoredEnergyAmount(amount,1e8,e->{});}Grid getEnergyService(){return this;}double extractAEPower(double amount,Actionable mode,PowerMultiplier m){requests.add(amount);return m.divide(energy.extract(m.multiply(amount),mode==Actionable.MODULATE));}}
static class Node {final Grid grid;Node(Grid grid){this.grid=grid;}Grid getGrid(){return grid;}void ifPresent(Consumer<Grid> f){if(grid!=null)f.accept(grid);}}
static class Charger {static final double POWER_THRESHOLD=1599;final Inv inv;final Node node;final StoredEnergyAmount buffer;final Level level=new Level();boolean working;
Charger(ItemStack item,double local,Grid grid){inv=new Inv(item);node=new Node(grid);buffer=new StoredEnergyAmount(local,1600,e->{});}Node getMainNode(){return node;}double getInternalCurrentPower(){return buffer.getAmount();}double getInternalMaxPower(){return 1600;}void setInternalCurrentPower(double amount){buffer.setStored(amount);}double extractAEPower(double amount,Actionable mode,PowerMultiplier m){return m.divide(buffer.extract(m.multiply(amount),mode==Actionable.MODULATE));}void injectExternalPower(PowerUnit unit,double amount,Actionable mode){buffer.insert(amount,mode==Actionable.MODULATE);}void markForUpdate(){}
''' + original_method() + '''
}
static double v(Scanner in){return Double.parseDouble(in.next());}
public static void main(String[] args){Scanner in=new Scanner(System.in);int cases=in.nextInt();for(int c=0;c<cases;c++){
double max=v(in),stored=v(in),override=v(in),local=v(in),gridAmount=v(in),rate=v(in);AEConfig.factor=v(in);PowerMultiplier.CONFIG.multiplier=v(in);
Item item=new Item(max,rate);ItemStack stack=new ItemStack(item,1);stack.set(AEComponents.STORED_ENERGY,stored);if(override>=0)stack.set(AEComponents.ENERGY_CAPACITY,override);Grid grid=gridAmount<0?null:new Grid(gridAmount);Charger charger=new Charger(stack,local,grid);int n=in.nextInt();
for(int i=0;i<n;i++){if(grid!=null)grid.requests.clear();charger.doWork(in.nextInt());int requests=grid==null?0:grid.requests.size();System.out.println(item.getAECurrentPower(stack)+" "+charger.getInternalCurrentPower()+" "+(grid==null?-1:grid.energy.getAmount())+" "+(charger.working?1:0)+" "+requests+" "+(requests>0?grid.requests.get(0):0)+" "+(requests>1?grid.requests.get(1):0));}}}}
'''
    write('Main.java', main)
    return sources


def cases(count):
    rng = random.Random(58192018)
    lines = [str(count)]
    for _ in range(count):
        maximum = rng.choice([320, 8000, 20000, 200000, 1600000])
        current = rng.choice([0, maximum - 10, maximum, maximum + 1, rng.randrange(maximum)])
        override = rng.choice([-1, 0, maximum / 2, maximum * 3])
        lines.append(' '.join(map(str, [maximum, current, override, rng.choice([0, 1e-7, 1598, 1599, 1600, rng.randrange(1600)]),
            rng.choice([-1, 0, 1, 1000, 10000, 5000000]), rng.choice([32, 80, 160, 240, 800, 1600, 2400]),
            rng.choice([0, .5, 1, 2, 16]), rng.choice([.5, 1, 2, 4]), 100])))
        lines.extend(str(rng.choice([1, 10, 20, 100])) for _ in range(100))
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cases', type=int, default=2000)
    parser.add_argument('--java-home', type=Path, default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args = parser.parse_args()
    if args.cases < 1:
        parser.error('--cases must be positive')
    upstream.verify(ROOT / 'target/upstream/ae2')
    sources = generate(); classes = WORK / 'classes'; classes.mkdir(exist_ok=True)
    subprocess.run([str(args.java_home / 'bin/javac'), '-d', str(classes), *sources], check=True)
    inputs = cases(args.cases); (WORK / 'inputs.txt').write_text(inputs)
    java = subprocess.run([str(args.java_home / 'bin/java'), '-cp', str(classes), 'Main'], input=inputs, text=True, capture_output=True, check=True).stdout
    subprocess.run(['cargo', 'build', '--locked', '--example', 'charger_oracle'], cwd=ROOT, check=True)
    rust = subprocess.run([str(ROOT / 'target/debug/examples/charger_oracle')], input=inputs, text=True, capture_output=True, check=True).stdout
    (WORK / 'java.txt').write_text(java); (WORK / 'rust.txt').write_text(rust)
    count = args.cases * 100
    if len(java.splitlines()) != count or len(rust.splitlines()) != count:
        raise ValueError('oracle line count mismatch')
    failures = []
    for i, (a, b) in enumerate(zip(java.splitlines(), rust.splitlines())):
        av = list(map(float, a.split())); bv = list(map(float, b.split()))
        if len(av) != len(bv) or any(struct.pack('>d', x) != struct.pack('>d', y) for x, y in zip(av, bv)):
            failures.append(i)
    report = {'passed': not failures, 'cases': args.cases, 'operations': count, 'failed_operations': len(failures), 'first_failures': failures[:20],
              'upstream': upstream.LOCK['revision'], 'input_sha256': hashlib.sha256(inputs.encode()).hexdigest(),
              'source_sha256': {name: hashlib.sha256((SOURCE / name).read_bytes()).hexdigest() for name in item_oracle.ORIGINALS + EXTRA + [CHARGER]},
              'do_work_sha256': hashlib.sha256(original_method().encode()).hexdigest(),
              'scope': 'verbatim ChargerBlockEntity.doWork method in a stub host; unchanged AEBasePoweredItem/StoredEnergyAmount/PowerMultiplier; chargeable item local/direct-grid/overflow/common-refill order, working flag, exact requests and remaining amounts, config multipliers, no-grid/empty-grid, capacity overrides and overfull items',
              'comparison': 'exact f64 bits',
              'not_covered': ['complete ChargerBlockEntity superclass/native/world lifecycle', 'actual grid EnergyService dispatch and events', 'recipe branch/RNG/recipe lookup', 'item-specific rate methods and live config suppliers', 'nonfinite or invalid config/local buffer inputs']}
    (WORK / 'report.json').write_text(json.dumps(report, indent=2) + '\n'); print(json.dumps(report, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
