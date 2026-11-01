#!/usr/bin/env python3
"""Isolated native Charger hopper/dropper rules, persistence and rollback.

Synthetic recipes test tag membership and a powered recipe ingredient without
changing the user's installed data. All generated data stays under target/.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
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
    parent = ROOT / 'target/smoke'; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='charger-automation-', dir=parent))
    game_dir = work / 'game'; game_dir.mkdir()
    content = work / 'content'
    shutil.copytree(ROOT / 'target/content/ae2', content)
    recipes = content / 'data/ae2/recipe'
    # Unsupported result components must not change ingredient admission/extraction.
    (recipes / 'automation_test_tag.json').write_text(json.dumps({
        'type': 'ae2:charger', 'ingredient': {'tag': 'ae2:automation_test_input'},
        'result': {'id': 'minecraft:book', 'components': {'example:unsupported': 7}}}))
    (recipes / 'automation_test_powered.json').write_text(json.dumps({
        'type': 'ae2:charger', 'ingredient': {'item': 'ae2:charged_staff'},
        'result': {'id': 'minecraft:book'}}))
    tags = content / 'data/ae2/tags/item'; tags.mkdir(parents=True, exist_ok=True)
    (tags / 'automation_test_input.json').write_text(json.dumps({'values': ['minecraft:iron_ingot']}))
    socket = work / 'control.json'
    env = {**os.environ, 'HORIZON_HOME': str(work / 'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []; processes = []; logs = []; checks = []
    charger = (8, 74, 7); above = (8, 75, 7); below = (8, 73, 7)

    def ctl(*command):
        run = subprocess.run([str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '25', *map(str, command)],
                             env=env, cwd=game_dir, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout)
        transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work / 'commands.json').write_text(json.dumps(transcript, indent=2) + '\n')
        if run.returncode:
            raise RuntimeError(f'{command}: {data}')
        return data

    def launch(number):
        log = (work / f'game-{number}.log').open('w'); logs.append(log)
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'ChargerAutomationSmoke', '--creative', '--seed', '42',
            '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640',
            '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(content)],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process)
        deadline = time.monotonic() + 60
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f'game exited: {log.name}')
            try:
                ctl('wait-ready'); return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(.1)
        raise RuntimeError(f'game not ready: {log.name}')

    def slots(pos):
        return ctl('container', *pos)['slots']

    def merge(pos, value):
        ctl('command', f"data merge block {' '.join(map(str, pos))} {value}")

    def put(pos, spec):
        merge(pos, '{Items:[{Slot:0b,' + spec + '}]}')

    def source(spec=''):
        merge(charger, '{internalCurrentPower:0.0d}')
        ctl('command', 'data modify block 8 74 7 inv.item0 set value {' + spec + '}')
        ctl('wait', 2)

    def nbt(pos):
        return '\n'.join(ctl('command', f"data get block {' '.join(map(str, pos))}")['chat'])

    def empty(pos):
        assert all(slot is None for slot in slots(pos)), slots(pos)

    def one(pos, item, count=1):
        assert slots(pos)[0] == {'item': item, 'count': count}, slots(pos)

    try:
        first = launch(1)
        ctl('tp', 8.5, 80, 4.5); ctl('wait', 60)
        ctl('command', 'fill 5 72 3 11 72 10 stone'); ctl('command', 'fill 5 73 3 11 79 10 air')
        ctl('tp', 8.5, 73, 4.5); ctl('wait', 5)
        placed = ctl('setblock', *charger, 'ae2:charger[facing=north]')
        assert placed['block'].startswith('ae2:charger'), placed
        ctl('setblock', *above, 'minecraft:hopper[facing=down]')
        put(above, 'id:"minecraft:stone",count:3'); ctl('wait', 25)
        one(above, 'minecraft:stone', 3); empty(charger)
        put(above, 'id:"minecraft:iron_ingot",count:3'); ctl('wait', 25)
        one(charger, 'minecraft:iron_ingot'); one(above, 'minecraft:iron_ingot', 2)
        merge(above, '{Items:[]}')
        ctl('setblock', *above, 'air'); ctl('setblock', *below, 'minecraft:hopper[facing=east]')
        ctl('wait', 25); empty(below); one(charger, 'minecraft:iron_ingot')
        checks.append('tag ingredient admitted despite unsupported result; recipe input extraction blocked; limit/remainder')
        # Ingredient membership, not presence as a recipe result, permits extraction.
        source('id:"minecraft:stone",count:3'); ctl('wait', 30)
        empty(charger); one(below, 'minecraft:stone', 3)
        checks.append('source loaded unrelated item overstack extracts without clamping')
        source('id:"ae2:wrapped_generic_stack"'); ctl('wait', 20)
        one(charger, 'ae2:wrapped_generic_stack'); one(below, 'minecraft:stone', 3)
        checks.append('exposed handler rejects internal wrapped stack extraction')
        ctl('setblock', *below, 'air')
        # All six inventory faces, including grid front and bottom.
        positions = [(above, 'hopper', 'down'), ((8,74,6), 'hopper', 'south'),
                     ((8,74,8), 'hopper', 'north'), ((7,74,7), 'hopper', 'east'),
                     ((9,74,7), 'hopper', 'west'), (below, 'dropper', 'up')]
        for pos, block, facing in positions:
            source()
            ctl('setblock', *pos, f'minecraft:{block}[facing={facing}]')
            put(pos, 'id:"ae2:certus_quartz_crystal",count:3')
            if block == 'dropper':
                ctl('setblock', 8, 73, 8, 'minecraft:redstone_block')
            ctl('wait', 25)
            one(charger, 'ae2:certus_quartz_crystal'); one(pos, 'ae2:certus_quartz_crystal', 2)
            merge(pos, '{Items:[]}')
            ctl('setblock', *pos, 'air'); ctl('setblock', 8, 73, 8, 'air')
        checks.append('all six insertion faces hopper/dropper including front; one physical slot')
        # Every supported powered item is admitted; the unusual fallback allows
        # immediate extraction even while empty when no recipe accepts it.
        powered = ['ae2:charged_staff', 'ae2:entropy_manipulator', 'ae2:wireless_terminal',
                   'ae2:wireless_crafting_terminal', 'ae2:matter_cannon', 'ae2:color_applicator']
        powered += [f'ae2:portable_{kind}_cell_{tier}' for kind in ['item', 'fluid']
                    for tier in ['1k', '4k', '16k', '64k', '256k']]
        for item in powered:
            source(); ctl('setblock', *above, 'minecraft:hopper[facing=down]')
            put(above, f'id:"{item}",count:1'); ctl('wait', 15)
            empty(above); one(charger, item); ctl('setblock', *above, 'air')
            ctl('setblock', *below, 'minecraft:hopper[facing=east]'); ctl('wait', 15)
            if item == 'ae2:charged_staff':
                empty(below); one(charger, item)
                source('id:"ae2:charged_staff",components:{"ae2:stored_energy":8000.0d}')
                ctl('wait', 15)
            empty(charger); one(below, item); ctl('setblock', *below, 'air')
        checks.append('all sixteen powered item insertion/extraction; empty powered recipe input held until full')
        # Failed transfer must restore all components, exact count and original inv.
        spec = 'id:"ae2:portable_item_cell_1k",count:3,components:{"ae2:stored_energy":777.0d,"example:payload":99L}'
        source(spec)
        ctl('server-hold', 'on')
        try:
            ctl('setblock', *below, 'minecraft:hopper[facing=east]')
            blocked = ','.join(f'{{Slot:{slot}b,id:"minecraft:stone",count:63}}' for slot in range(5))
            merge(below, '{Items:[' + blocked + ']}')
        finally:
            ctl('server-hold', 'off')
        ctl('wait', 25); one(charger, 'ae2:portable_item_cell_1k', 3)
        saved = nbt(charger)
        assert '777.0d' in saved and '99L' in saved and 'inv:' in saved, saved
        merge(below, '{Items:[]}'); ctl('wait', 30)
        empty(charger)
        result = slots(below)
        assert sum(slot['count'] for slot in result if slot) == 3, result
        assert all(slot['item'] == 'ae2:portable_item_cell_1k' for slot in result if slot), result
        assert [slot['count'] for slot in result if slot] == [1, 1, 1], result
        saved = nbt(below); assert '777.0d' in saved and '99L' in saved, saved
        assert re.search(r'item0:\s*\{\s*\}', nbt(charger)), nbt(charger)
        checks.append('failed pull restoration; source overstack retains components/current power; immediate empty original persistence')
        ctl('setblock', *below, 'air')
        source('id:"ae2:charged_staff",count:0')
        merge(charger, '{Items:[{Slot:0b,id:"ae2:charged_staff",count:1,components:{"ae2:stored_energy":8000.0d}}]}')
        ctl('setblock', *above, 'minecraft:hopper[facing=down]')
        put(above, 'id:"ae2:certus_quartz_crystal",count:1')
        ctl('setblock', *below, 'minecraft:hopper[facing=east]'); ctl('wait', 25)
        one(above, 'ae2:certus_quartz_crystal'); empty(below)
        assert re.search(r'count:\s*0', nbt(charger)), nbt(charger)
        ctl('save'); ctl('quit'); first.wait(timeout=20); launch(2); ctl('wait', 20)
        one(above, 'ae2:certus_quartz_crystal'); empty(below)
        assert re.search(r'count:\s*0', nbt(charger)), nbt(charger)
        source(); ctl('wait', 20)
        one(charger, 'ae2:certus_quartz_crystal'); empty(above); empty(below)
        checks.append('malformed original blocks insertion/extraction across restart until explicit repair')
        ctl('setblock', 10, 74, 7, 'minecraft:barrel')
        ctl('setblock', 10, 75, 7, 'minecraft:hopper[facing=down]')
        put((10, 75, 7), 'id:"minecraft:stone",count:3'); ctl('wait', 30)
        empty((10, 75, 7)); one((10, 74, 7), 'minecraft:stone', 3)
        checks.append('vanilla barrel and hopper retain ordinary stack insertion')
        ctl('quit')
        report = {'passed': True, 'checks': checks, 'logs': str(work),
                  'limits': ['external capability API', 'addon power registry', 'all destruction/chunk paths',
                             'comparator fullness',
                             'complete Charger parity and Java world lifecycle']}
        (work / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))
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
        (game_dir / socket).unlink(missing_ok=True)


if __name__ == '__main__':
    main()
