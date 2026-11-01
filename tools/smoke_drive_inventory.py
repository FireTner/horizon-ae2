#!/usr/bin/env python3
"""Drive source ItemStack codec: zero-count refusal, blocked echo, count-two load.

Creates its own world/home and closes only its own clients.
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
    prefix = "drive-inventory-"
    work = Path(tempfile.mkdtemp(prefix=prefix, dir=parent)); game_dir = work / "game"; game_dir.mkdir()
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
        process = subprocess.Popen([str(args.game.resolve()), "--world", "DriveInventorySmoke", "--creative", "--seed", "42",
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

    def original():
        return "\n".join(ctl("command", "data get block 8 73 7 inv")["chat"])

    def set_source(count):
        # Direct source NBT load differs from normal input, whose limit stays 1.
        field = "" if count == 1 else f"count:{count},"
        ctl("command", 'data modify block 8 73 7 inv.item0 set value {id:"ae2:item_storage_cell_1k",' + field + 'components:{"ae2:storage_cell_inv":[{"#t":"ae2:i",id:"minecraft:iron_ingot","#":123L}]}}')

    try:
        first = launch(1)
        ctl("command", "fill 5 72 3 11 72 9 stone"); ctl("command", "fill 5 73 3 11 78 9 air")
        ctl("tp", 8.5, 73, 4.5); ctl("setblock", *drive, "ae2:drive[facing=north]"); finite(back); ctl("wait", 40)
        rate(2.0 + 2.0/128.0)
        set_source(1); ctl("wait", 30); rate(2.5 + 2.0/128.0)
        assert ctl("container", *drive)["slots"][0] == {"item":"ae2:item_storage_cell_1k","count":1}
        print("PASS: valid original slot mounts one 1k backend with its exact idle cost",flush=True)

        set_source(0); ctl("wait", 30); rate(2.0 + 2.0/128.0)
        saved = original(); assert re.search(r'count:\s*0', saved) and '123L' in saved, saved
        ctl("command", 'data modify block 8 73 7 Items set value [{Slot:0b,id:"ae2:item_storage_cell_4k",count:1}]')
        ctl("wait", 30); rate(2.0 + 2.0/128.0)
        assert re.search(r'count:\s*0', original()) and '123L' in original(), original()
        assert ctl("container", *drive)["slots"][0] == {"item":"ae2:item_storage_cell_4k","count":1}
        print("PASS: source count zero remains raw/unmounted; changing native echo cannot activate or overwrite it",flush=True)

        ctl("save"); ctl("quit"); first.wait(timeout=20); launch(2); ctl("wait", 30)
        rate(2.0 + 2.0/128.0)
        assert re.search(r'count:\s*0', original()) and '123L' in original(), original()
        set_source(1); ctl("wait", 30); rate(2.5 + 2.0/128.0)
        assert ctl("container", *drive)["slots"][0] == {"item":"ae2:item_storage_cell_1k","count":1}
        print("PASS: blocked raw inventory survives restart; explicit original repair remounts the saved data",flush=True)

        set_source(2); ctl("wait", 30); rate(2.5 + 2.0/128.0)
        assert ctl("container", *drive)["slots"][0] == {"item":"ae2:item_storage_cell_1k","count":2}
        assert re.search(r'count:\s*2', original()) and '123L' in original(), original()
        print("PASS: direct positive count-two source stack is preserved and mounts one backend with one cell idle drain",flush=True)
        ctl("quit")
        report={"passed":True,"logs":str(work),"scope":["count_zero_refusal_before_native_clamp","original_raw_retention","blocked_native_echo_does_not_unblock","restart","explicit_repair","direct_source_count_two_preserved_one_backend"],
                "measurements":measurements,"not_covered":["all malformed/missing-content recovery","complete legacy migrations","all drop/explosion/capability paths"]}
        (work/"result.json").write_text(json.dumps(report,indent=2)+"\n")
        print(f"PASS: Drive source inventory codec; logs: {work}",flush=True)
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
