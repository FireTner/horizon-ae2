#!/usr/bin/env python3
"""Verify common tags in the real isolated Horizon server with /execute if items.

Checks actual item predicates, including an existing AE2 tag, then restarts.
Only writes owned target/smoke worlds and closes its own client processes.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
LOCK = json.loads((ROOT / "upstream.lock.json").read_text())
CHECKS = [("c:ingots/iron", "minecraft:iron_ingot"), ("c:ingots/copper", "minecraft:copper_ingot"),
          ("c:rods/wooden", "minecraft:stick"), ("c:nuggets/iron", "minecraft:iron_nugget"),
          ("c:nuggets/gold", "minecraft:gold_nugget"), ("c:dyes/red", "minecraft:red_dye"),
          ("c:dusts/ender_pearl", "ae2:ender_dust")]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    parent = ROOT / "target/smoke"; parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="platform-tags-", dir=parent)); game_dir = work / "game"; game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-platform-tags-{os.getpid()}.sock"
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
        process = subprocess.Popen([str(args.game.resolve()), "--world", "PlatformTagSmoke", "--creative", "--seed", "42",
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

    def check(phase):
        for index, (tag, item) in enumerate(CHECKS):
            ctl("command", f"item replace entity @s weapon.mainhand with {item}")
            positive = f"AE2_TAG_{phase}_{index}_POSITIVE"
            result = ctl("command", f"execute if items entity @s weapon.mainhand #{tag} run say {positive}")
            assert any(positive in line for line in result["chat"]), (tag, result)
            ctl("command", "item replace entity @s weapon.mainhand with minecraft:stone")
            negative = f"AE2_TAG_{phase}_{index}_NEGATIVE"
            result = ctl("command", f"execute unless items entity @s weapon.mainhand #{tag} run say {negative}")
            assert any(negative in line for line in result["chat"]), (tag, result)

    try:
        first = launch(1); check(1); ctl("save"); ctl("quit"); first.wait(timeout=20)
        launch(2); check(2); ctl("quit")
        report = {"passed": True, "logs": str(work), "platform_revision": LOCK["platform_tags"]["revision"],
                  "checks": CHECKS, "positive_checks": len(CHECKS) * 2, "negative_checks": len(CHECKS) * 2,
                  "scope": "real server item predicates and content tag layering, existing AE2 contribution, restart",
                  "not_covered": ["membership of every carried tag", "all non-item tag registries", "all recipe conditions and survival crafting"]}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: original platform tags load in Horizon before and after restart; logs: {work}", flush=True)
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
