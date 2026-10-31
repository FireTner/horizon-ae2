#!/usr/bin/env python3
"""Isolated real hopper/dropper Drive transfers and component retention.

Only creates owned target/smoke worlds. Tests built-in vanilla automation,
not general native/NeoForge capability or external Create compatibility.
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
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    parent = ROOT / "target/smoke"; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="drive-automation-", dir=parent)); game_dir = work / "game"; game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-drive-automation-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    transcript = []; processes = []; logs = []

    def ctl(*command):
        run = subprocess.run([str(args.ctl.resolve()), "--socket", str(socket), "--timeout", "25", *map(str, command)],
                             env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout); transcript.append({"command": command, "exit": run.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if run.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch(number):
        log = (work / f"game-{number}.log").open("w"); logs.append(log)
        process = subprocess.Popen([str(args.game.resolve()), "--world", "DriveAutomationSmoke", "--creative", "--seed", "42",
            "--offscreen", "--control-socket", str(socket), "--render-distance", "3", "--width", "960", "--height", "640",
            "--game-dir", str(game_dir), "--resources", str(args.resources.resolve()), "--dev-mod", str(ROOT / "target/content/ae2")],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process); deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"game exited: {log.name}")
            try:
                ctl("wait-ready"); return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(.1)
        raise RuntimeError(f"game did not become ready: {log.name}")

    drive = (8, 74, 7); above = (8, 75, 7); below = (8, 73, 7)
    cell = "ae2:item_storage_cell_1k"

    def slots(pos):
        return ctl("container", *pos)["slots"]

    def put(pos, stack):
        ctl("command", f"data merge block {' '.join(map(str, pos))} {{Items:[{{Slot:0b,{stack}}}]}}")

    def contents(pos):
        text = "\n".join(ctl("command", f"data get block {' '.join(map(str, pos))}")["chat"])
        assert "minecraft:iron_ingot" in text and re.search(r'"#"\s*:\s*123L', text), text
        return text

    try:
        first = launch(1)
        ctl("command", "fill 5 72 3 11 72 10 stone"); ctl("command", "fill 5 73 3 11 79 10 air")
        ctl("tp", 8.5, 73, 4.5); ctl("setblock", *drive, "ae2:drive[facing=north]")
        ctl("setblock", *above, "minecraft:hopper[facing=down]")
        put(above, 'id:"minecraft:stone",count:3'); ctl("wait", 25)
        assert slots(above)[0] == {"item": "minecraft:stone", "count": 3}
        assert all(slot is None for slot in slots(drive))
        put(above, f'id:"{cell}",count:1,components:{{"ae2:storage_cell_inv":[{{"#t":"ae2:i",id:"minecraft:iron_ingot","#":123L}}]}}')
        ctl("wait", 25)
        assert all(slot is None for slot in slots(above))
        assert slots(drive)[0] == {"item": cell, "count": 1}; contents(drive)
        # A failed pull must restore the exact cell, including components.
        # Counts 63 leave the hopper non-full, but no slot can accept a cell.
        ctl("server-hold", "on")
        try:
            ctl("setblock", *below, "minecraft:hopper[facing=east]")
            blocked = ','.join(f'{{Slot:{slot}b,id:"minecraft:stone",count:63}}' for slot in range(5))
            ctl("command", f"data merge block {' '.join(map(str, below))} {{Items:[{blocked}]}}")
        finally:
            ctl("server-hold", "off")
        ctl("wait", 25)
        assert slots(drive)[0] == {"item": cell, "count": 1}; contents(drive)
        assert slots(below) == [{"item": "minecraft:stone", "count": 63}] * 5
        # The exact same cell may then be extracted while the ME node is offline.
        ctl("command", f"data merge block {' '.join(map(str, below))} {{Items:[]}}"); ctl("wait", 25)
        assert all(slot is None for slot in slots(drive))
        assert slots(below)[0] == {"item": cell, "count": 1}; contents(below)
        ctl("setblock", *above, "air"); ctl("setblock", *below, "air")
        # Four horizontal faces accept a disk, including the ME front face.
        for offset, facing in [((0, 0, -1), "south"), ((0, 0, 1), "north"), ((-1, 0, 0), "east"), ((1, 0, 0), "west")]:
            pos = tuple(drive[i] + offset[i] for i in range(3))
            before = sum(slot is not None for slot in slots(drive))
            ctl("setblock", *pos, f"minecraft:hopper[facing={facing}]")
            put(pos, f'id:"{cell}",count:1'); ctl("wait", 25)
            assert sum(slot is not None for slot in slots(drive)) == before + 1
            assert all(slot is None for slot in slots(pos)); ctl("setblock", *pos, "air")
        # A dropper below inserts through the sixth (bottom) face.
        ctl("setblock", *below, "minecraft:dropper[facing=up]"); put(below, f'id:"{cell}",count:1')
        ctl("setblock", 8, 73, 8, "minecraft:redstone_block"); ctl("wait", 25)
        assert sum(slot is not None for slot in slots(drive)) == 5
        assert all(slot is None for slot in slots(below))
        ctl("setblock", 8, 73, 8, "air"); ctl("setblock", *below, "air")
        # Fill all ten slots from one deliberately stacked input. Padding is
        # unreachable and each physical slot still contains exactly one cell.
        ctl("setblock", *above, "minecraft:hopper[facing=down]")
        put(above, f'id:"{cell}",count:6'); ctl("wait", 70)
        result = slots(drive)
        assert result[:10] == [{"item": cell, "count": 1}] * 10, result
        assert all(slot is None for slot in result[10:]), result
        assert slots(above)[0] == {"item": cell, "count": 1}, slots(above)
        ctl("save"); ctl("quit"); first.wait(timeout=20); launch(2); ctl("wait", 20)
        assert slots(drive)[:10] == [{"item": cell, "count": 1}] * 10
        assert slots(above)[0] == {"item": cell, "count": 1}
        ctl("quit")
        report = {"passed": True, "logs": str(work), "scope": ["reject_non_cells", "all_six_insert_faces", "offline_insert_extract", "failed_pull_restores_cell",
            "components_retained", "ten_slots_limit_one", "full_drive_remainder", "save_restart"],
            "not_covered": ["external capability API", "Create integration", "world dirty backend mutation through terminal", "all chunk/drop paths"]}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: Drive automation faces, filters, limits and restart; logs: {work}", flush=True)
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


if __name__ == "__main__":
    main()
