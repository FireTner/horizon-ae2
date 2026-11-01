#!/usr/bin/env python3
"""Isolated world tests: built-in powered items in Charger, conservation/restart.

Creates only its own game/home under target/smoke and closes its own clients.
This does not exercise handheld actions, Workbench or external power capabilities.
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
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--game', type=Path, default=ROOT / 'target/horizon-cargo/fast/horizon-client')
    args = parser.parse_args()
    parent = ROOT / 'target/smoke'; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='item-charging-', dir=parent))
    game_dir = work / 'game'; game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f'ae2-item-charging-{os.getpid()}.sock'
    env = {**os.environ, 'HORIZON_HOME': str(work / 'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []; processes = []; logs = []; checks = []
    charger = (8, 73, 7); back = (8, 73, 8); front = (8, 73, 6); crank = (8, 74, 7)

    def ctl(*command):
        run = subprocess.run([str(args.ctl.resolve()), '--socket', str(socket), '--timeout', '25', *map(str, command)],
                             env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout)
        transcript.append({'command': command, 'exit': run.returncode, 'result': data})
        (work / 'commands.json').write_text(json.dumps(transcript, indent=2) + '\n')
        if run.returncode:
            raise RuntimeError(f'{command}: {data}')
        return data

    def launch(number):
        log = (work / f'game-{number}.log').open('w'); logs.append(log)
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'ItemChargingSmoke', '--creative', '--seed', '42',
            '--offscreen', '--control-socket', str(socket), '--render-distance', '3', '--width', '960', '--height', '640',
            '--game-dir', str(game_dir), '--resources', str(args.resources.resolve()), '--dev-mod', str(ROOT / 'target/content/ae2')],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process)
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f'game exited: {log.name}')
            try:
                ctl('wait-ready'); return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(.1)
        raise RuntimeError(f'game did not become ready: {log.name}')

    def nbt(pos):
        return '\n'.join(ctl('command', f"data get block {' '.join(map(str, pos))}")['chat'])

    def number(text, key, default=0.0):
        match = re.search(r'(?:"' + re.escape(key) + r'"|' + re.escape(key) + r'):\s*([-+0-9.eE]+)d', text)
        return float(match.group(1)) if match else default

    def setup(local, back_energy=None, front_energy=None):
        ctl('server-hold', 'on')
        try:
            ctl('command', 'clear @s')
            for pos in (crank, back, front, charger):
                ctl('setblock', *pos, 'air')
            ctl('setblock', *charger, 'ae2:charger[facing=north]')
            ctl('command', f'data merge block 8 73 7 {{internalCurrentPower:{float(local)}d}}')
            for pos, value in ((back, back_energy), (front, front_energy)):
                if value is not None:
                    ctl('setblock', *pos, 'ae2:energy_cell')
                    ctl('command', f"data merge block {' '.join(map(str, pos))} {{internalCurrentPower:{float(value)}d}}")
        finally:
            ctl('server-hold', 'off')

    def insert(id, components=''):
        spec = id + ('[' + components + ']' if components else '')
        ctl('command', f'item replace entity @s hotbar.0 with {spec}')
        assert not any('Unknown' in line or '<--[HERE]' in line for line in transcript[-1]['result'].get('chat', [])), transcript[-1]
        ctl('select', 1); ctl('wait', 3); ctl('interact', *charger); ctl('select', 9); ctl('wait', 3)
        slots = ctl('container', *charger)['slots']
        assert slots == [{'item': id, 'count': 1}], slots

    def sample(with_back=False, with_front=False):
        ctl('server-hold', 'on')
        try:
            value = nbt(charger)
            data = {'local': number(value, 'internalCurrentPower'), 'item': number(value, 'ae2:stored_energy'), 'nbt': value,
                    'back': number(nbt(back), 'internalCurrentPower') if with_back else 0.0,
                    'front': number(nbt(front), 'internalCurrentPower') if with_front else 0.0}
            data['total'] = data['local'] + data['item'] + data['back'] + data['front']
            return data
        finally:
            ctl('server-hold', 'off')

    def check(name, value):
        checks.append({'name': name, 'state': value})
        print(f'PASS: {name}', flush=True)

    try:
        first = launch(1)
        ctl('command', 'fill 5 72 3 11 72 9 stone'); ctl('command', 'fill 5 73 3 11 78 9 air')
        ctl('tp', 8.5, 73, 4.5)
        upgrades = 'ae2:upgrades=[{slot:0,item:{id:"ae2:energy_card",count:1}},{slot:3,item:{id:"ae2:energy_card",count:1}}]'
        payload = 'ae2:storage_cell_inv=[{"#t":"ae2:i",id:"minecraft:iron_ingot","#":123L}],minecraft:custom_data={ae2_test:123456789L}'
        setup(1600, 1000)
        insert('ae2:portable_item_cell_1k', 'ae2:energy_capacity=1000.0d,ae2:stored_energy=990.0d,' + upgrades + ',' + payload)
        ctl('wait', 30); value = sample(True)
        assert value['item'] == 1000 and value['local'] == 1600 and value['back'] == 990, value
        assert 'ae2:energy_card' in value['nbt'] and 'minecraft:iron_ingot' in value['nbt'] and '123456789L' in value['nbt'], value
        check('near-full upgraded portable: source-ordered extraction returns overflow without losing energy/components', value)

        setup(1600, 5000)
        insert('ae2:matter_cannon', 'ae2:energy_capacity=5000.0d')
        ctl('wait', 30); value = sample(True)
        assert value['item'] == 5000 and value['local'] == 1600 and value['back'] == 0, value
        check('Matter Cannon uses local buffer plus direct ME extraction and conserves 6600 AE', value)

        setup(0, front_energy=2000)
        insert('ae2:portable_fluid_cell_1k', 'ae2:energy_capacity=1000.0d,ae2:stored_energy=10.0d')
        ctl('wait', 30); value = sample(with_front=True)
        assert value['item'] == 10 and value['local'] == 0 and value['front'] == 2000, value
        check('front face remains excluded for item charging', value)

        setup(0)
        insert('ae2:charged_staff')
        ctl('setblock', *crank, 'ae2:crank[facing=up]'); ctl('interact', *crank); ctl('wait', 40)
        value = sample()
        assert value['item'] == 160 and value['local'] == 0, value
        check('one manual Crank rotation charges a staff by exactly 160 AE', value)

        ids = ['ae2:charged_staff', 'ae2:entropy_manipulator', 'ae2:matter_cannon', 'ae2:color_applicator',
               'ae2:wireless_terminal', 'ae2:wireless_crafting_terminal']
        ids += [f'ae2:portable_{kind}_cell_{tier}' for kind in ('item', 'fluid') for tier in ('1k', '4k', '16k', '64k', '256k')]
        for id in ids:
            setup(1600)
            insert(id, 'ae2:energy_capacity=320.0d')
            ctl('wait', 20); value = sample()
            assert value['item'] == 320 and value['local'] == 1280, (id, value)
        check('all 16 built-in powered items accept manual insertion and charge to an override without duplication', {'items': ids})

        setup(777)
        insert('ae2:portable_item_cell_1k', 'ae2:energy_capacity=5000.0d,ae2:stored_energy=1234.0d,' + payload)
        ctl('wait', 30); before = sample()
        assert before['item'] == 2011 and before['local'] == 0, before
        ctl('save'); ctl('quit'); first.wait(timeout=20)
        launch(2); ctl('wait', 20); after = sample()
        assert after['item'] == before['item'] and after['local'] == before['local'], (before, after)
        assert 'minecraft:iron_ingot' in after['nbt'] and '123456789L' in after['nbt'], after
        check('partial item energy, capacity override and storage payload survive process restart', after)
        ctl('select', 9); ctl('wait', 3); ctl('interact', *charger); ctl('tp', 8.5, 73, 6.5); ctl('wait', 20)
        assert ctl('container', *charger)['slots'] == [None]
        inventory = '\n'.join(ctl('command', 'data get entity @s Inventory')['chat'])
        assert 'ae2:portable_item_cell_1k' in inventory and number(inventory, 'ae2:stored_energy') == 2011, inventory
        assert 'minecraft:iron_ingot' in inventory and '123456789L' in inventory, inventory
        check('manual retrieval retains charged item and nested storage payload', {'inventory': inventory})
        ctl('look-at', 8.5, 73.5, 7.5); ctl('screenshot', str(work / 'item-charging.png')); ctl('quit')
        report = {'passed': True, 'logs': str(work), 'checks': checks,
                  'not_covered': ['handheld actions/menus', 'upgrade editing/callbacks', 'Charger hopper capability', 'live config reload', 'external multiplayer/Java world oracle']}
        (work / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
        print(f'PASS: item charging/restart; logs: {work}', flush=True)
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
