#!/usr/bin/env python3
"""Isolated AE2 adventure permissions, actual hands and client/server results check.

Uses real right clicks, two hands and held server packets to observe prediction.
Full multipart behavior and external multiplayer remain separate.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--ctl', type=Path, required=True)
    p.add_argument('--resources', type=Path, required=True)
    p.add_argument('--game', type=Path, default=ROOT/'target/horizon-cargo/fast/horizon-client')
    args = p.parse_args()
    parent = ROOT/'target/smoke'; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='adventure-', dir=parent))
    game_dir = work/'game'; game_dir.mkdir()
    socket = work/'control.json'
    env = {**os.environ, 'HORIZON_HOME': str(work/'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []; processes = []; logs = []; checks = []
    def ctl(*command, allow_fail=False):
        run = subprocess.run([str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '25', *map(str, command)], cwd=game_dir, env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout); transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work/'commands.json').write_text(json.dumps(transcript, indent=2)+'\n')
        if run.returncode and not allow_fail:
            raise RuntimeError(f'{command}: {data}')
        return data
    def launch(number):
        log = (work/f'game-{number}.log').open('w'); logs.append(log)
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'AdventureSmoke', '--creative', '--seed', '42', '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640', '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(ROOT/'target/content/ae2')], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process); deadline = time.monotonic()+100
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f'client exited: {log.name}')
            try:
                ctl('wait-ready')
                if 'using the default resources' in Path(log.name).read_text():
                    raise AssertionError('mod content fallback')
                return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(.1)
        raise RuntimeError('client not ready')
    def block(pos, name):
        result = ctl('setblock', *pos, name)
        expected = name.split('[')[0]
        if ':' not in expected:
            expected = 'minecraft:'+expected
        assert result['block'].startswith(expected), result
    def inventory():
        inv=ctl('inventory');return [slot for slot in inv['hotbar']+inv['main'] if slot.get('item')]
    anchor=(6,74,8);north=(6,74,7);south=(6,74,9)
    def aim():
        ctl('tp',6.5,73,5.5);ctl('wait',4);ctl('look-at',6.5,74.5,8.5);ctl('wait',4)
        hit=ctl('looking-at');assert hit.get('kind')=='block',hit
    def clear_targets():
        for p in [north,south]:block(p,'air')
        ctl('wait',4)
    def count(item):return sum(s['count'] for s in inventory() if s['item']==item)
    def swings():return ctl('state')['swings']
    def component(value):return 'minecraft:can_place_on='+value
    def main_stack(value=None,item='ae2:white_glass_cable'):
        suffix='['+component(value)+']' if value is not None else ''
        ctl('command','item replace entity @s hotbar.0 with '+item+suffix+' 3');ctl('select',1);ctl('wait',4)
    def secondary(value=None,item='minecraft:shield'):
        suffix='['+component(value)+']' if value is not None else ''
        ctl('command','item replace entity @s weapon.offhand with '+item+suffix+(' 3' if item.startswith('ae2:') else ' 1'));ctl('wait',4)
    def check_gate(label,value,allowed,anchor_block='stone',offhand_cable=False,sneak=False):
        ctl('mouse','right','up');clear_targets();block(anchor,anchor_block);aim()
        if offhand_cable:
            main_stack(None,'minecraft:stick');secondary(value,'ae2:white_glass_cable')
        else:
            main_stack(value);secondary()
        before=swings();ctl('server-hold','on')
        try:
            if sneak:ctl('key','shift','down')
            ctl('mouse','right','down')
            # A successful preview changes the block under the crosshair.
            # Release before inspection so Minecraft's held-click repeat does
            # not start a second use against that newly predicted cable.
            if allowed:ctl('mouse','right','up')
            ctl('wait',2)
            state=ctl('state');preview=ctl('block',*north)
            assert preview['block'].startswith('ae2:cable_bus') == allowed,(label,preview,state)
            assert ctl('getblock',*north)['block']=='minecraft:air',(label,'server mutated while held')
            assert count('ae2:white_glass_cable')==(0 if offhand_cable else 3),(label,inventory())
            if offhand_cable:assert ctl('status')['offhand']['count']==3,(label,ctl('status'))
            assert swings()==before+(1 if allowed else 0),(label,state)
            if not allowed and not offhand_cable:
                assert state['using'] is not None,(label,state)
            else:assert state['using'] is None,(label,state)
        finally:
            ctl('mouse','right','up')
            ctl('server-hold','off')
        ctl('wait',10)
        if sneak:ctl('key','shift','up')
        actual=ctl('block',*north);inv=ctl('inventory')
        assert actual['block'].startswith('ae2:cable_bus') == allowed,(label,actual)
        assert swings()==before+(1 if allowed else 0),(label,ctl('state'))
        if offhand_cable:
            assert ctl('status')['offhand']['count']==(2 if allowed else 3),(label,inv)
            assert count('minecraft:stick')==3,(label,inv)
        else:
            assert count('ae2:white_glass_cable')==(2 if allowed else 3),(label,inv)
            assert ctl('status')['offhand']['count']==1,(label,ctl('status'))
        checks.append({'case':label,'allowed':allowed,'offhand':offhand_cable})
        print('PASS: '+label,file=__import__('sys').stderr,flush=True)
    try:
        launch(1);ctl('tp',8.5,82,3.5);ctl('wait',60)
        ctl('command','fill 3 71 2 14 72 13 stone');ctl('command','fill 3 73 2 14 82 13 air')
        ctl('command','gamemode adventure')
        check_gate('missing component is PASS and permits the offhand shield',None,False)
        check_gate('full predicate permits stone','{predicates:[{blocks:"minecraft:stone"}]}',True)
        check_gate('full predicate denies dirt selector on stone','{predicates:[{blocks:"minecraft:dirt"}]}',False)
        check_gate('simple selector permits stone','{blocks:"minecraft:stone"}',True)
        check_gate('source empty-full fallback to simple unconditional','{predicates:[]}',True)
        check_gate('direct empty block list matches nothing','{blocks:[]}',False)
        check_gate('tag selector permits log','{predicates:[{blocks:"#minecraft:logs"}]}',True,'oak_log')
        check_gate('tag selector denies stone','{predicates:[{blocks:"#minecraft:logs"}]}',False)
        check_gate('typed enum range uses Java ordinal','{state:{facing:{min:"north",max:"west"}}}',True,'oak_stairs[facing=south]')
        check_gate('enum out of horizontal property domain fails','{state:{facing:{min:"up"}}}',False,'oak_stairs[facing=south]')
        check_gate('enum range lower bound rejects north','{state:{facing:{min:"south"}}}',False,'oak_stairs[facing=north]')
        check_gate('integer Unicode lower bound','{state:{fullness:{min:"０",max:"٤"}}}',True,'ae2:energy_cell')
        check_gate('integer out of property domain is rejected','{state:{fullness:{min:"5"}}}',False,'ae2:energy_cell')
        check_gate('offhand permitted stack independent of main hand','{predicates:[{blocks:"minecraft:stone"}]}',True,offhand_cable=True)
        check_gate('offhand missing permission is PASS',None,False,offhand_cable=True)
        check_gate('offhand denied full selector','{predicates:[{blocks:"minecraft:dirt"}]}',False,offhand_cable=True)
        check_gate('compound AE2 block entity NBT creates the client instance','{blocks:"ae2:energy_cell",nbt:{id:"ae2:energy_cell"}}',True,'ae2:energy_cell')
        check_gate('compound AE2 block entity mismatch','{blocks:"ae2:energy_cell",nbt:{id:"ae2:drive"}}',False,'ae2:energy_cell')
        check_gate('AE2 SNBT string block entity NBT',"{blocks:'ae2:energy_cell',nbt:'{id:\"ae2:energy_cell\"}'}",True,'ae2:energy_cell')
        check_gate('compound vanilla block entity NBT','{blocks:"minecraft:chest",nbt:{id:"minecraft:chest"}}',True,'minecraft:chest',sneak=True)
        check_gate('compound vanilla block entity mismatch','{blocks:"minecraft:chest",nbt:{id:"minecraft:stone"}}',False,'minecraft:chest',sneak=True)
        check_gate('original SNBT string block entity NBT',"{blocks:'minecraft:chest',nbt:'{id:\"minecraft:chest\"}'}",True,'minecraft:chest',sneak=True)
        check_gate('original SNBT rejects trailing data',"{blocks:'minecraft:chest',nbt:'{id:\"minecraft:chest\"}junk'}",False,'minecraft:chest',sneak=True)
        # Block use still has priority even when ItemStack cannot place a part.
        clear_targets();block(anchor,'minecraft:chest');aim();main_stack();ctl('command','item replace entity @s weapon.offhand with minecraft:air')
        ctl('mouse','right','down');ctl('mouse','right','up');ctl('wait',6)
        assert ctl('state')['screen']!='playing',ctl('state')
        assert count('ae2:white_glass_cable')==3,inventory()
        assert ctl('getblock',*north)['block']=='minecraft:air'
        checks.append({'case':'chest block reaction precedes disallowed part stack','allowed':False})
        report={'passed':True,'checks':checks,'logs':str(work),'scope':'Native fresh adventure gate and actual hand, client prediction and server authority','limits':['Fresh predicate matching only; mutable component cache, copy identity and reload lifecycle remain open','Only existing native BE codecs and registered client constructors are used; complete original BE fields, packet/default and component semantics remain open','Property facts cover original vanilla blocks and seven supported AE2 hosts; new/addon domains and missing native spin states remain open','Integrated native client/server; independent external multiplayer and full permissions/security are not covered']}
        report['source_sha256']={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in ['mixins/ae2/adventure.rs','mixins/ae2/adventure_snbt.rs','mixins/ae2/adventure_native.rs','mixins/ae2/cable_items.rs','mixins/ae2/cable_prediction.rs','mixins/ae2/horizon-mixin.toml']}
        report['binary_sha256']=hashlib.sha256(args.game.read_bytes()).hexdigest()
        report['property_facts_sha256']=hashlib.sha256((ROOT/'target/content/ae2/data/ae2/source-property-domains.json').read_bytes()).hexdigest()
        (work/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:
        for process in processes:
            if process.poll() is None:
                try:ctl('quit')
                except Exception:process.terminate()
                try:process.wait(timeout=12)
                except subprocess.TimeoutExpired:process.kill();process.wait()
        for log in logs:log.close()

if __name__=='__main__':main()
