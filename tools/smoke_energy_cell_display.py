#!/usr/bin/env python3
"""Owned world: finite Energy Cell fullness, real comparator output and restart."""
import argparse
import json
import math
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

ROOT=Path(__file__).resolve().parents[1]


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl',type=Path,required=True)
    parser.add_argument('--resources',type=Path,required=True)
    parser.add_argument('--game',type=Path,default=ROOT/'target/horizon-cargo/fast/horizon-client')
    args=parser.parse_args()
    parent=ROOT/'target/smoke';parent.mkdir(exist_ok=True)
    work=Path(tempfile.mkdtemp(prefix='energy-cell-display-',dir=parent));game_dir=work/'game';game_dir.mkdir()
    socket=work/'control.json';env={**os.environ,'HORIZON_HOME':str(work/'home'),'HORIZON_NO_AUDIO':'1'}
    transcript=[];processes=[];logs=[];checks=[]
    host=(8,73,7);comparator=(9,73,7);dust=(10,73,7);consumer=(7,73,7)

    def ctl(*command):
        run=subprocess.run([str(args.ctl.resolve()),'--socket',str(socket),'--timeout','25',*map(str,command)],cwd=game_dir,env=env,text=True,capture_output=True,timeout=30)
        data=json.loads(run.stdout);transcript.append({'command':command,'exit':run.returncode,'result':data})
        (work/'commands.json').write_text(json.dumps(transcript,indent=2)+'\n')
        if run.returncode:raise RuntimeError(f'{command}: {data}')
        return data

    def launch(number):
        log=(work/f'game-{number}.log').open('w');logs.append(log)
        process=subprocess.Popen([str(args.game.resolve()),'--world','EnergyCellDisplaySmoke','--creative','--seed','42','--offscreen','--control-socket',str(socket),'--render-distance','3','--width','960','--height','640','--game-dir',str(game_dir),'--resources',str(args.resources.resolve()),'--dev-mod',str(ROOT/'target/content/ae2')],cwd=game_dir,env=env,stdout=log,stderr=subprocess.STDOUT)
        processes.append(process);deadline=time.monotonic()+90
        while time.monotonic()<deadline:
            if process.poll() is not None:raise RuntimeError(f'game exited: {log.name}')
            try:
                ctl('wait-ready')
                assert 'using the default resources' not in Path(log.name).read_text(),log.name
                return process
            except (RuntimeError,subprocess.TimeoutExpired):time.sleep(.1)
        raise RuntimeError(f'not ready: {log.name}')

    def merge(value,pending=False):
        ctl('command',f'data merge block 8 73 7 {{internalCurrentPower:{value}d,neighborChangePending:{1 if pending else 0}b}}')

    def nbt():return '\n'.join(ctl('command','data get block 8 73 7')['chat'])

    def inspect(label,maximum,expected_energy=None):
        ctl('wait',8);ctl('server-hold','on')
        try:
            text=nbt();match=re.search(r'internalCurrentPower:\s*([-+0-9.eE]+)d',text);assert match,text
            amount=float(match[1]);block=ctl('getblock',*host)['block'];wire=ctl('getblock',*dust)['block']
            expected_fullness=math.floor(4*max(0,min(1,amount/maximum+.01)))
            expected_signal=math.floor(amount/maximum*14)+(1 if amount>0 else 0)
            assert f'fullness={expected_fullness}' in block,(label,amount,block)
            assert re.search(r'\bpower=(\d+)',wire) and int(re.search(r'\bpower=(\d+)',wire)[1])==expected_signal,(label,amount,expected_signal,wire)
            assert re.search(r'neighborChangePending:\s*0b',text),text
            if expected_energy is not None:assert amount==expected_energy,(label,amount,expected_energy)
            result={'case':label,'energy':amount,'fullness':expected_fullness,'signal':expected_signal}
            checks.append(result);return result
        finally:ctl('server-hold','off')

    try:
        first=launch(1)
        ctl('tp',8.5,80,4.5);ctl('wait',60)
        ctl('command','fill 4 71 3 13 72 11 stone');ctl('command','fill 4 73 3 13 79 11 air')
        ctl('tp',8.5,73,4.5);ctl('wait',5)
        for name,maximum in [('energy_cell',200000.),('dense_energy_cell',1600000.)]:
            ctl('server-hold','on')
            try:
                for level in range(5):
                    result=ctl('setblock',*host,f'ae2:{name}[fullness={level}]')['block']
                    assert f'fullness={level}' in result,result
            finally:ctl('server-hold','off')
            ctl('setblock',*comparator,'minecraft:comparator[facing=west]');ctl('setblock',*dust,'minecraft:redstone_wire')
            merge(0.0000005)
            inspect(f'{name} sub-MIN_AMOUNT load rounds to zero',maximum,0.)
            # Keep the comparator in place: this tests notifications as well as arithmetic.
            for i,ratio in enumerate([0.,.000001,.01,.239999,.24,.25,.489999,.49,.50,.739999,.74,.75,.989999,.99,1.]):
                value=maximum*ratio;merge(value,pending=i%2==0)
                inspect(f'{name} ratio {ratio}',maximum,value)
        ctl('setblock',*host,'ae2:energy_cell');merge(48200.)
        inspect('before live grid drain across fullness threshold',200000.,48200.)
        ctl('setblock',*consumer,'ae2:controller');ctl('wait',120)
        drained=inspect('controller drain updates fullness without replacing comparator',200000.)
        assert drained['energy']<48000 and drained['fullness']==0,drained
        ctl('setblock',*consumer,'air')
        # Same fullness, different analog level: model updates alone cannot notify this.
        merge(20000.);a=inspect('same fullness signal 2',200000.,20000.)
        merge(40000.);b=inspect('same fullness signal 3',200000.,40000.)
        assert a['fullness']==b['fullness']==0 and a['signal']!=b['signal']
        # Creative is a single cube, not a full finite cell nor a redstone inventory.
        ctl('setblock',*host,'ae2:creative_energy_cell');ctl('wait',8)
        creative=ctl('getblock',*host)['block'];assert '[' not in creative,creative
        wire=ctl('getblock',*dust)['block'];assert 'power=0' in wire,wire
        checks.append('creative has one property-free state and no analog output')
        # Vanilla container output is unaffected by the new LevelAccess hook.
        ctl('setblock',*host,'minecraft:barrel')
        ctl('command','data merge block 8 73 7 {Items:[{Slot:0b,id:"minecraft:stone",count:64}]}')
        # Rebuild the vanilla control circuit after seeding its inventory, as
        # in smoke_inventory_signal. AE2 notification cases above all retain
        # the same comparator and dust throughout each cell's matrix.
        ctl('setblock',*comparator,'air');ctl('setblock',*comparator,'minecraft:comparator[facing=west]');ctl('setblock',*dust,'air');ctl('setblock',*dust,'minecraft:redstone_wire');ctl('wait',8)
        assert 'power=1' in ctl('getblock',*dust)['block']
        checks.append('vanilla barrel analog output remains one')
        ctl('setblock',*host,'ae2:dense_energy_cell');merge(800000.,True)
        saved=inspect('before restart dense half full',1600000.,800000.)
        ctl('setblock',6,73,7,'ae2:energy_cell[fullness=0]')
        ctl('command','data merge block 6 73 7 {internalCurrentPower:200000.0d}')
        ctl('setblock',6,73,9,'ae2:energy_cell[fullness=0]')
        ctl('command','data merge block 6 73 9 {internalCurrentPower:100000.0d}')
        ctl('tp',8.5,73,4.5);ctl('look-at',7.5,73.5,7.5);ctl('key','f1','down');ctl('key','f1','up');ctl('wait',8)
        ctl('screenshot',str(work/'energy-cell-levels.png'))
        ctl('save');first.terminate();first.wait(timeout=10);socket.unlink(missing_ok=True)
        launch(2);ctl('tp',8.5,73,4.5);ctl('wait',20)
        restored=inspect('after restart dense half full',1600000.,800000.)
        assert saved['fullness']==restored['fullness']==2 and saved['signal']==restored['signal']==8
        report={'passed':True,'checks':checks,'logs':str(work),
                'scope':'native finite cell states, real comparator/dust, ticking updates, live Controller drain and restart',
                'limits':['full source TickManager/event/readiness lifecycle','item placement/drop energy components','Java/NeoForge world oracle and complete rendering/material parity']}
        (work/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:process.kill();process.wait(timeout=10)
        for log in logs:log.close()
        socket.unlink(missing_ok=True)


if __name__=='__main__':main()
