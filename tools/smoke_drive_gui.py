#!/usr/bin/env python3
"""Real Drive screen: visible slots, filtering, pickup/shift and restart retention.

Creates only owned target/smoke data. Does not claim priority/dynamic render or
all AEBaseMenu action parity. The library tests exercise drag and hotbar swaps.
"""
import argparse
import json
import os
from pathlib import Path
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
    work = Path(tempfile.mkdtemp(prefix="drive-gui-", dir=parent)); game_dir = work / "game"; game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-drive-gui-{os.getpid()}.sock"
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
        process = subprocess.Popen([str(args.game.resolve()), "--world", "DriveGuiSmoke", "--creative", "--seed", "42",
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

    drive = (8, 73, 7)
    item = "ae2:item_storage_cell_1k"; fluid = "ae2:fluid_storage_cell_4k"

    def inventory_slot(id):
        inv = ctl("inventory")
        return next(slot["slot"] for slot in inv["hotbar"] + inv["main"] if slot.get("item") == id)

    def click(slot, *flags):
        ctl("click-slot", slot, *flags); ctl("wait", 5)

    def slots():
        return ctl("container", *drive)["slots"]

    def open_drive():
        ctl("select", 9); ctl("interact", *drive); ctl("wait", 5)
        screen = ctl("screen")
        assert screen["screen"] == "container", screen
        visible = [slot["slot"] for slot in screen["slots"] if 300 <= slot["slot"] < 400]
        assert visible == list(range(300, 310)), visible

    try:
        first = launch(1)
        ctl("command", "fill 5 72 3 11 72 9 stone"); ctl("command", "fill 5 73 3 11 78 9 air")
        ctl("tp", 8.5, 73, 4.5); ctl("command", "clear @s"); ctl("setblock", *drive, "ae2:drive[facing=north]")
        for id, count in [(item, 1), (fluid, 1), ("minecraft:stone", 4)]:
            ctl("give", id, count)
        ctl("wait", 5); open_drive()
        stone_slot = inventory_slot("minecraft:stone"); click(stone_slot); click(300)
        assert slots()[0] is None
        assert ctl("inventory")["carried"]["item"] == "minecraft:stone"
        click(stone_slot); assert ctl("inventory")["carried"] is None
        click(inventory_slot(item)); click(300)
        assert slots()[0] == {"item": item, "count": 1}
        assert ctl("inventory")["carried"] is None
        click(inventory_slot(fluid), "shift")
        assert slots()[1] == {"item": fluid, "count": 1}
        click(300, "shift")
        assert slots()[0] is None
        assert inventory_slot(item) == 9, "AEBaseMenu sends extracted cells to main inventory before hotbar"
        click(9, "shift"); assert slots()[0] == {"item": item, "count": 1}
        assert all(slot is None for slot in slots()[10:])
        assert ctl("inventory")["counts"].get("minecraft:stone") == 4
        ctl("screenshot", str(work / "drive-screen.png")); ctl("key", "escape", "down"); ctl("key", "escape", "up"); ctl("wait", 5)
        # The next ordinary container keeps vanilla insertion and reverse
        # quick-move order; a Drive policy must not leak across windows.
        barrel = (10, 73, 7); ctl("setblock", *barrel, "minecraft:barrel"); ctl("interact", *barrel); ctl("wait", 5)
        click(inventory_slot("minecraft:stone")); click(300)
        assert ctl("container", *barrel)["slots"][0] == {"item": "minecraft:stone", "count": 4}
        click(300, "shift"); assert inventory_slot("minecraft:stone") == 8
        ctl("key", "escape", "down"); ctl("key", "escape", "up"); ctl("wait", 5)
        ctl("save"); ctl("quit"); first.wait(timeout=20); launch(2); ctl("wait", 10); open_drive()
        assert slots()[0] == {"item": item, "count": 1} and slots()[1] == {"item": fluid, "count": 1}
        assert ctl("inventory")["counts"].get("minecraft:stone") == 4
        ctl("screenshot", str(work / "drive-screen-restart.png")); ctl("quit")
        report = {"passed": True, "logs": str(work), "scope": ["original_screen_layout", "ten_visible_slots",
            "reject_non_cells", "pickup", "shift_insert", "shift_extract_order", "vanilla_window_isolation", "no_item_loss", "restart"],
            "not_covered": ["priority submenu", "AEBaseMenu special bulk actions", "external multiplayer client", "dynamic chassis/LEDs"]}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: Drive GUI filters, shifts and restart; logs: {work}", flush=True)
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
