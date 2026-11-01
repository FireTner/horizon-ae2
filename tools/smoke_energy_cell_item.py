#!/usr/bin/env python3
"""Owned world: Energy Cell charging, real placement/drop and render-only models."""
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
    args=parser.parse_args();parent=ROOT/'target/smoke';parent.mkdir(exist_ok=True)
    work=Path(tempfile.mkdtemp(prefix='energy-cell-item-',dir=parent));game_dir=work/'game';game_dir.mkdir()
    socket=work/'control.json';env={**os.environ,'HORIZON_HOME':str(work/'home'),'HORIZON_NO_AUDIO':'1'}
    transcript=[];processes=[];logs=[];checks=[]
    charger=(8,73,7);back=(8,73,8);placed=(6,73,7);barrel=(11,73,7)

    def ctl(*command):
        run=subprocess.run([str(args.ctl.resolve()),'--socket',str(socket),'--timeout','30',*map(str,command)],cwd=game_dir,env=env,text=True,capture_output=True,timeout=40)
        data=json.loads(run.stdout);transcript.append({'command':command,'exit':run.returncode,'result':data})
        (work/'commands.json').write_text(json.dumps(transcript,indent=2)+'\n')
        if run.returncode:raise RuntimeError(f'{command}: {data}')
        if command[0]=='command':assert not any('Unknown' in line or '<--[HERE]' in line for line in data.get('chat',[])),data
        return data

    def launch(number):
        log=(work/f'game-{number}.log').open('w');logs.append(log)
        process=subprocess.Popen([str(args.game.resolve()),'--world','EnergyCellItemSmoke','--creative','--seed','42','--offscreen','--control-socket',str(socket),'--render-distance','3','--width','960','--height','640','--game-dir',str(game_dir),'--resources',str(args.resources.resolve()),'--dev-mod',str(ROOT/'target/content/ae2')],cwd=game_dir,env=env,stdout=log,stderr=subprocess.STDOUT)
        processes.append(process);deadline=time.monotonic()+90
        while time.monotonic()<deadline:
            if process.poll() is not None:raise RuntimeError(f'game exited: {log.name}')
            try:
                ctl('wait-ready');assert 'using the default resources' not in Path(log.name).read_text(),log.name;return process
            except (RuntimeError,subprocess.TimeoutExpired):time.sleep(.1)
        raise RuntimeError(f'not ready: {log.name}')

    def nbt(pos):return '\n'.join(ctl('command',f"data get block {' '.join(map(str,pos))}")['chat'])
    def number(text,key):
        match=re.search(r'(?:"'+re.escape(key)+r'"|'+re.escape(key)+r'):\s*([-+0-9.eE]+)d',text)
        return float(match[1]) if match else 0.0
    def merge(pos,text):ctl('command',f"data merge block {' '.join(map(str,pos))} {text}")
    def items():
        inv=ctl('inventory');return [slot for slot in inv['hotbar']+inv['main'] if slot.get('item')]
    def find(item):return [slot for slot in items() if slot['item']==item]
    def choose(item):
        entries=find(item);assert entries,items()
        slot=entries[0]['slot'];assert 0<=slot<=8,entries
        ctl('select',slot+1)
    def give(item,energy=None,count=1,extra=''):
        components=([f'ae2:stored_energy={float(energy)}d'] if energy is not None else [])+([extra] if extra else [])
        ctl('command',f'item replace entity @s hotbar.0 with {item}'+('['+','.join(components)+']' if components else '')+f' {count}')
        ctl('select',1);ctl('wait',3)
    def assert_item(item,energy,count=1):
        stacks=find(item);assert sum(s['count'] for s in stacks)==count,stacks
        for s in stacks:
            text=s.get('components','')
            assert number(text,'ae2:stored_energy')==energy,(energy,s)
            if energy==0:assert 'ae2:stored_energy' not in text,s
            assert 'minecraft:block_state' not in text,s
        return stacks
    def put_and_break(item,energy,expected,mode='player',count=1):
        ctl('command','clear @s');ctl('tp',8.5,73,4.5)
        give(item,energy,count);ctl('place',*placed);ctl('wait',8)
        text=nbt(placed);assert number(text,'internalCurrentPower')==expected,text
        remaining=sum(s['count'] for s in find(item));assert remaining==count-1,(remaining,count)
        if mode=='player':ctl('break',*placed)
        else:ctl('command',f"setblock {' '.join(map(str,placed))} air destroy")
        assert ctl('getblock',*placed)['block']=='minecraft:air'
        ctl('tp',6.5,73,7.5);ctl('wait',25)
        stacks=assert_item(item,expected,count)
        checks.append({'case':f'{mode} placement/drop {item}','input_energy':energy,'block_energy':expected,'count':count,'returned':stacks})

    try:
        first=launch(1);ctl('tp',8.5,82,4.5);ctl('wait',60)
        ctl('command','fill 4 71 3 14 72 11 stone');ctl('command','fill 4 73 3 14 80 11 air')
        ctl('tp',8.5,73,4.5);ctl('wait',5)
        for item,maximum in [('ae2:energy_cell',200000.),('ae2:dense_energy_cell',1600000.)]:
            ctl('server-hold','on')
            try:
                for pos in [charger,back]:ctl('setblock',*pos,'air')
                ctl('setblock',*charger,'ae2:charger[facing=north]');merge(charger,'{internalCurrentPower:1600.0d}')
                ctl('setblock',*back,'ae2:energy_cell');merge(back,'{internalCurrentPower:1000.0d}')
            finally:ctl('server-hold','off')
            ctl('command','clear @s')
            give(item,maximum-10,extra='ae2:energy_capacity=1.0d,ae2:upgrades=[{slot:0,item:{id:"ae2:energy_card",count:1}}],minecraft:custom_data={cell_marker:123}')
            ctl('interact',*charger);ctl('select',9);ctl('wait',40)
            ctl('server-hold','on')
            try:
                data=nbt(charger);cell=nbt(back)
                assert number(data,'ae2:stored_energy')==maximum,data
                total=number(data,'internalCurrentPower')+number(data,'ae2:stored_energy')+number(cell,'internalCurrentPower')+number(data,'e')+number(cell,'e')
                assert total==maximum-10+2600,(total,data,cell)
                assert '"ae2:energy_capacity": 1.0d' in data and 'cell_marker: 123' in data,data
                checks.append({'case':f'{item} fixed capacity/overflow/conservation','total':total,'maximum':maximum})
            finally:ctl('server-hold','off')
            ctl('interact',*charger);ctl('tp',8.5,73,6.5);ctl('wait',25)
            stacks=assert_item(item,maximum);assert re.search(r'cell_marker:\s*123\b',stacks[0].get('components','')),stacks
            # Use the actually charged item in survival, not a separately generated replacement.
            ctl('command','gamemode survival');choose(item);ctl('place',*placed);ctl('wait',8)
            assert number(nbt(placed),'internalCurrentPower')==maximum
            assert not find(item),items()
            ctl('break',*placed);ctl('tp',6.5,73,7.5);ctl('wait',25);assert_item(item,maximum)
            checks.append(f'{item} Charger -> player placement -> survival drop preserves energy')
            ctl('command','gamemode creative');ctl('tp',8.5,73,4.5)
        for pos in [charger,back]:ctl('setblock',*pos,'air')
        ctl('command','gamemode survival')
        for value,expected in [(None,0.),(.0000005,0.),(.000001,.000001),(200001.,200000.)]:
            put_and_break('ae2:energy_cell',value,expected)
        put_and_break('ae2:dense_energy_cell',123456.,123456.,'destroy')
        put_and_break('ae2:energy_cell',1000.,1000.,count=2)
        ctl('command','gamemode creative');ctl('command','clear @s');ctl('tp',10.5,73,4.5)
        # Render all ten item models. The block-state selector must never leak
        # its temporary fullness into real stack components or saved hashes.
        ctl('setblock',*barrel,'minecraft:barrel')
        stacks=[]
        for index,(name,maximum) in enumerate([('energy_cell',200000.),('dense_energy_cell',1600000.)]):
            for level in range(5):stacks.append(f'{{Slot:{index*5+level}b,id:"ae2:{name}",count:1,components:{{"ae2:stored_energy":{maximum*level/5}d,"ae2:energy_capacity":1.0d,"minecraft:custom_data":{{model_marker:77}}}}}}')
        merge(barrel,'{Items:['+','.join(stacks)+']}')
        before=nbt(barrel);assert 'minecraft:block_state' not in before
        ctl('look-at',11.5,73.5,7.5);ctl('interact',*barrel);ctl('wait',10)
        ctl('screenshot',str(work/'energy-cell-item-models.png'));ctl('key','escape','tap');ctl('wait',5)
        assert nbt(barrel)==before
        checks.append('rendering all ten item models preserves original energy/capacity/custom components')
        ctl('command','clear @s');ctl('command','gamemode survival')
        ctl('tp',8.5,73,7.5);give('ae2:dense_energy_cell',333333.)
        persisted=(6,73,9);ctl('place',*persisted);ctl('wait',8)
        assert number(nbt(persisted),'internalCurrentPower')==333333.
        ctl('save');first.terminate();first.wait(timeout=10);socket.unlink(missing_ok=True)
        launch(2);ctl('tp',10.5,73,4.5);ctl('wait',20)
        assert nbt(barrel)==before
        assert number(nbt(persisted),'internalCurrentPower')==333333.
        ctl('tp',8.5,73,7.5);ctl('wait',8);ctl('break',*persisted);assert ctl('getblock',*persisted)['block']=='minecraft:air';ctl('tp',6.5,73,9.5);ctl('wait',25)
        assert_item('ae2:dense_energy_cell',333333.)
        checks.append('placed charged dense cell survives restart and then drops its exact energy')
        checks.append('all ten componentful energy-cell items survive process restart without render projection')
        report={'passed':True,'logs':str(work),'checks':checks,'limits':['complete tooltip formatting/base naming/ownership settings','all explosion/unloaded/saved-BE/pick-block/loot-command paths','full Java/NeoForge lifecycle and external multiplayer','nonfinite/malformed native power component recovery']}
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
