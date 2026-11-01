#!/usr/bin/env python3
"""Owned Controller world: real state registry, energy, topology/geometry and restart."""
import argparse
import json
import math
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--game', type=Path, default=ROOT / 'target/horizon-cargo/fast/horizon-client')
    args = parser.parse_args()
    parent = ROOT/'target/smoke'; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='controller-', dir=parent))
    game_dir = work/'game'; game_dir.mkdir()
    socket = work/'control.json'
    env = {**os.environ, 'HORIZON_HOME': str(work/'home'), 'HORIZON_NO_AUDIO':'1'}
    transcript=[]; processes=[]; logs=[]; checks=[]; group=[]
    center=(8,74,8)

    def ctl(*command):
        run=subprocess.run([str(args.ctl.resolve()),'--socket',str(socket),'--timeout','25',*map(str,command)],
                           cwd=game_dir,env=env,text=True,capture_output=True,timeout=30)
        data=json.loads(run.stdout);transcript.append({'command':command,'exit':run.returncode,'result':data})
        (work/'commands.json').write_text(json.dumps(transcript,indent=2)+'\n')
        if run.returncode:raise RuntimeError(f'{command}: {data}')
        return data

    def launch(number):
        log=(work/f'game-{number}.log').open('w');logs.append(log)
        process=subprocess.Popen([str(args.game.resolve()),'--world','ControllerSmoke','--creative','--seed','42',
            '--offscreen','--control-socket',str(socket),'--render-distance','3','--width','960','--height','640',
            '--game-dir',str(game_dir),'--resources',str(args.resources.resolve()),'--dev-mod',str(ROOT/'target/content/ae2')],
            cwd=game_dir,env=env,stdout=log,stderr=subprocess.STDOUT)
        processes.append(process);deadline=time.monotonic()+90
        while time.monotonic()<deadline:
            if process.poll() is not None:raise RuntimeError(f'game exited: {log.name}')
            try:
                ctl('wait-ready')
                if 'using the default resources' in Path(log.name).read_text():
                    raise AssertionError(f'content resource fallback: {log.name}')
                return process
            except (RuntimeError,subprocess.TimeoutExpired):time.sleep(.1)
        raise RuntimeError(f'game not ready: {log.name}')

    def block(pos,name):
        result=ctl('setblock',*pos,name)
        expected=name.split('[')[0]
        if ':' not in expected:expected='minecraft:'+expected
        assert result['block'].startswith(expected),result

    def merge(pos,value):ctl('command',f"data merge block {' '.join(map(str,pos))} {value}")
    def nbt(pos):return '\n'.join(ctl('command',f"data get block {' '.join(map(str,pos))}")['chat'])
    def energy(pos):
        text=nbt(pos)
        stored=re.search(r'internalCurrentPower:\s*([-+0-9.eE]+)d',text)
        buffer=re.search(r'\be:\s*([-+0-9.eE]+)d',text)
        return (float(stored[1]) if stored else 0)+(float(buffer[1]) if buffer else 0)

    def sample(positions):
        ctl('server-hold','on')
        try:return {'tick':ctl('simulation')['game_time'],'energy':sum(energy(p) for p in positions)}
        finally:ctl('server-hold','off')

    def expect(pos,state,kind=None):
        text=ctl('getblock',*pos)['block']
        assert text.startswith('ae2:controller[') and f'state={state}' in text,(pos,state,text)
        if kind:assert f'type={kind}' in text,(pos,kind,text)
        return text

    def clear():
        ctl('server-hold','on')
        try:
            for pos in group:block(pos,'air')
            group.clear()
        finally:ctl('server-hold','off')
        ctl('wait',4)

    def make(controllers,creative=None,stored=None):
        clear();ctl('server-hold','on')
        try:
            for pos in controllers:block(pos,'ae2:controller');group.append(pos)
            if creative is not None:block(creative,'ae2:creative_energy_cell');group.append(creative)
            if stored is not None:merge(controllers[0],'{internalCurrentPower:'+str(float(stored))+'d}')
        finally:ctl('server-hold','off')

    try:
        first=launch(1)
        ctl('tp',8.5,80,4.5);ctl('wait',70)
        ctl('command','fill 4 71 3 20 72 13 stone');ctl('command','fill 4 73 3 20 80 13 air')
        ctl('tp',8.5,73,4.5);ctl('wait',5)
        ctl('server-hold','on')
        try:
            for state in ['offline','online','conflicted']:
                for kind in ['block','column_x','column_y','column_z','inside_a','inside_b']:
                    block(center,f'ae2:controller[state={state},type={kind}]');expect(center,state,kind)
            group.append(center)
        finally:ctl('server-hold','off')
        checks.append('all original 18 state/type combinations resolve as real block states')
        make([center],stored=1000)
        ctl('wait',40);expect(center,'online','block')
        before=sample([center]);ctl('wait',30);after=sample([center])
        assert math.isclose(before['energy']-after['energy'],3*(after['tick']-before['tick']),abs_tol=1e-8),(before,after)
        checks.append({'three_AE_idle_and_no_refill':{'before':before,'after':after}})
        make([center]);ctl('wait',40);expect(center,'offline','block')
        assert energy(center)==0,nbt(center)
        checks.append('unpowered controller remains offline with no invented energy')
        make([center],stored=10000);ctl('wait',5)
        assert 0<energy(center)<=8000,nbt(center)
        checks.append('public local storage is capped at original 8000 AE')
        # The public Controller store powers a real Charger; total stored+node
        # buffer+private energy loses exactly the recipe and controller idle.
        charger=(8,74,7)
        make([center],stored=5000)
        ctl('server-hold','on')
        try:
            block(charger,'ae2:charger[facing=north]');group.append(charger)
            merge(charger,'{inv:{item0:{id:"ae2:certus_quartz_crystal"}}}')
            before={'tick':ctl('simulation')['game_time'],'energy':energy(center)+energy(charger)}
        finally:ctl('server-hold','off')
        for _ in range(200):
            ctl('wait',10)
            if ctl('container',*charger)['slots']==[{'item':'ae2:charged_certus_quartz_crystal','count':1}]:break
        else:raise AssertionError('Controller-powered Charger never completed')
        after=sample([center,charger]);used=before['energy']-after['energy']
        assert math.isclose(used,1600+3*(after['tick']-before['tick']),abs_tol=1e-7),(before,after,used)
        checks.append({'controller_powers_real_charger':{'before':before,'after':after,'recipe_AE':1600}})
        for kind,offsets,expected in [
            ('block',[],'online'),('column_x',[(-1,0,0),(1,0,0)],'online'),
            ('column_y',[(0,-1,0),(0,1,0)],'online'),('column_z',[(0,0,-1),(0,0,1)],'online'),
            ('inside_a',[(-1,0,0),(1,0,0),(0,-1,0),(0,1,0)],'conflicted'),
            ('inside_b',[(-1,0,0),(1,0,0),(0,-1,0),(0,1,0)],'conflicted'),
        ]:
            pos=(9,74,8) if kind=='inside_b' else center
            controllers=[pos]+[tuple(pos[i]+delta[i] for i in range(3)) for delta in offsets]
            supply=(pos[0]-1,pos[1],pos[2]) if kind in ['block','column_z'] else (pos[0],pos[1],pos[2]+1)
            make(controllers,creative=supply);ctl('wait',40);expect(pos,expected,kind)
            checks.append({'geometry':kind,'state':expected})
        ctl('look-at',9.5,74.5,8.5);ctl('wait',5);ctl('screenshot',str(work/'controller-connected.png'))
        controllers=[(x,74,8) for x in range(8,16)]
        make(controllers,creative=(7,74,8));ctl('wait',40)
        for pos in controllers:expect(pos,'conflicted')
        block(controllers[-1],'air');group.remove(controllers[-1]);ctl('wait',8)
        for pos in controllers[:-1]:expect(pos,'online')
        checks.append('eight-wide conflict recovers to online at original seven-wide bound')
        make([(8,74,8),(10,74,8)],creative=(9,74,8));ctl('wait',40)
        expect((8,74,8),'conflicted');expect((10,74,8),'conflicted')
        checks.append('controllers connected only through another node conflict')
        make([center],stored=5000);ctl('wait',40);expect(center,'online')
        before=sample([center]);ctl('save');ctl('quit');first.wait(timeout=20);launch(2);ctl('wait',40)
        after=sample([center]);expect(center,'online','block')
        assert math.isclose(before['energy']-after['energy'],3*(after['tick']-before['tick']),abs_tol=1e-7),(before,after)
        checks.append({'restart_retains_energy_without_refill':{'before':before,'after':after}})
        ctl('quit')
        report={'passed':True,'checks':checks,'logs':str(work),
                'limits':['Network Status Menu and external energy capabilities','event/readiness/chunk/spatial lifecycle fidelity',
                          'full controller lighting/material rendering parity','Java/NeoForge world oracle and full mod parity']}
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
