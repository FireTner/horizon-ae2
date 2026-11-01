#!/usr/bin/env python3
"""Verbatim EnergyCellBlockItem power methods and source item fill predicate.

Components/block profiles are a minimal host. Legacy override float comparison
is verified against the pinned official Minecraft 1.21.1 BakedOverride bytecode.
This does not exercise placement, loot, NeoForge or the native world lifecycle.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import random
import struct
import subprocess
import zipfile

import upstream
from controller_oracle import fragment
ROOT=upstream.ROOT
WORK=ROOT/'target/energy-cell-item-oracle'
FILE='block/networking/EnergyCellBlockItem.java'


def bits(value):return struct.unpack('>Q',struct.pack('>d',value))[0]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--java-home',type=Path,default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None)
    parser.add_argument('--cases',type=int,default=100000)
    args=parser.parse_args()
    upstream.verify(upstream.SOURCE)
    if args.cases<1000:parser.error('at least 1000 cases required')
    text=(upstream.SOURCE/'src/main/java/appeng'/FILE).read_text()
    methods={name:fragment(text,header) for name,header in {
        'inject':'public double injectAEPower(', 'extract':'public double extractAEPower(',
        'maximum':'public double getAEMaxPower(', 'current':'public double getAECurrentPower(',
        'rate':'public double getChargeRate(', 'capacity':'private double getMaxEnergyCapacity(',
        'set':'private void setAECurrentPower('}.items()}
    definition=(upstream.SOURCE/'src/main/java/appeng/core/definitions/AEBlocks.java').read_text()
    assert 'new EnergyCellBlock(200000, 800, 200)' in definition
    assert 'new EnergyCellBlock(1600000, 1600, 1600)' in definition
    model_property=(upstream.SOURCE/'src/main/java/appeng/init/client/InitItemModelsProperties.java').read_text()
    assert 'return (float) (curPower / maxPower);' in model_property
    binary=lambda name:str(args.java_home/'bin'/name) if args.java_home else name
    # The official artifacts and mappings were downloaded/verified in the comparator stage.
    reference=ROOT/'target/minecraft-1.21.1-reference'
    version=json.loads((reference/'version.json').read_text())
    assert hashlib.sha1((reference/'client.jar').read_bytes()).hexdigest()==version['downloads']['client']['sha1']
    assert hashlib.sha1((reference/'client_mappings.txt').read_bytes()).hexdigest()==version['downloads']['client_mappings']['sha1']
    mapping=(reference/'client_mappings.txt').read_text()
    assert 'net.minecraft.client.renderer.block.model.ItemOverrides$BakedOverride -> ggg$a:' in mapping
    bytecode=subprocess.check_output([binary('javap'),'-classpath',str(reference/'client.jar'),'-c','-p','ggg$a'],text=True)
    assert 'fcmpg' in bytecode and 'ifge' in bytecode,bytecode
    WORK.mkdir(parents=True,exist_ok=True);(WORK/'BakedOverride.javap.txt').write_text(bytecode)
    with zipfile.ZipFile(ROOT/'target/jars/appliedenergistics2-19.2.18.jar') as jar:
        model_hashes={}
        for name in ('energy_cell','dense_energy_cell'):
            path=f'assets/ae2/models/item/{name}.json';raw=jar.read(path);model=json.loads(raw)
            thresholds=[x['predicate']['ae2:fill_level'] for x in model['overrides']]
            assert thresholds==[.2,.4,.6,.8]
            assert [x['model'] for x in model['overrides']]==[f'ae2:block/{name}_{i}' for i in range(1,5)]
            model_hashes[path]=hashlib.sha256(raw).hexdigest()
    source='''
import java.io.*;import java.util.*;
public class EnergyCellItemOracle {
 enum Actionable { SIMULATE,MODULATE }
 static class AEComponents { static final String STORED_ENERGY="energy"; }
 static class ItemStack {
  Double stored; double getOrDefault(String key,double fallback){return stored==null?fallback:stored;}
  void set(String key,double v){stored=v;} void remove(String key){stored=null;}
 }
 record EnergyCellBlock(double maxPower,double chargeRate){double getMaxPower(){return maxPower;} double getChargeRate(){return chargeRate;}}
 static class EnergyCellBlockItem {
  EnergyCellBlock block;Object getBlock(){return block;}
  @@METHODS@@
 }
 static int model(double current,double max){float ratio=(float)(current/max);int level=0;for(float t:new float[]{.2f,.4f,.6f,.8f}){if(ratio<t)break;level++;}return level;}
 public static void main(String[] args)throws Exception{
  var input=new BufferedReader(new InputStreamReader(System.in));var out=new StringBuilder();String line;
  while((line=input.readLine())!=null){var row=new StringTokenizer(line);int kind=Integer.parseInt(row.nextToken());boolean present=Integer.parseInt(row.nextToken())!=0;
   double stored=Double.longBitsToDouble(Long.parseUnsignedLong(row.nextToken()));boolean extract=Integer.parseInt(row.nextToken())!=0;
   double amount=Double.longBitsToDouble(Long.parseUnsignedLong(row.nextToken()));var mode=Integer.parseInt(row.nextToken())!=0?Actionable.MODULATE:Actionable.SIMULATE;
   var item=new EnergyCellBlockItem();item.block=new EnergyCellBlock(kind==0?200000:1600000,kind==0?800:1600);var stack=new ItemStack();stack.stored=present?stored:null;
   double result=extract?item.extractAEPower(stack,amount,mode):item.injectAEPower(stack,amount,mode);
   out.append(Long.toUnsignedString(Double.doubleToLongBits(result))).append(' ').append(Long.toUnsignedString(Double.doubleToLongBits(item.getAECurrentPower(stack)))).append(' ')
      .append(stack.stored!=null?1:0).append(' ').append(model(item.getAECurrentPower(stack),item.getAEMaxPower(stack))).append(' ').append((int)item.getChargeRate(stack)).append('\\n');
  }System.out.print(out);
 }
}
'''.replace('@@METHODS@@','\n'.join(methods.values()))
    java=WORK/'EnergyCellItemOracle.java';java.write_text(source);classes=WORK/'classes';classes.mkdir(exist_ok=True)
    subprocess.run([binary('javac'),'--release','21','-d',str(classes),str(java)],check=True)
    rng=random.Random(0xAE2CE11A);rows=[]
    edges=[0.,-0.,0.000001,0.00001,math.nextafter(.00001,0),.00005,.0001,-1.,math.inf,-math.inf,math.nan]
    for i in range(args.cases):
        kind=i%2;maximum=[200000.,1600000.][kind]
        stored=rng.choice([*edges,maximum-1,maximum,maximum+1,*[maximum*t for t in thresholds]]) if i%3==0 else rng.uniform(-maximum,maximum*2)
        amount=rng.choice([*edges,1.,800.,1600.,maximum*2]) if i%3==0 else rng.uniform(-maximum,maximum*2)
        rows.append([kind,int(i%11!=0),bits(stored),rng.randrange(2),bits(amount),rng.randrange(2)])
    data=''.join(' '.join(map(str,row))+'\n' for row in rows).encode();(WORK/'input.txt').write_bytes(data)
    java_run=subprocess.run([binary('java'),'-cp',str(classes),'EnergyCellItemOracle'],input=data,capture_output=True,check=True)
    rust_run=subprocess.run(['cargo','run','--locked','--quiet','--example','energy_cell_item_oracle'],input=data,capture_output=True,cwd=ROOT,check=True)
    (WORK/'java.txt').write_bytes(java_run.stdout);(WORK/'rust.txt').write_bytes(rust_run.stdout)
    if java_run.stdout!=rust_run.stdout:
        for i,(a,b) in enumerate(zip(java_run.stdout.splitlines(),rust_run.stdout.splitlines())):
            if a!=b:raise AssertionError(f'case {i} {rows[i]} Java {a!r} Rust {b!r}')
        raise AssertionError('output lengths differ')
    report={'passed':True,'operations':len(rows),'revision':upstream.LOCK['revision'],'input_sha256':hashlib.sha256(data).hexdigest(),
            'source_sha256':{FILE:hashlib.sha256(text.encode()).hexdigest(),'core/definitions/AEBlocks.java':hashlib.sha256(definition.encode()).hexdigest(),'init/client/InitItemModelsProperties.java':hashlib.sha256(model_property.encode()).hexdigest()},'verbatim_fragment_sha256':{key:hashlib.sha256(value.encode()).hexdigest() for key,value in methods.items()},
            'minecraft_override_bytecode_sha256':hashlib.sha256(bytecode.encode()).hexdigest(),
            'minecraft_client_sha1':version['downloads']['client']['sha1'],'minecraft_mappings_sha1':version['downloads']['client_mappings']['sha1'],
            'item_override_thresholds':thresholds,'item_model_sha256':model_hashes,'scope':'unchanged EnergyCellBlockItem power methods; component/block-profile stubs, model float threshold semantics from source assets/official bytecode',
            'limits':['minimal component/block host','no placement/drop/tooltip/NeoForge lifecycle','NaN outputs compared canonically as Java doubleToLongBits','not a world oracle']}
    (ROOT/'docs/energy-cell-item-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))


if __name__=='__main__':main()
