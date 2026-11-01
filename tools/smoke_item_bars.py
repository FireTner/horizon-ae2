#!/usr/bin/env python3
"""Render empty/half/full AE2 bars beside vanilla durability/counts in an isolated world.

Checks exact bar pixels as well as rendering and authoritative item counts.
"""
import argparse
import json
import os
import re
import struct
import zlib
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]


def verify_pixels(path):
    # Read 8-bit, noninterlaced RGB(A) PNG from the actual offscreen framebuffer.
    # No image editing and no third-party image library is involved.
    data = path.read_bytes(); assert data[:8] == b'\x89PNG\r\n\x1a\n'
    pos = 8; compressed = b''
    while pos < len(data):
        n = struct.unpack('>I', data[pos:pos + 4])[0]
        kind = data[pos + 4:pos + 8]; chunk = data[pos + 8:pos + 8 + n]; pos += n + 12
        if kind == b'IHDR':
            width, height, bits, color, _, _, interlace = struct.unpack('>IIBBBBB', chunk)
        elif kind == b'IDAT':
            compressed += chunk
    assert (width, height) == (960, 640), (width, height)
    assert bits == 8 and color in (2, 6) and interlace == 0
    bpp = 3 if color == 2 else 4; stride = width * bpp; raw = zlib.decompress(compressed)
    previous = bytearray(stride); rows = []
    for y in range(height):
        row = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        filter_type = raw[y * (stride + 1)]
        for i in range(stride):
            a = row[i - bpp] if i >= bpp else 0
            b = previous[i]; c = previous[i - bpp] if i >= bpp else 0
            if filter_type == 0: value = 0
            elif filter_type == 1: value = a
            elif filter_type == 2: value = b
            elif filter_type == 3: value = (a + b) // 2
            else:
                assert filter_type == 4
                prediction = a + b - c
                pa = abs(prediction - a); pb = abs(prediction - b); pc = abs(prediction - c)
                value = a if pa <= pb and pa <= pc else b if pb <= pc else c
            row[i] = (row[i] + value) & 255
        rows.append(row); previous = row
    # Fixed test viewport + isolated default options: GUI scale 2, 176x166 barrel.
    left = (width - 176 * 2) // 2; top = (height - 166 * 2) // 2
    widths = [0, 7, 13, 7, 13]
    for slot, expected in enumerate(widths):
        start = left + (8 + 18 * slot + 2) * 2
        for dy in range(4):
            y = top + (142 + 13) * 2 + dy
            for dx in range(26):
                wanted = (0, 255, 0) if dy < 2 and dx < expected * 2 else (0, 0, 0)
                actual = tuple(rows[y][(start + dx) * bpp:(start + dx) * bpp + 3])
                assert actual == wanted, (slot, expected, dx, dy, actual, wanted)
    return {"verified_widths": widths, "scale": 2, "bar_color": "#00ff00", "background": "#000000"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    parent = ROOT / "target/smoke"; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="item-bars-", dir=parent)); game_dir = work / "game"; game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-item-bars-{os.getpid()}.sock"
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
        process = subprocess.Popen([str(args.game.resolve()), "--world", "ItemBarsSmoke", "--creative", "--seed", "42",
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

    try:
        launch(1)
        ctl("command", "fill 5 72 3 11 72 9 stone"); ctl("command", "fill 5 73 3 11 78 9 air")
        ctl("tp", 8.5, 73, 4.5); ctl("command", "clear @s")
        specifications = [
            "ae2:portable_item_cell_1k",
            "ae2:portable_item_cell_1k[ae2:stored_energy=10000.0d]",
            "ae2:portable_item_cell_1k[ae2:stored_energy=20000.0d]",
            "ae2:charged_staff[ae2:stored_energy=4000.0d]",
            "ae2:color_applicator[ae2:stored_energy=20000.0d]",
            "minecraft:diamond_pickaxe[damage=700]",
            "minecraft:stone 64",
        ]
        for slot, specification in enumerate(specifications):
            result = ctl("command", f"item replace entity @s hotbar.{slot} with {specification}")
            assert not any('Unknown' in line or '<--[HERE]' in line for line in result.get('chat', [])), result
        ctl("select", 9); ctl("wait", 5)
        ctl("setblock", 8, 73, 7, "minecraft:barrel"); ctl("interact", 8, 73, 7); ctl("wait", 5)
        screen = ctl("screen"); assert screen["screen"] == "container", screen
        inventory = ctl("inventory")
        assert inventory["counts"].get("ae2:portable_item_cell_1k") == 3, inventory
        # Container clicks compare the real wire hashes, not just visible ids.
        # Move the half-charged cell through a vanilla barrel and back.
        for slot in [1, 300]:
            ctl("click-slot", slot); ctl("wait", 5)
        assert ctl("container", 8, 73, 7)["slots"][0] == {"item": "ae2:portable_item_cell_1k", "count": 1}
        saved = "\n".join(ctl("command", "data get block 8 73 7 Items")["chat"])
        stored = re.search(r'"ae2:stored_energy":\s*([-+0-9.eE]+)d', saved)
        assert stored and float(stored.group(1)) == 10000.0, saved
        for slot in [300, 1]:
            ctl("click-slot", slot); ctl("wait", 5)
        assert ctl("container", 8, 73, 7)["slots"][0] is None
        assert ctl("inventory")["counts"].get("ae2:portable_item_cell_1k") == 3
        ctl("screenshot", str(work / "item-bars.png"))
        pixels = verify_pixels(work / "item-bars.png")
        report = {"passed": True, "logs": str(work), "scope": ["powered_bars_render_in_native_container", "vanilla_tool_and_stack_render_regression", "componentful_stack_hashes_and_menu_roundtrip"],
                  "expected_hotbar_bars": [0, 7, 13, 7, 13, "vanilla durability", "vanilla count 64"],
                  "pixel_checks": pixels, "not_covered": ["energy tooltip", "reequip/creative tab", "external multiplayer"]}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        ctl("quit"); print(f"PASS: rendered native energy bars; logs: {work}", flush=True)
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
