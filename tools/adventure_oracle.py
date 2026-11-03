#!/usr/bin/env python3
"""Compare fresh adventure predicates against original Minecraft 1.21.1 classes.

The level is a read-only Java proxy around real original states and block entities.
No Minecraft client/server entry point, installed resources or player world is used.
"""
import argparse
import hashlib
import itertools
import json
import os
from pathlib import Path
import random
import subprocess
import zipfile
import minecraft_reference as reference
ROOT=reference.ROOT
WORK=ROOT/'target/adventure-oracle'

class Raw(str):
    pass

def snbt(value):
    if isinstance(value,Raw):return str(value)
    if isinstance(value,str):return json.dumps(value,ensure_ascii=False)
    if isinstance(value,bool):return '1b' if value else '0b'
    if isinstance(value,list):return '['+','.join(map(snbt,value))+']'
    if isinstance(value,dict):return '{'+','.join(snbt(k)+':'+snbt(v) for k,v in value.items())+'}'
    return str(value)

def cases():
    rows=[];tags={'ae2test:logs':['minecraft:oak_log','minecraft:birch_log'],'ae2test:empty':[]}
    def add(component,block='minecraft:stone',state=None,entity=None):
        row={'component':snbt(component),'block':block,'tags':tags}
        if state is not None:row['state']=state
        if entity is not None:row['entity']=entity
        rows.append(row)
    blocks=['minecraft:stone','minecraft:dirt','minecraft:oak_log','minecraft:oak_stairs','minecraft:chest','minecraft:light']
    selectors=[None,'stone',':stone','minecraft:stone','minecraft:dirt','#ae2test:logs','#ae2test:empty','#ae2test:missing','missing:block',[],['minecraft:stone','minecraft:dirt'],['#ae2test:logs'],1,Raw('[I;]')]
    for selector,block in itertools.product(selectors,blocks):
        predicate={} if selector is None else {'blocks':selector}
        for c in [predicate,{'predicates':[predicate]},{'predicates':[predicate,{'blocks':'minecraft:chest'}]},{'predicates':[predicate],'show_in_tooltip':False}]:add(c,block)
    for c in [Raw('[]'),Raw('1b'),'stone',{}, {'predicates':[]},{'predicates':Raw('[I;]')},{'predicates':1},{'predicates':[1]}, {'predicates':[],'blocks':'minecraft:stone'}]:
        for b in blocks:add(c,b)
    bounds=['north','south','west','east','up','down','NORTH','missing','']
    matchers=bounds+[{}]+[{'min':a,'max':b} for a,b in itertools.product(bounds,bounds)]+[{'min':1},{'max':1},{'min':False},[],1]
    for facing,matcher in itertools.product(['north','south','east','west'],matchers):
        add({'predicates':[{'state':{'facing':matcher}}]},'minecraft:oak_stairs',{'facing':facing})
    for level,matcher in itertools.product([0,7,15],['0','00','+0','7','+07','٠٧','７','15','16','-1','2147483648',{}, {'min':'0','max':'15'},{'min':'7'},{'max':'7'},{'min':'16'},{'min':'١٥'},{'min':7}]):
        add({'predicates':[{'state':{'level':matcher}}]},'minecraft:light',{'level':str(level)})
    for value,matcher in itertools.product(['true','false'],['true','false','TRUE',{}, {'min':'false'},{'max':'false'},{'min':'true','max':'false'}]):
        add({'predicates':[{'state':{'waterlogged':matcher}}]},'minecraft:oak_stairs',{'waterlogged':value})
    empty='{id:"minecraft:chest",Items:[]}'
    filled='{id:"minecraft:chest",Items:[{Slot:0b,id:"minecraft:stone",count:2}],Lock:"door",CustomName:\'{"text":"Box"}\'}'
    masks=[{}, {'id':'minecraft:chest'}, {'id':'minecraft:stone'},{'x':0},{'x':Raw('0b')},{'Items':[]},{'Items':[{}]},{'Items':[{'id':'minecraft:stone'}]},{'Items':[{'id':'minecraft:stone'},{'id':'minecraft:stone'}]},{'Lock':'door'}]
    for entity,mask,as_text in itertools.product([None,empty,filled],masks,[False,True]):
        add({'predicates':[{'nbt':snbt(mask) if as_text else mask}]},'minecraft:chest',entity=entity)
    # Codec fallback and typed optional fields, independent of the world mutation.
    for flag in [True,False,0,1,256,Raw('256b'),Raw('256L'),'no',{},[]]:
        add({'predicates':[{'blocks':'minecraft:dirt'}],'show_in_tooltip':flag})
    rng=random.Random(0xAE2AD1)
    for _ in range(3000):
        b=rng.choice(blocks);key=rng.choice(['facing','level','waterlogged','missing']);matcher=rng.choice(matchers)
        predicate={'state':{key:matcher}}
        if rng.randrange(2):predicate['blocks']=rng.choice(selectors)
        add({'predicates':[predicate]},b)
    atoms=['0','1','1b','1s','1L','0.0f','-0.0f','0.0d','-0.0d','1.0d','"x"','"é"','[I;]','[I;1,2]','[L;1L]','[B;1b]','[]','[1]','[1,1]','[1,2]','{}','{a:1}','{a:1,b:2}','[{a:1}]','[{a:1},{a:1}]']
    for a,b in itertools.product(atoms,atoms):rows.append({'compare_nbt':[a,b]})
    literals=['0','-0','+0','00','01','-01','2147483648','-2147483649','127b','128b','-129b','01b','32768s','9223372036854775808L','0.','1.','01.0','.5','+.5','-.0','1e2','1e2f','1e+2d','1.e-3','-1.e-3D','1e999f','1e999d','NaN','NaNf','Infinityd','true','FALSE','1_000','0x10','[1,2]','[1,2b]','[B;]','[b;]','[B;1b,-1b,]','[B;1]','[I;1,2]','[L;1L,-2L]','[L;1]','[]','[[]]','[[],[1]]','[{},{}]','{a:1,a:2}','{a:1,}','{a:}',"'a\\n'",'"a\\t"',"'a\\\\b'","'a\\\"'",'"helloé"','""',"'é'",'"a:b"']
    for literal in literals:
        for value in ['{x:'+literal+'}',' {x:'+literal+'} ','{x:'+literal+'}x','{x:'+literal+'} {}']:
            rows.append({'parse_snbt':value})
        add({'blocks':'minecraft:chest','nbt':'{x:'+literal+'}'},'minecraft:chest',entity=empty)
    for key in ['','""',"''",'a','"a b"','"é"','a:b']:
        rows.append({'parse_snbt':'{'+key+':1}'})
    for unit in [9,10,11,12,13,0x1c,0x20,0xa0,0x1680,0x2000,0x2007,0x2008,0x2028,0x202f,0x205f,0x3000]:
        rows.append({'parse_snbt':chr(unit)+'{x:1}'+chr(unit)})
    rng=random.Random(1211)
    for _ in range(3000):
        sign=rng.choice(['','+','-']);digits=''.join(rng.choice('0123456789') for _ in range(rng.randrange(0,25)))
        fraction=rng.choice(['','.','.0','.1234567890123456789']);exponent=rng.choice(['','e0','E+2','e-999','e+999','ee1'])
        suffix=rng.choice(['','b','B','s','L','f','F','d','D'])
        rows.append({'parse_snbt':'{x:'+sign+digits+fraction+exponent+suffix+'}'})
    for n in range(65536):rows.append({'integer':[n]})
    for n in ['+0','-0','+2147483647','-2147483648','2147483648','-2147483649','٠٧','１２３',' 1','1 ','+','--1','+١٢٣']:
        rows.append({'integer':n})
    return rows

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--java-home',type=Path,default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None);args=p.parse_args()
    WORK.mkdir(parents=True,exist_ok=True);cp=reference.prepare();classes=reference.WORK/'classes';classes.mkdir(exist_ok=True)
    binary=lambda name:str(args.java_home/'bin'/name) if args.java_home else name
    source=ROOT/'tools/java/MinecraftAdventureReference.java'
    subprocess.run([binary('javac'),'-proc:none','-cp',cp,'-d',str(classes),str(source)],check=True)
    config=WORK/'log4j2.xml';config.write_text('<Configuration status="OFF"><Loggers><Root level="OFF"/></Loggers></Configuration>')
    java=[binary('java'),'-Dlog4j.configurationFile='+str(config),'-cp',cp+os.pathsep+str(classes),'ae2reference.MinecraftAdventureReference']
    def run_java(data=None,extra=()):
        r=subprocess.run([*java,*extra],input=data,capture_output=True,cwd=WORK)
        if r.returncode:raise RuntimeError(r.stderr.decode(errors='replace')[-5000:])
        return r.stdout
    properties=run_java(extra=['--properties']);(WORK/'properties.json').write_bytes(properties)
    inputs=cases();data=''.join(json.dumps(x,ensure_ascii=True)+'\n' for x in inputs).encode();(WORK/'input.jsonl').write_bytes(data)
    ref=run_java(data);(WORK/'java.jsonl').write_bytes(ref);results=[json.loads(s) for s in ref.splitlines()];assert len(results)==len(inputs)
    ports=[];want=[]
    for case,result in zip(inputs,results):
        if 'integer' in case:ports.append(case);want.append({'integer':result.get('integer')})
        elif 'parse_snbt' in case:ports.append(case);want.append({'parse_snbt':result.get('parse_snbt')})
        elif 'compare_nbt' in case:ports.append({'expected':result['expected'],'actual':result['actual']});want.append({'compare_nbt':result['compare_nbt']})
        else:
            ports.append({'input':case,**{k:v for k,v in result.items() if k in ['component_view','nbt_literals','entity_view','state_view']}})
            want.append({k:v for k,v in result.items() if k in ['decoded','show','matches']})
    stdin=''.join(json.dumps(x)+'\n' for x in ports).encode();(WORK/'rust-input.jsonl').write_bytes(stdin)
    run=subprocess.run(['cargo','run','--quiet','--locked','--example','adventure_oracle','--',str(WORK/'properties.json')],cwd=ROOT,input=stdin,capture_output=True)
    if run.returncode:raise RuntimeError(run.stderr.decode(errors='replace')[-5000:])
    (WORK/'rust.jsonl').write_bytes(run.stdout);got=[json.loads(s) for s in run.stdout.splitlines()];assert len(got)==len(want)
    failures=[{'case':inputs[i],'java':a,'rust':b} for i,(a,b) in enumerate(zip(want,got)) if a!=b]
    (WORK/'failures.json').write_text(json.dumps(failures[:100],ensure_ascii=True,indent=2)+'\n')
    if failures:raise AssertionError(str(len(failures))+' mismatches; see target/adventure-oracle/failures.json')
    with zipfile.ZipFile(reference.WORK/'client.jar') as archive:
        class_hashes={name:hashlib.sha256(archive.read(name+'.class')).hexdigest() for name in ['csc','aw','eb','eb$b','eb$c','eb$d','eb$e','dk','uq','dtt','dua','duc','uz','dtg','dcw','dvi']}
    report={'passed':True,'cases':len(inputs),'predicate_cases':sum('component' in x for x in inputs),'nbt_cases':sum('compare_nbt' in x for x in inputs),'snbt_cases':sum('parse_snbt' in x for x in inputs),'integer_cases':sum('integer' in x for x in inputs),'property_schemas':len(json.loads(properties)),
        'minecraft':'1.21.1','metadata_sha1':reference.META_SHA1,'client_sha256':hashlib.sha256((reference.WORK/'client.jar').read_bytes()).hexdigest(),
        'classes_sha256':class_hashes,'reference_helper_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'port_sha256':hashlib.sha256((ROOT/'mixins/ae2/adventure.rs').read_bytes()).hexdigest(),
        'snbt_sha256':hashlib.sha256((ROOT/'mixins/ae2/adventure_snbt.rs').read_bytes()).hexdigest(),'input_sha256':hashlib.sha256(data).hexdigest(),'java_output_sha256':hashlib.sha256(ref).hexdigest(),
        'lookup_reference':'Level.getBlockEntity delegates to LevelChunk with EntityCreationType.IMMEDIATE; source bytecode inspected, not exercised by the proxy-level oracle',
        'scope':'Original unmodified AdventureModePredicate CODEC, BlockPredicate, StatePropertiesPredicate, NbtPredicate, NbtUtils and Java Integer parsing; real original block states and chest block entities',
        'limits':['LevelReader is a read-only proxy; synthetic tag bindings are declared by each case','Predicate instances are fresh per case; mutable component-object cache/lifecycle requires separate evidence','Rust receives original typed input/state/entity views; native component, registry and world bindings require separate game checks','No Minecraft client/server entry point or complete Java/NeoForge world is run']}
    (ROOT/'docs/adventure-oracle.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))

if __name__=='__main__':main()
