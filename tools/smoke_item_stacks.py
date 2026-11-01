#!/usr/bin/env python3
"""Isolated native stack limits: every unstackable registration, controls, full/restart."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

from item_stack_limits import UNSTACKABLE

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ctl', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--game', type=Path, default=ROOT / 'target/horizon-cargo/fast/horizon-client')
    args = parser.parse_args()
    parent = ROOT / 'target/smoke'; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='item-stacks-', dir=parent))
    game_dir = work / 'game'; game_dir.mkdir()
    socket = work / 'control.json'
    env = {**os.environ, 'HORIZON_HOME': str(work / 'home'), 'HORIZON_NO_AUDIO': '1'}
    transcript = []; processes = []; logs = []; cases = []
    barrel = (8, 74, 7); hopper = (8, 73, 7)

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
        process = subprocess.Popen([str(args.game.resolve()), '--world', 'ItemStackLimitsSmoke', '--creative', '--seed', '42',
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

    def slots(pos):
        return [s for s in ctl('container', *pos)['slots'] if s]

    def merge(pos, text):
        ctl('command', f"data merge block {' '.join(map(str, pos))} {text}")

    def setup(item, count=3, full=False):
        ctl('server-hold', 'on')
        try:
            component = 'components:{"ae2:stored_energy":777.0d,"example:payload":99L}'
            stack = f'id:"{item}",count:{count},{component}'
            merge(barrel, '{Items:[{Slot:0b,' + stack + '}]}')
            filled = ','.join(f'{{Slot:{i}b,id:"{item}",count:1,{component}}}' for i in range(5)) if full else ''
            merge(hopper, '{Items:[' + filled + ']}')
        finally:
            ctl('server-hold', 'off')

    def check_components(pos, copies):
        text = '\n'.join(ctl('command', f"data get block {' '.join(map(str, pos))}")['chat'])
        assert text.count('777.0d') == copies and text.count('99L') == copies, text

    try:
        first = launch(1)
        ctl('tp', 8.5, 80, 4.5); ctl('wait', 60)
        ctl('command', 'fill 5 72 3 11 72 10 stone'); ctl('command', 'fill 5 73 3 11 79 10 air')
        ctl('tp', 8.5, 73, 4.5); ctl('wait', 5)
        ctl('setblock', *barrel, 'minecraft:barrel'); ctl('setblock', *hopper, 'minecraft:hopper[facing=east]')
        targets = [(f'ae2:{name}', 1) for name in sorted(UNSTACKABLE)]
        targets += [('ae2:blank_pattern', 64), ('ae2:certus_quartz_crystal', 64), ('minecraft:stone', 64)]
        for item, limit in targets:
            setup(item); ctl('wait', 35)
            result = slots(hopper)
            assert not slots(barrel), (item, slots(barrel))
            expected = [1, 1, 1] if limit == 1 else [3]
            assert [s['count'] for s in result] == expected, (item, result)
            assert all(s['item'] == item for s in result), result
            check_components(hopper, len(expected))
            cases.append({'item': item, 'limit': limit, 'destination_counts': expected})
        item = 'ae2:portable_item_cell_1k'
        setup(item, count=1, full=True); ctl('wait', 35)
        assert slots(barrel) == [{'item': item, 'count': 1}], slots(barrel)
        assert slots(hopper) == [{'item': item, 'count': 1}] * 5, slots(hopper)
        check_components(hopper, 5)
        setup(item); ctl('wait', 35); check_components(hopper, 3)
        ctl('save'); ctl('quit'); first.wait(timeout=20); launch(2); ctl('wait', 10)
        assert not slots(barrel) and slots(hopper) == [{'item': item, 'count': 1}] * 3
        check_components(hopper, 3)
        for item, expected in [('ae2:portable_item_cell_1k', [1, 1, 1]), ('ae2:blank_pattern', [3])]:
            ctl('command', 'clear @s'); ctl('give', item, 3)
            inv = ctl('inventory')
            counts = [s['count'] for s in inv['hotbar'] + inv['main'] if s.get('item') == item]
            assert counts == expected, (item, counts)
        ctl('quit')
        report = {'passed': True, 'cases': cases, 'logs': str(work),
                  'additional': ['full hopper refuses sixth unstackable item', 'components and exact counts survive restart',
                                 'player inventory splits unstackable give; stackable AE2 control still merges'],
                  'limits': ['component-specific max-stack overrides', 'external multiplayer',
                             'other item properties and full item behaviours', 'Java/NeoForge world oracle']}
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
