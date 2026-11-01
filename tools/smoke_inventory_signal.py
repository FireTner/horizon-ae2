#!/usr/bin/env python3
"""Actual comparator/dust output for AE2 host slots, mutations and vanilla isolation."""
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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--game', type=Path, default=ROOT / 'target/horizon-cargo/fast/horizon-client')
    args = parser.parse_args()
    parent = ROOT / 'target/smoke'; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='inventory-signal-', dir=parent))
    game_dir = work / 'game'; game_dir.mkdir()
    socket = work / 'control.json'
    env = {**os.environ, 'HORIZON_HOME': str(work / 'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []; processes = []; logs = []; cases = []
    host = (8, 73, 7); comparator = (9, 73, 7); dust = (10, 73, 7)
    above = (8, 74, 7); below = (8, 72, 7)

    def ctl(*command):
        run = subprocess.run([str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '25', *map(str, command)],
                             cwd=game_dir, env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout)
        transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work / 'commands.json').write_text(json.dumps(transcript, indent=2) + '\n')
        if run.returncode:
            raise RuntimeError(f'{command}: {data}')
        return data

    def launch(number):
        log = (work / f'game-{number}.log').open('w'); logs.append(log)
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'InventorySignalSmoke', '--creative', '--seed', '42',
            '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640',
            '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()),
            '--dev-mod', str(ROOT / 'target/content/ae2')], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
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

    def merge(pos, text):
        ctl('command', f"data merge block {' '.join(map(str, pos))} {text}")

    def slots(pos):
        return [s for s in ctl('container', *pos)['slots'] if s]

    def expect(label, expected):
        ctl('wait', 8)
        block = ctl('getblock', *dust)['block']
        match = re.search(r'\bpower=(\d+)', block)
        assert match and int(match[1]) == expected, (label, expected, block)
        cases.append({'case': label, 'expected': expected, 'actual': int(match[1])})

    def circuit():
        # Diode FACING points to its input, toward the host west of it.
        ctl('setblock', *comparator, 'minecraft:comparator[facing=west]')
        ctl('setblock', *dust, 'minecraft:redstone_wire')

    def original(block, stacks):
        # Replacing the comparator isolates arithmetic from mutation notifications;
        # the later hopper/manual scenarios retain it and test those notifications.
        ctl('setblock', *comparator, 'air')
        ctl('setblock', *host, block)
        inv = ','.join(f'item{i}:{{id:"{item}",count:{count}}}' for i, item, count in stacks)
        merge(host, '{inv:{' + inv + '}}'); ctl('wait', 3); circuit()

    try:
        first = launch(1)
        ctl('tp', 8.5, 80, 4.5); ctl('wait', 60)
        ctl('command', 'fill 5 71 3 12 72 10 stone'); ctl('command', 'fill 5 73 3 12 79 10 air')
        ctl('tp', 8.5, 73, 4.5); ctl('wait', 5)
        for label, stacks, expected in [
            ('charger empty', [], 0),
            ('charger stackable raw recipe item fills its limit-one slot', [(0, 'ae2:certus_quartz_crystal', 1)], 15),
            ('charger powered item', [(0, 'ae2:portable_item_cell_1k', 1)], 15),
            ('charger air stack is empty even with a positive saved count', [(0, 'minecraft:air', 3)], 0),
        ]:
            original('ae2:charger[facing=north]', stacks); expect(label, expected)
        levels = [0, 2, 3, 5, 6, 8, 9, 10, 12, 13, 15]
        for n, expected in enumerate(levels):
            original('ae2:drive[facing=north]', [(i, 'ae2:item_storage_cell_1k', 1) for i in range(n)])
            expect(f'drive {n}/10 occupied, ignoring eight wire-menu padding slots', expected)
        original('ae2:drive[facing=north]', [(0, 'ae2:item_storage_cell_1k', 2)])
        expect('drive direct source count two is not clamped', 3)
        # The existing strict malformed policy remains intact and cannot be
        # bypassed with a filled native echo.
        original('ae2:charger[facing=north]', [(0, 'ae2:charged_staff', 0)])
        merge(host, '{Items:[{Slot:0b,id:"ae2:charged_staff",count:1}]}')
        expect('malformed original stays empty for analog output', 0)
        for name, item, expected in [('charger', 'ae2:charged_staff', 15), ('drive', 'ae2:item_storage_cell_1k', 2)]:
            original(f'ae2:{name}[facing=north]', []); expect(f'{name} before hopper insertion', 0)
            ctl('setblock', *above, 'minecraft:hopper[facing=down]')
            merge(above, '{Items:[{Slot:0b,id:"' + item + '",count:1}]}'); ctl('wait', 20)
            assert slots(host) == [{'item': item, 'count': 1}], slots(host)
            expect(f'{name} hopper insertion updates existing comparator', expected)
            ctl('setblock', *above, 'air'); ctl('setblock', *below, 'minecraft:hopper[facing=west]'); ctl('wait', 20)
            assert not slots(host) and slots(below) == [{'item': item, 'count': 1}]
            expect(f'{name} hopper extraction updates existing comparator', 0)
            ctl('setblock', *below, 'stone')
        original('ae2:charger[facing=north]', [])
        ctl('command', 'clear @s'); ctl('give', 'ae2:certus_quartz_crystal', 1); ctl('select', 1); ctl('wait', 3)
        ctl('interact', *host); expect('manual Charger insertion notifies comparator', 15)
        ctl('select', 9); ctl('wait', 3); ctl('interact', *host)
        expect('manual Charger extraction notifies comparator', 0)
        original('ae2:drive[facing=north]', [])
        ctl('command', 'clear @s'); ctl('give', 'ae2:item_storage_cell_1k', 1); ctl('wait', 5)
        ctl('interact', *host); ctl('wait', 5)
        inv = ctl('inventory')
        slot = next(s['slot'] for s in inv['hotbar'] + inv['main'] if s.get('item') == 'ae2:item_storage_cell_1k')
        ctl('click-slot', slot, 'shift'); expect('Drive GUI insertion notifies comparator', 2)
        ctl('click-slot', 300, 'shift'); expect('Drive GUI extraction notifies comparator', 0)
        ctl('key', 'escape', 'down'); ctl('key', 'escape', 'up'); ctl('wait', 3)
        for count, expected in [(0, 0), (1, 1), (32, 8), (64, 15)]:
            ctl('setblock', *comparator, 'air'); ctl('setblock', *host, 'minecraft:barrel')
            items = ','.join(f'{{Slot:{i}b,id:"minecraft:stone",count:{count}}}' for i in range(27)) if count else ''
            merge(host, '{Items:[' + items + ']}'); circuit()
            expect(f'vanilla barrel {count} per slot unchanged', expected)
        original('ae2:drive[facing=north]', [(i, 'ae2:item_storage_cell_1k', 1) for i in range(10)])
        expect('full Drive before restart', 15)
        ctl('save'); ctl('quit'); first.wait(timeout=20); launch(2)
        expect('full Drive comparator after restart', 15)
        ctl('quit')
        report = {'passed': True, 'cases': cases, 'logs': str(work),
                  'scope': 'actual comparator and adjacent dust, source slot limits/counts, notifications and vanilla isolation',
                  'limits': ['Java/NeoForge world oracle', 'other AE2 hosts and energy-cell comparator',
                             'full malformed/missing inventory recovery and all drop/chunk paths']}
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
        socket.unlink(missing_ok=True)


if __name__ == '__main__':
    main()
