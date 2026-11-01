#!/usr/bin/env python3
"""Verify original AE2 Charger inv.item0, live echo, malformed retention and restart.

Uses its own isolated world/home; closes only its own clients.
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
    work = Path(tempfile.mkdtemp(prefix='charger-inventory-', dir=parent))
    game_dir = work / 'game'; game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f'ae2-charger-inventory-{os.getpid()}.sock'
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
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'ChargerInventorySmoke', '--creative', '--seed', '42',
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

    def original():
        return "\n".join(ctl('command', 'data get block 8 73 7 inv')['chat'])

    def set_original(value):
        ctl('command', f'data modify block 8 73 7 inv.item0 set value {value}')

    try:
        first = launch(1)
        ctl('command', 'fill 5 72 3 11 72 9 stone'); ctl('command', 'fill 5 73 3 11 78 9 air')
        ctl('tp', 8.5, 73, 4.5)
        setup(777)
        ctl('command', 'data merge block 8 73 7 {inv:{item0:{id:"ae2:portable_item_cell_1k",count:1,components:{"ae2:stored_energy":1234.0d,"ae2:energy_capacity":5000.0d,"ae2:storage_cell_inv":[{"#t":"ae2:i",id:"minecraft:iron_ingot","#":123L}]}}}}')
        ctl('wait', 40); value = sample(); saved = original()
        assert value['item'] == 2011 and value['local'] == 0, value
        assert number(saved, 'ae2:stored_energy') == 2011 and 'minecraft:iron_ingot' in saved, saved
        assert 'HorizonWorking' not in value['nbt'], value
        check('original inv.item0 hydrates native slot and charging persists energy/contents without stale echo refill', {'native': value, 'inv': saved})

        set_original('{id:"ae2:charged_staff",count:1,components:{"ae2:stored_energy":1000.0d}}')
        ctl('wait', 20)
        assert ctl('container', *charger)['slots'] == [{'item':'ae2:charged_staff','count':1}]
        ctl('command', 'data merge block 8 73 7 {internalCurrentPower:500.0d}')
        ctl('wait', 60); value = sample(); saved = original()
        assert value['item'] == 1500 and value['local'] == 0, value
        assert number(saved, 'ae2:stored_energy') == 1500, saved
        check('explicit source-format replacement wins over old native mirror and live charging is not reverted', value)

        set_original('{id:"ae2:charged_staff",count:0}')
        ctl('wait', 30); invalid = original()
        assert re.search(r'count:\s*0', invalid), invalid
        ctl('select', 9); ctl('wait', 3); ctl('interact', *charger); ctl('wait', 5)
        assert re.search(r'count:\s*0', original()), original()
        assert ctl('container', *charger)['slots'] == [{'item':'ae2:charged_staff','count':1}]
        set_original('{}'); ctl('wait', 20)
        assert ctl('container', *charger)['slots'] == [None]
        check('malformed source count is retained without creating a card/item or allowing manual extraction; explicit repair unblocks', {'invalid': invalid, 'repaired': original()})

        setup(0)
        ctl('command', 'data modify block 8 73 7 Items set value [{Slot:0b,id:"ae2:color_applicator",count:1,components:{"ae2:stored_energy":777.0d,"ae2:energy_capacity":1000.0d}}]')
        ctl('wait', 20); saved = original()
        assert 'ae2:color_applicator' in saved and number(saved, 'ae2:stored_energy') == 777, saved
        ctl('command', 'data merge block 8 73 7 {"test:extra":123456789L}')
        ctl('save'); ctl('quit'); first.wait(timeout=20)
        launch(2); ctl('wait', 20); value = sample(); saved = original()
        assert value['item'] == 777 and value['local'] == 0 and '123456789L' in value['nbt'], value
        assert 'ae2:color_applicator' in saved and number(saved, 'ae2:stored_energy') == 777, saved
        ctl('select', 9); ctl('wait', 3); ctl('interact', *charger); ctl('tp', 8.5, 73, 6.5); ctl('wait', 20)
        assert ctl('container', *charger)['slots'] == [None]
        assert 'item0: {}' in original() or '"item0": {}' in original(), original()
        inventory = "\n".join(ctl('command', 'data get entity @s Inventory')['chat'])
        assert 'ae2:color_applicator' in inventory and number(inventory, 'ae2:stored_energy') == 777, inventory
        check('legacy native Items migrates to original inv and survives restart/retrieval with exact private energy', {'inv_before': saved, 'inventory': inventory})

        setup(1600)
        set_original('{id:"ae2:certus_quartz_crystal",count:1}')
        for _ in range(100):
            ctl('wait', 20)
            if ctl('container', *charger)['slots'] == [{'item':'ae2:charged_certus_quartz_crystal','count':1}]:
                break
        else:
            raise AssertionError('original-format recipe input did not finish')
        saved = original(); value = sample()
        assert 'ae2:charged_certus_quartz_crystal' in saved and value['local'] == 0, (saved, value)
        check('recipe output updates original inv.item0 and consumes exactly 1600 AE', {'inv': saved, 'native': value})
        ctl('quit')
        report = {'passed':True,'logs':str(work),'checks':checks,
                  'not_covered':['complete Minecraft malformed/missing-content recovery','Charger hopper/capability and full destruction/explosion/drop lifecycle','external multiplayer','all legacy migrations']}
        (work/'result.json').write_text(json.dumps(report,indent=2)+'\n')
        print(f'PASS: original Charger inventory persistence; logs: {work}',flush=True)
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
