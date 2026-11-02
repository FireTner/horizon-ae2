#!/usr/bin/env python3
"""Isolated CableBus CTRL packet ordering, key binding and native Pick Block check.

Drives the integrated client/server through real key and middle-button input.
Complete client prediction, side parts and external multiplayer remain separate.
"""
import argparse
import json
import os
from pathlib import Path
import re
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
    work = Path(tempfile.mkdtemp(prefix='cable-input-', dir=parent))
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
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'CableInputSmoke', '--creative', '--seed', '42', '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640', '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(ROOT/'target/content/ae2')], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
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
    def merge(pos, tag):
        ctl('command', f'data merge block {" ".join(map(str,pos))} {tag}')
    def nbt(pos):
        return '\n'.join(ctl('command', f'data get block {" ".join(map(str,pos))}')['chat'])
    def cable(pos, item):
        block(pos, 'ae2:cable_bus')
        merge(pos, '{hasRedstone:2,cable:{id:"ae2:'+item+'"}}')


    def inventory():
        inv=ctl('inventory');return [slot for slot in inv['hotbar']+inv['main'] if slot.get('item')]
    def give(item,count=1):
        ctl('command','item replace entity @s hotbar.0 with '+item+' '+str(count));ctl('select',1);ctl('wait',2)
    def click(button):
        ctl('mouse',button,'down');ctl('mouse',button,'up')
    anchor=(6,74,8);north=(6,74,7);south=(6,74,9)
    def reset_targets():
        for p in [north,south]:block(p,'air')
    def aim():
        ctl('tp',6.5,73,5.5);ctl('wait',4);ctl('look-at',6.5,74.5,8.5);ctl('wait',4)
        hit=ctl('looking-at');assert hit.get('kind')=='block',hit
    def assert_placed(target,other,item):
        assert 'ae2:cable_bus' in ctl('getblock',*target)['block'],target
        assert ctl('getblock',*other)['block']=='minecraft:air',other
        assert item in nbt(target),nbt(target)
        assert 'ae2:fluix_glass_cable' in nbt(anchor),nbt(anchor)
    try:
        first=launch(1);ctl('tp',8.5,82,3.5);ctl('wait',60)
        ctl('command','fill 3 71 2 14 72 13 stone');ctl('command','fill 3 73 2 14 82 13 air')
        cable(anchor,'fluix_glass_cable');ctl('wait',8)
        item='ae2:red_smart_cable'
        for pressed in [False,True,False,True,False,True]:
            reset_targets();aim();give(item,2)
            ctl('key','ctrl','down' if pressed else 'up')
            click('right');ctl('wait',8)
            assert_placed(south if pressed else north,north if pressed else south,item)
        ctl('key','ctrl','up');checks.append('real CTRL press/release reverses central placement for the acting player')
        # The barrier batches a control packet and the dependent use packet in
        # the server's next drain. Deferred mod delivery would use the old value.
        for _ in range(6):
            reset_targets();aim();give(item,2);ctl('key','ctrl','up');ctl('wait',4)
            ctl('server-hold','on')
            try:
                ctl('key','ctrl','down');click('right');time.sleep(.2)
            finally:ctl('server-hold','off')
            ctl('wait',8);assert_placed(south,north,item);ctl('key','ctrl','up')
        checks.append('six coalesced CTRL/use packet batches observe the new attachment before placement')
        reset_targets();ctl('command','clear @s');aim()
        colors=['white','light_gray','gray','black','lime','yellow','orange','brown','red','pink','magenta','purple','blue','light_blue','cyan','green','fluix']
        kinds=['glass_cable','covered_cable','smart_cable','covered_dense_cable','smart_dense_cable']
        for kind in kinds:
            for color in colors:
                block(anchor,'air');cable(anchor,color+'_'+kind);ctl('wait',4)
                ctl('command','clear @s');aim();click('middle');ctl('wait',6)
                stacks=inventory();assert len(stacks)==1,stacks
                assert stacks[0]['item']=='ae2:'+color+'_'+kind and stacks[0]['count']==1,stacks
                assert not stacks[0].get('components'),stacks
            print('PASS: 17 creative Pick Block results for '+kind,file=__import__('sys').stderr,flush=True)
        checks.append('normal creative Pick Block returns each of all 85 bare part IDs without name/node/host data')
        # Ctrl-Pick adds the host's authoritative block data in the separate
        # vanilla step, rather than copying the part name as a normal item name.
        block(anchor,'air');cable(anchor,'cyan_smart_cable')
        name=json.dumps(json.dumps({"text":"Named center"}))
        merge(anchor,'{cable:{id:"ae2:cyan_smart_cable",customName:'+name+'}}')
        ctl('wait',4);ctl('command','clear @s');aim()
        modifier='ctrl'
        ctl('key',modifier,'down');click('middle');ctl('wait',6);ctl('key',modifier,'up')
        stacks=inventory();assert len(stacks)==1 and stacks[0]['item']=='ae2:cyan_smart_cable',stacks
        assert 'minecraft:block_entity_data' in stacks[0].get('components',''),stacks
        assert 'Named center' in stacks[0].get('components',''),stacks
        assert 'ae2:cable_visual' not in stacks[0].get('components',''),stacks
        checks.append('creative Ctrl-Pick retains original host NBT without transient visual envelope fields')
        ctl('command','clear @s');give('ae2:cyan_smart_cable',7);ctl('command','gamemode survival');aim()
        click('middle');ctl('wait',6)
        stacks=inventory();assert sum(s['count'] for s in stacks if s['item']=='ae2:cyan_smart_cable')==7,stacks
        checks.append('survival Pick Block selects an existing cable stack without creating another')
        ctl('command','clear @s');aim();click('middle');ctl('wait',6);assert not inventory(),inventory()
        checks.append('survival Pick Block cannot manufacture a missing part')
        block(anchor,'air');cable(anchor,'fluix_glass_cable');ctl('wait',4)
        ctl('command','gamemode creative');ctl('key','ctrl','down');ctl('save');ctl('quit');first.wait(timeout=25)
        # Settings are changed only in this fixture's private game directory.
        settings=game_dir/'horizon-mods.txt';old=settings.read_text() if settings.exists() else ''
        lines=[line for line in old.splitlines() if not line.startswith('key.ae2.part_placement_opposite:')]
        settings.write_text('\n'.join(lines)+'\nkey.ae2.part_placement_opposite:key.keyboard.g\n')
        second=launch(2);ctl('wait',8);reset_targets();aim();give(item,2)
        ctl('key','ctrl','down');click('right');ctl('wait',8);ctl('key','ctrl','up');assert_placed(north,south,item)
        reset_targets();aim();give(item,2);ctl('key','g','down');click('right');ctl('wait',8);ctl('key','g','up');assert_placed(south,north,item)
        checks.append('restart clears old CTRL state and honors the configured opposite-placement key instead of hard-coded controls')
        ctl('save');ctl('quit');second.wait(timeout=25)
        # Vanilla sprint and AE2's modifier are independent physical bindings.
        lines=[line for line in settings.read_text().splitlines() if not line.startswith('key.ae2.part_placement_opposite:')]
        settings.write_text('\n'.join(lines)+'\nkey.ae2.part_placement_opposite:key.keyboard.left.control\n')
        options=game_dir/'options.txt';lines=[line for line in (options.read_text() if options.exists() else '').splitlines() if not line.startswith('key_key.sprint:')]
        options.write_text('\n'.join(lines)+'\nkey_key.sprint:key.keyboard.g\n')
        launch(3);ctl('wait',8);reset_targets();aim();give(item,2)
        # The ctl "down" helper also inserts the translated action key into
        # physical_keys. Its repeat path preserves only the physical key, as
        # the window keyboard handler does; use that path for this remap case.
        ctl('key','g','repeat');click('right');ctl('wait',8);ctl('key','g','up');assert_placed(north,south,item)
        reset_targets();aim();give(item,2);ctl('key','ctrl','down');click('right');ctl('wait',8);ctl('key','ctrl','up');assert_placed(south,north,item)
        checks.append('rebinding vanilla sprint does not change the physical AE2 opposite-placement modifier')
        ctl('quit')
        report={'passed':True,'checks':checks,'logs':str(work),'limits':[
            'Integrated authenticated client/server, not a separate LAN/dedicated-client run',
            'Sprint remap uses the ctl repeat path because ctl down also inserts the canonical action into physical_keys',
            'Center-only clone selection; unsupported raw side/facade hosts remain refused',
            'Complete client part prediction, FAIL/swing/offhand semantics, facades, side parts and source lifecycle remain open']}
        (work/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill(); process.wait(timeout=10)
        for log in logs:
            log.close()
        socket.unlink(missing_ok=True)


if __name__ == '__main__':
    main()
