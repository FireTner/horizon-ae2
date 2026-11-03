#!/usr/bin/env python3
"""Build AE2-owned source property facts from the verified original Java reference.

Only generated metadata is added to the mod. Horizon resources and the original
Minecraft/AE2 jars are read-only. Java is a build/check dependency, never runtime.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import minecraft_reference as reference
import upstream
ROOT=reference.ROOT
JAVA=ROOT/'tools/java/MinecraftAdventureReference.java'
OUT=reference.WORK/'source-property-domains.json'


def source_domains():
    def enum(values):return {'kind':'enum','values':{v:i for i,v in enumerate(values)}}
    def integer(end):return {'kind':'integer','values':{str(i):i for i in range(end+1)}}
    boolean={'kind':'boolean','values':{'false':0,'true':1}}
    facing=enum(['down','up','north','south','west','east'])
    orientation={'facing':facing,'spin':integer(3)}
    return {
        'ae2:cable_bus':{'light_level':integer(15),'waterlogged':boolean},
        'ae2:controller':{'state':enum(['offline','online','conflicted']),'type':enum(['block','column_x','column_y','column_z','inside_a','inside_b'])},
        'ae2:energy_cell':{'fullness':integer(4)},'ae2:dense_energy_cell':{'fullness':integer(4)},'ae2:creative_energy_cell':{},
        'ae2:charger':orientation,'ae2:drive':orientation,
    }


def build(java_home=None):
    upstream.verify(upstream.SOURCE)
    cp=reference.prepare();classes=reference.WORK/'classes';classes.mkdir(exist_ok=True)
    binary=lambda name:str(java_home/'bin'/name) if java_home else name
    version=subprocess.run([binary('java'),'-version'],capture_output=True,text=True,check=True)
    if 'version "21' not in version.stderr+version.stdout:
        raise ValueError('source property preparation requires JDK 21')
    provenance={'minecraft':'1.21.1','metadata_sha1':reference.META_SHA1,'client_sha256':hashlib.sha256((reference.WORK/'client.jar').read_bytes()).hexdigest(),
                'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'reference_helper_sha256':hashlib.sha256(JAVA.read_bytes()).hexdigest(),'upstream':upstream.LOCK['revision']}
    paths=['block/networking/CableBusBlock.java','block/networking/ControllerBlock.java','block/networking/EnergyCellBlock.java','block/misc/ChargerBlock.java','block/storage/DriveBlock.java','block/AEBaseBlock.java','api/orientation/IOrientationStrategy.java','api/orientation/OrientationStrategies.java']
    provenance['ae2_sha256']={p:hashlib.sha256((upstream.SOURCE/'src/main/java/appeng'/p).read_bytes()).hexdigest() for p in paths}
    if OUT.exists():
        old=json.loads(OUT.read_text())
        if old.get('provenance')==provenance:return old
    run=subprocess.run([binary('javac'),'-proc:none','-cp',cp,'-d',str(classes),str(JAVA)],capture_output=True,text=True)
    if run.returncode:raise RuntimeError(run.stderr[-5000:])
    quiet=reference.WORK/'log4j2-properties.xml';quiet.write_text('<Configuration status="OFF"><Loggers><Root level="OFF"/></Loggers></Configuration>')
    run=subprocess.run([binary('java'),'-Dlog4j.configurationFile='+str(quiet),'-cp',cp+os.pathsep+str(classes),'ae2reference.MinecraftAdventureReference','--properties'],cwd=reference.WORK,capture_output=True,text=True)
    if run.returncode:raise RuntimeError(run.stderr[-5000:])
    blocks=json.loads(run.stdout);assert len(blocks)==1060
    blocks.update(source_domains())
    result=json.loads(json.dumps({'schema':1,'provenance':provenance,'blocks':blocks},sort_keys=True))
    OUT.write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--java-home',type=Path,default=Path(os.environ['JAVA_HOME']) if 'JAVA_HOME' in os.environ else None);a=p.parse_args()
    result=build(a.java_home);print(json.dumps({'file':str(OUT),'blocks':len(result['blocks']),'sha256':hashlib.sha256(OUT.read_bytes()).hexdigest()},indent=2))
