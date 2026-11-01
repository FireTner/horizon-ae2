#!/usr/bin/env python3
"""Run unchanged AEBasePoweredItem against Rust component-energy rules; JDK 21+."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import random
import struct
import subprocess

import upstream

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'target/upstream/ae2/src/main/java'
WORK = ROOT / 'target/item-energy-oracle'
ORIGINALS = ['appeng/items/tools/powered/powersink/AEBasePoweredItem.java',
             'appeng/api/implementations/items/IAEItemPowerStorage.java',
             'appeng/api/config/AccessRestriction.java', 'appeng/api/config/Actionable.java']


def generate():
    files = {name: (SOURCE / name).read_text() for name in ORIGINALS}
    files.update({
        'net/neoforged/neoforge/fluids/capability/IFluidHandler.java': 'package net.neoforged.neoforge.fluids.capability; public interface IFluidHandler {enum FluidAction {EXECUTE,SIMULATE}}',
        'net/minecraft/network/chat/Component.java': 'package net.minecraft.network.chat; public class Component {}',
        'net/minecraft/util/Mth.java': 'package net.minecraft.util; public class Mth {public static int clamp(int a,int b,int c){return Math.min(c,Math.max(b,a));}public static int hsvToRgb(float a,float b,float c){return 0;}}',
        'net/minecraft/world/item/CreativeModeTab.java': 'package net.minecraft.world.item; public class CreativeModeTab {public static class ItemDisplayParameters {}public interface Output {void accept(ItemStack s);}}',
        'net/minecraft/world/item/TooltipFlag.java': 'package net.minecraft.world.item; public class TooltipFlag {}',
        'net/minecraft/world/item/ItemStack.java': '''package net.minecraft.world.item; public class ItemStack {
public final java.util.Map<String,Double> data=new java.util.HashMap<>(); public ItemStack(Object item,int count){}
public double getOrDefault(String key,double value){return data.getOrDefault(key,value);}public void set(String key,double v){data.put(key,v);}public void remove(String key){data.remove(key);}public static boolean isSameItem(ItemStack a,ItemStack b){return true;}}''',
        'net/neoforged/api/distmarker/Dist.java': 'package net.neoforged.api.distmarker; public enum Dist {CLIENT,DEDICATED_SERVER}',
        'net/neoforged/api/distmarker/OnlyIn.java': 'package net.neoforged.api.distmarker; public @interface OnlyIn {Dist value();}',
        'appeng/api/networking/energy/IAEPowerStorage.java': 'package appeng.api.networking.energy; public interface IAEPowerStorage {}',
        'appeng/api/ids/AEComponents.java': 'package appeng.api.ids; public class AEComponents {public static final String STORED_ENERGY="energy",ENERGY_CAPACITY="capacity";}',
        'appeng/core/localization/Tooltips.java': 'package appeng.core.localization; public class Tooltips {public static net.minecraft.network.chat.Component energyStorageComponent(double a,double b){return new net.minecraft.network.chat.Component();}}',
        'appeng/items/AEBaseItem.java': '''package appeng.items;
import java.util.*;import net.minecraft.world.item.*;import net.minecraft.network.chat.*;
public class AEBaseItem {public static class Properties {}public static class TooltipContext {}
public AEBaseItem(Properties p){} public void appendHoverText(ItemStack s,TooltipContext c,List<Component> l,TooltipFlag f){}
public void addToMainCreativeTab(CreativeModeTab.ItemDisplayParameters p,CreativeModeTab.Output o){}
public boolean isBarVisible(ItemStack s){return false;}public boolean shouldCauseReequipAnimation(ItemStack a,ItemStack b,boolean c){return false;}
public int getBarWidth(ItemStack s){return 0;}public int getBarColor(ItemStack s){return 0;}}''',
        'Main.java': '''import java.util.*;import appeng.items.tools.powered.powersink.*;import appeng.api.config.*;import appeng.api.ids.*;import net.minecraft.world.item.*;
public class Main {
static class Item extends AEBasePoweredItem {Item(double max){super(()->max,new Properties());}public double getChargeRate(ItemStack s){return 0;}void max(ItemStack s,double m){setAEMaxPower(s,m);}void multiplier(ItemStack s,int m){setAEMaxPowerMultiplier(s,m);}void reset(ItemStack s){resetAEMaxPower(s);}void current(ItemStack s,double p){setAECurrentPower(s,p);}}
static double value(Scanner in){return Double.parseDouble(in.next());}
public static void main(String[] args){Scanner in=new Scanner(System.in);int cases=in.nextInt();
for(int c=0;c<cases;c++){Item item=new Item(value(in));ItemStack s=new ItemStack(item,1);String max=in.next(),stored=in.next();if(!max.equals("none"))s.set(AEComponents.ENERGY_CAPACITY,Double.parseDouble(max));if(!stored.equals("none"))s.set(AEComponents.STORED_ENERGY,Double.parseDouble(stored));int n=in.nextInt();
for(int j=0;j<n;j++){String op=in.next();double result=0;switch(op){
case "I": {double a=value(in);boolean commit=in.nextInt()!=0;result=item.injectAEPower(s,a,commit?Actionable.MODULATE:Actionable.SIMULATE);break;}
case "E": {double a=value(in);boolean commit=in.nextInt()!=0;result=item.extractAEPower(s,a,commit?Actionable.MODULATE:Actionable.SIMULATE);break;}
case "B":result=item.getBarWidth(s);break;case "S":item.max(s,value(in));break;case "M":item.multiplier(s,in.nextInt());break;case "R":item.reset(s);break;case "C":item.current(s,value(in));break;default:throw new Error(op);}
System.out.println(result+" "+item.getAECurrentPower(s)+" "+item.getAEMaxPower(s)+" "+(s.data.containsKey(AEComponents.STORED_ENERGY)?1:0)+" "+(s.data.containsKey(AEComponents.ENERGY_CAPACITY)?1:0));}}}}
''',
    })
    sources = []
    for name, text in files.items():
        path = WORK / 'src' / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        sources.append(str(path))
    return sources


def cases(count, bars=False):
    rng = random.Random(90412021)
    lines = [str(count)]
    edge = ['0.0', '-0.0', '0.000099999', '0.0001', '0.000100001', '-1.0', 'NaN', 'Infinity', '-Infinity']
    for _ in range(count):
        default = rng.choice([100, 20000, 200000])
        lines.append(f'{default} {rng.choice(["none", str(default * 2)])} {rng.choice(["none", str(default + 100), "0.0001"])} 200')
        for i in range(200):
            op = rng.choice(['I', 'I', 'E', 'S', 'M', 'R', 'C', *(['B'] if bars else [])])
            value = rng.choice(edge) if i % 13 == 0 else str(rng.randrange(0, default * 3) / 8)
            if op in ('I', 'E'):
                lines.append(f'{op} {value} {rng.randrange(2)}')
            elif op == 'M':
                lines.append(f'M {rng.randrange(-5, 106)}')
            else:
                lines.append(op if op in ('R', 'B') else f'{op} {value}')
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cases', type=int, default=2000)
    parser.add_argument('--bars', action='store_true', help='include original energy-bar widths in the traces')
    parser.add_argument('--java-home', type=Path, default=Path('/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home'))
    args = parser.parse_args()
    if args.cases < 1:
        parser.error('--cases must be positive')
    upstream.verify(ROOT / 'target/upstream/ae2')
    sources = generate()
    classes = WORK / 'classes'; classes.mkdir(exist_ok=True)
    subprocess.run([str(args.java_home / 'bin/javac'), '-d', str(classes), *sources], check=True)
    inputs = cases(args.cases, args.bars)
    (WORK / 'inputs.txt').write_text(inputs)
    java = subprocess.run([str(args.java_home / 'bin/java'), '-cp', str(classes), 'Main'], input=inputs, text=True, capture_output=True, check=True).stdout
    subprocess.run(['cargo', 'build', '--locked', '--example', 'item_energy_oracle'], cwd=ROOT, check=True)
    rust = subprocess.run([str(ROOT / 'target/debug/examples/item_energy_oracle')], input=inputs, text=True, capture_output=True, check=True).stdout
    (WORK / 'java.txt').write_text(java); (WORK / 'rust.txt').write_text(rust)
    expected = args.cases * 200
    if len(java.splitlines()) != expected or len(rust.splitlines()) != expected:
        raise ValueError('output operation count mismatch')
    def equal(a, b):
        return (math.isnan(a) and math.isnan(b)) or struct.pack('>d', a) == struct.pack('>d', b)
    failures = []
    for i, (a, b) in enumerate(zip(java.splitlines(), rust.splitlines())):
        av = list(map(float, a.split())); bv = list(map(float, b.split()))
        if len(av) != len(bv) or not all(equal(x, y) for x, y in zip(av, bv)):
            failures.append(i)
    report = {'passed': not failures, 'cases': args.cases, 'operations': expected,
              'failed_operations': len(failures), 'first_failures': failures[:20],
              'upstream': upstream.LOCK['revision'], 'input_sha256': hashlib.sha256(inputs.encode()).hexdigest(),
              'source_sha256': {name: hashlib.sha256((SOURCE / name).read_bytes()).hexdigest() for name in ORIGINALS},
              'scope': 'unchanged AEBasePoweredItem energy methods; simulation/commit, overflow, extract, thresholds, capacity override/removal, multiplier/reset, overcapacity, negative/nonfinite inputs signed zero and energy bar widths; component map and Minecraft/UI infrastructure stubbed',
              'comparison': 'exact f64 bits, treating all NaN payloads as equivalent',
              'includes_bar_widths': args.bars,
              'not_covered': ['Minecraft component codecs', 'Charger/native world integration', 'tool charge rates/upgrades', 'bar color/tooltip/creative-tab/reequip behavior', 'dynamic config supplier changes during a trace']}
    (WORK / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    if failures:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
