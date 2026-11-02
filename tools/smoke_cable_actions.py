#!/usr/bin/env python3
"""Isolated CableBus placement prediction, interaction results and server correction check.

Uses real right clicks, two hands and held server packets to observe prediction.
Full multipart behavior and external multiplayer remain separate.
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
    work = Path(tempfile.mkdtemp(prefix='cable-actions-', dir=parent))
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
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'CableActionsSmoke', '--creative', '--seed', '42', '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640', '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(ROOT/'target/content/ae2')], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
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
    def aim():
        ctl('tp',6.5,73,5.5);ctl('wait',4);ctl('look-at',6.5,74.5,8.5);ctl('wait',4)
        hit=ctl('looking-at');assert hit.get('kind')=='block',hit
    def clear_targets():
        for p in [north,south]:block(p,'air')
        ctl('wait',4)
    def count(item):return sum(s['count'] for s in inventory() if s['item']==item)
    def swings():return ctl('state')['swings']
    def offhand(item='minecraft:shield',number=1):
        ctl('command','item replace entity @s weapon.offhand with '+item+' '+str(number));ctl('wait',3)
    def predicted(pos,expected):
        data=ctl('block',*pos);assert data['block']==expected,data
        return data
    def click_once():
        ctl('mouse','right','down');ctl('mouse','right','up');ctl('wait',2)
    try:
        launch(1);ctl('tp',8.5,82,3.5);ctl('wait',60)
        ctl('command','gamerule doMobSpawning false')
        ctl('command','fill 3 71 2 14 72 13 stone');ctl('command','fill 3 73 2 14 82 13 air')
        block(anchor,'stone');ctl('command','gamemode survival')
        ctl('command','scoreboard objectives add ae2_used minecraft.used:ae2.white_glass_cable')
        # Source client success places the part and swings, but does not shrink.
        # Server packets are held while only the real client continues ticking.
        for kind in ['glass_cable','covered_cable','smart_cable','covered_dense_cable','smart_dense_cable']:
            clear_targets();aim();item='ae2:white_'+kind;give(item,3);offhand('minecraft:stone',64)
            before=swings();ctl('server-hold','on')
            try:
                click_once();preview=predicted(north,'ae2:cable_bus')
                assert ctl('getblock',*north)['block']=='minecraft:air'
                assert count(item)==3,inventory()
                assert swings()==before+1,ctl('state')
                assert ctl('status')['offhand']['count']==64
            finally:ctl('server-hold','off')
            ctl('wait',10);predicted(north,'ae2:cable_bus')
            assert item in nbt(north),nbt(north)
            assert count(item)==2,inventory()
            assert swings()==before+1,ctl('state')
            assert ctl('status')['offhand']['count']==64
            hit=ctl('looking-at');assert hit.get('block',{}).get('pos')==list(north),hit
            if kind=='glass_cable':
                score=ctl('command','scoreboard players get @s ae2_used')
                assert any('has 1 ' in line for line in score['chat']),score
            checks.append({'case':'client preview and one server consumption for '+kind,'preview':preview})
            print('PASS: prediction, one swing and one consumption for '+kind,file=__import__('sys').stderr,flush=True)
        # The actor's own body obstructs the target. FAIL must stop before the
        # off-hand shield, and neither client nor server starts a swing.
        clear_targets();aim();ctl('tp',6.5,73,7.2);ctl('wait',4);ctl('look-at',6.5,74.5,8.5)
        item='ae2:cyan_smart_cable';give(item,3);offhand()
        before=swings();ctl('server-hold','on')
        try:
            ctl('mouse','right','down');ctl('wait',2)
            assert ctl('state')['using'] is None,ctl('state')
            assert swings()==before,ctl('state')
            ctl('mouse','right','up')
        finally:ctl('server-hold','off')
        ctl('wait',8);assert swings()==before,ctl('state');assert count(item)==3
        checks.append('FAIL does not swing or raise the off-hand shield before or after the server processes it')
        # The raw facade occupies the otherwise empty destination host. Its
        # wire fields must prevent an incorrect optimistic central placement.
        clear_targets();block(north,'ae2:cable_bus');merge(north,'{hasRedstone:2,facadeNorth:{opaque:99}}');ctl('wait',6);aim();give(item,3)
        before=swings();click_once();ctl('wait',8)
        assert swings()==before,ctl('state');assert 'opaque: 99' in nbt(north),nbt(north);assert count(item)==3
        assert ctl('state')['using'] is None
        checks.append('opaque facade-host admission is refused on both sides without clearing the source payload')
        # A streamed entity blocks the source client core before the server can
        # decide; the provisional host is removed locally, with no swing/shrink.
        clear_targets();aim();give(item,3)
        ctl('command','summon minecraft:cow 6.9 74 7.5 {NoAI:1b,NoGravity:1b,Silent:1b}');ctl('wait',8)
        ctl('look-at',6.05,74.5,8.5);ctl('wait',3)
        assert ctl('looking-at').get('kind')=='block',ctl('looking-at')
        before=swings();ctl('server-hold','on')
        try:
            click_once();predicted(north,'minecraft:air');assert swings()==before,ctl('state');assert count(item)==3
        finally:ctl('server-hold','off')
        ctl('wait',8);assert ctl('getblock',*north)['block']=='minecraft:air';assert swings()==before
        ctl('command','kill @e[type=minecraft:cow]');ctl('wait',6)
        checks.append('client and server entity obstruction roll back the host without swing or item consumption')
        # An empty compatible host must retain the accepted part after the
        # block's sequence acknowledgement, rather than restoring its old NBT.
        clear_targets();block(north,'ae2:cable_bus');merge(north,'{hasRedstone:2}');ctl('wait',6);aim();give(item,3)
        before=swings();click_once();ctl('wait',10)
        assert count(item)==2;assert item in nbt(north);assert swings()==before+1
        hit=ctl('looking-at');assert hit.get('block',{}).get('pos')==list(north),hit
        checks.append('authoritative BE data survives acknowledgement when a predicted part reuses an empty host')
        # Force a real server-only refusal through this adapter's documented
        # strict corrupt-owner-map gate. This is a reconciliation test, not a
        # claim about Java missing-data recovery.
        ctl('save');registry=game_dir/'saves/CableActionsSmoke/data/ae2_players.dat';saved=registry.read_bytes()
        for reverse in [False,True]:
            clear_targets();aim();give(item,3);before=swings()
            ctl('key','ctrl','down' if reverse else 'up');ctl('wait',3)
            target=south if reverse else north
            ctl('server-hold','on');registry.write_bytes(b'AE2 isolated invalid owner map')
            try:
                click_once();predicted(target,'ae2:cable_bus');assert swings()==before+1
            finally:ctl('server-hold','off')
            try:
                ctl('wait',10);predicted(target,'minecraft:air');assert ctl('getblock',*target)['block']=='minecraft:air'
                assert count(item)==3;assert swings()==before+1;assert ctl('state')['using'] is None
            finally:registry.write_bytes(saved)
            checks.append('server refusal corrects '+('CTRL reverse' if reverse else 'normal')+' preview without spending the item or adding a server swing')
        ctl('key','ctrl','up')
        # Two predicted native actions can share the same position before a
        # single server drain. A placement BE must not revive the later break.
        clear_targets();ctl('command','gamemode creative');ctl('wait',4);aim();give(item,2)
        ctl('server-hold','on')
        try:
            click_once();predicted(north,'ae2:cable_bus')
            ctl('mouse','left','down');ctl('mouse','left','up');ctl('wait',2)
            predicted(north,'minecraft:air')
        finally:ctl('server-hold','off')
        ctl('wait',10);predicted(north,'minecraft:air')
        assert ctl('getblock',*north)['block']=='minecraft:air'
        ctl('command','gamemode survival');ctl('wait',4)
        checks.append('coalesced predicted placement and native break do not revive a stale cable host after acknowledgement')
        # The block's use still runs before PartItem.useOn.
        clear_targets();block(anchor,'chest');aim();give(item,3);before=swings();click_once();ctl('wait',8)
        assert ctl('state')['screen']=='container',ctl('state');assert count(item)==3
        assert ctl('getblock',*north)['block']=='minecraft:air'
        ctl('key','esc','tap');ctl('wait',4);block(anchor,'stone')
        checks.append('chest reaction retains priority over part placement')
        # No stale failure result can affect a subsequent ordinary block item.
        clear_targets();aim();give('minecraft:stone',3);before=swings();click_once();ctl('wait',8)
        assert ctl('getblock',*north)['block']=='minecraft:stone';assert swings()>before
        checks.append('scope cleanup leaves subsequent vanilla item actions unchanged')
        ctl('save');ctl('quit')
        report={'passed':True,'checks':checks,'logs':str(work),'limits':[
            'Integrated authenticated client/server; external LAN and dedicated clients remain separate',
            'Covers supported center parts; full side parts, facades, source security/settings, source visual packets and all lifecycle paths remain open',
            'Corrupt owner map is an intentional native strict-refusal fault, not a Java recovery oracle',
            'No complete Java/NeoForge world comparison; exact source lighting, neighbour events and packet timing remain separate']}
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
