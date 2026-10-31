#!/usr/bin/env python3
"""Isolated native Drive verification: original NBT, idle power, front, restart.

This is the currently connected backend, not a terminal/GUI acceptance test.
Only creates its own game/home under target/smoke; all owned processes are closed.
"""
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
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    parent = ROOT / "target/smoke"; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="drive-", dir=parent)); game_dir = work / "game"; game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-drive-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    transcript = []; processes = []; logs = []; measurements = []

    def ctl(*command):
        run = subprocess.run([str(args.ctl.resolve()), "--socket", str(socket), "--timeout", "25", *map(str, command)],
                             env=env, text=True, capture_output=True, timeout=30)
        data = json.loads(run.stdout)
        transcript.append({"command": command, "exit": run.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if run.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch(number):
        log = (work / f"game-{number}.log").open("w"); logs.append(log)
        process = subprocess.Popen([str(args.game.resolve()), "--world", "DriveSmoke", "--creative", "--seed", "42",
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

    drive = (8, 73, 7); back = (8, 73, 8); front = (8, 73, 6)

    def nbt(pos):
        return "\n".join(ctl("command", f"data get block {' '.join(map(str, pos))}")["chat"])

    def power(pos):
        match = re.search(r"internalCurrentPower:\s*([-+0-9.eE]+)d", nbt(pos))
        return float(match.group(1)) if match else 0.0

    def finite(pos):
        ctl("setblock", *pos, "ae2:energy_cell")
        ctl("command", f"data merge block {' '.join(map(str, pos))} {{internalCurrentPower:20000.0d}}")

    def sample(pos):
        # Pause the authoritative server, so NBT and game_time describe one
        # state. This is a test control, not Minecraft's /tick freeze.
        ctl("server-hold", "on")
        try:
            return {"tick": ctl("simulation")["game_time"], "energy": power(pos)}
        finally:
            ctl("server-hold", "off")

    def rate(expected):
        before = sample(back); ctl("wait", 25); after = sample(back)
        ticks = after["tick"] - before["tick"]
        used = before["energy"] - after["energy"]
        measurements.append({"before": before, "after": after, "expected_per_tick": expected})
        assert ticks > 0 and math.isclose(used, ticks * expected, abs_tol=1e-8), (before, after, expected)

    def contents():
        slots = ctl("container", *drive)["slots"]
        assert len(slots) == 10, slots
        assert slots[0] == {"item": "ae2:item_storage_cell_1k", "count": 1}, slots
        assert slots[9] == {"item": "ae2:fluid_storage_cell_4k", "count": 1}, slots
        value = nbt(drive)
        assert "minecraft:iron_ingot" in value and "minecraft:water" in value, value
        assert re.search(r'"#"\s*:\s*123L', value), value
        assert re.search(r'"#"\s*:\s*1000L', value), value

    try:
        first = launch(1)
        ctl("command", "fill 5 72 3 11 72 9 stone"); ctl("command", "fill 5 73 3 11 78 9 air")
        ctl("tp", 8.5, 73, 4.5); ctl("setblock", *drive, "ae2:drive[facing=north]"); finite(back); ctl("wait", 40)
        rate(2.0 + 2.0 / 128.0)
        print("PASS: empty Drive consumes 2 AE/tick plus actual ad-hoc channel cost", flush=True)
        data = '''{inv:{item0:{id:"ae2:item_storage_cell_1k",count:1,components:{"ae2:storage_cell_inv":[{"#t":"ae2:i",id:"minecraft:iron_ingot","#":123L}]}},item9:{id:"ae2:fluid_storage_cell_4k",count:1,components:{"ae2:storage_cell_inv":[{"#t":"ae2:f",id:"minecraft:water","#":1000L}]}}}}'''
        ctl("command", f"data merge block {' '.join(map(str, drive))} {data}"); ctl("wait", 20)
        contents(); rate(2.0 + .5 + 1.0 + 2.0 / 128.0)
        print("PASS: original inv.item0/item9 load item/fluid cells with their exact idle drain", flush=True)
        ctl("setblock", *back, "air"); finite(front); ctl("wait", 40)
        assert power(front) == 20000.0
        contents(); print("PASS: front face is excluded; offline disks retain contents", flush=True)
        ctl("setblock", *front, "air"); finite(back); ctl("wait", 40)
        ctl("save"); ctl("quit"); first.wait(timeout=20); launch(2); ctl("wait", 20)
        contents(); rate(2.0 + .5 + 1.0 + 2.0 / 128.0)
        ctl("look-at", 8.5, 73.5, 7.5); ctl("screenshot", str(work / "drive-backend.png")); ctl("quit")
        report = {"passed": True, "logs": str(work), "scope": ["original_inv_ten_slots", "item_fluid_cell_persistence",
            "drive_and_cell_idle_drain", "front_exclusion", "offline_retention", "save_restart"],
            "measurements": measurements, "not_covered": ["GUI", "terminal transfers", "dynamic chassis and LEDs", "chunk/dimension/multiplayer"]}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: Drive backend survives restart; logs: {work}", flush=True)
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
