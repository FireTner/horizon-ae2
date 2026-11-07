#!/usr/bin/env python3
"""Grow a flawless budding certus block into a cluster and harvest four crystals.

Uses BuddingCertusQuartzBlock's 1/5 growth roll. The flawless host does not decay.
A player break of ae2:quartz_cluster drops four ae2:certus_quartz_crystal
(quartz_cluster.json set_count, no Fortune).
"""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
PREPARED = Path("/home/cachyfire/.ao/data/worktrees/horizon-ae2/horizon-ae2-2/target/content/ae2")
FACES = ((0, 1, 0), (0, -1, 0), (0, 0, -1), (0, 0, 1), (-1, 0, 0), (1, 0, 0))
HOST = (8, 73, 8)


def ensure_content():
    content = ROOT / "target/content/ae2"
    if not (content / "ae2-recipe-map.json").is_file():
        if content.exists():
            shutil.rmtree(content)
        shutil.copytree(PREPARED, content, symlinks=True)
    budding = content / "data/ae2/horizon/block/flawless_budding_quartz.json"
    budding.write_text(json.dumps({"template": "minecraft:budding_amethyst"}) + "\n")
    for name in ("small_quartz_bud", "medium_quartz_bud", "large_quartz_bud", "quartz_cluster"):
        path = content / "data/ae2/horizon/block" / f"{name}.json"
        path.write_text(json.dumps({
            "template": "minecraft:amethyst_cluster",
            "properties": {"requires_tool": False},
        }) + "\n")
    return content


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ctl", type=Path, required=True)
    parser.add_argument("--resources", type=Path, required=True)
    parser.add_argument("--game", type=Path, default=ROOT / "target/horizon-cargo/fast/horizon-client")
    args = parser.parse_args()
    content = ensure_content()
    parent = ROOT / "target/smoke"
    parent.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="quartz-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    socket = Path(tempfile.gettempdir()) / f"ae2-quartz-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(work / "home"), "HORIZON_NO_AUDIO": "1"}
    processes, logs, transcript = [], [], []

    def ctl(*command):
        run = subprocess.run(
            [str(args.ctl.resolve()), "--socket", str(socket), "--timeout", "40", *map(str, command)],
            env=env, text=True, capture_output=True, timeout=50,
        )
        data = json.loads(run.stdout)
        transcript.append({"command": command, "exit": run.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if run.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch():
        log = (work / "game.log").open("w")
        logs.append(log)
        process = subprocess.Popen(
            [str(args.game.resolve()), "--world", "QuartzSmoke", "--creative", "--seed", "42",
             "--offscreen", "--control-socket", str(socket), "--render-distance", "3",
             "--width", "960", "--height", "640", "--game-dir", str(game_dir),
             "--resources", str(args.resources.resolve()), "--dev-mod", str(content)],
            cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT,
        )
        processes.append(process)
        deadline = time.monotonic() + 40
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"game exited: {log.name}")
            try:
                ctl("wait-ready")
                return
            except (RuntimeError, subprocess.TimeoutExpired, json.JSONDecodeError):
                time.sleep(0.1)
        raise RuntimeError("game did not become ready")

    try:
        launch()
        ctl("command", "fill 4 72 4 12 72 12 stone")
        ctl("command", "fill 4 73 4 12 78 12 air")
        ctl("tp", 8.5, 73, 4.5)
        ctl("setblock", *HOST, "ae2:flawless_budding_quartz")
        ctl("command", "gamerule random_tick_speed 200000")
        cluster = None
        for _ in range(40):
            ctl("wait", 10)
            for face in FACES:
                pos = tuple(HOST[i] + face[i] for i in range(3))
                block = ctl("block", *pos)["block"]
                if block == "ae2:quartz_cluster":
                    cluster = pos
                    break
            if cluster:
                break
        if cluster is None:
            around = {str(face): ctl("block", *(HOST[i] + face[i] for i in range(3)))["block"] for face in FACES}
            raise AssertionError(f"budding quartz did not grow a cluster: {around}")
        print(f"PASS: cluster grew at {cluster}", flush=True)
        ctl("command", "gamemode survival")
        ctl("command", "item replace entity @s hotbar.0 with minecraft:diamond_pickaxe 1")
        held = None
        for _ in range(20):
            held = ctl("select", 1).get("held")
            if isinstance(held, dict) and held.get("item") == "minecraft:diamond_pickaxe":
                break
            ctl("wait", 2)
        else:
            raise AssertionError(f"diamond pickaxe was not in the hotbar: {held}")
        ctl("tp", cluster[0] + 0.5, cluster[1], cluster[2] - 1.5)
        broken = ctl("break", *cluster)

        def crystals():
            total = 0
            for source in (
                broken.get("drops") or {},
                broken.get("picked_up") or {},
                ctl("inventory").get("counts") or {},
            ):
                total = max(total, int(source.get("ae2:certus_quartz_crystal", 0)))
            return total

        count = crystals()
        for _ in range(12):
            if count >= 4:
                break
            ctl("wait", 5)
            count = crystals()
        if count != 4:
            raise AssertionError(
                f"cluster dropped {count} certus quartz crystals, source loot is 4: {broken}"
            )
        report = {"passed": True, "logs": str(work), "cluster": cluster, "crystals": count}
        (work / "result.json").write_text(json.dumps(report, indent=2) + "\n")
        print(f"PASS: harvested 4 ae2:certus_quartz_crystal; logs: {work}", flush=True)
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
        for log in logs:
            log.close()
        socket.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
