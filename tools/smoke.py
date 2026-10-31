#!/usr/bin/env python3
"""Run Charger/Crank and restart checks in a NEW isolated Horizon test world.

Requires a built native client, horizon-ctl and prepared 26.3 resources.
Never uses player saves/options. Keeps logs and commands in target/smoke/.
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
    parent = ROOT / "target/smoke"
    parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="charger-", dir=parent))
    game_dir = work / "game"
    game_dir.mkdir()
    home = work / "home"
    socket = Path(tempfile.gettempdir()) / f"horizon-ae2-{os.getpid()}.sock"
    env = {**os.environ, "HORIZON_HOME": str(home), "HORIZON_NO_AUDIO": "1"}
    transcript = []
    processes = []
    log_handles = []

    def ctl(*command):
        result = subprocess.run([str(args.ctl.resolve()), "--socket", str(socket),
                                 "--timeout", "25", *map(str, command)],
                                capture_output=True, text=True, timeout=30, env=env)
        data = json.loads(result.stdout)
        transcript.append({"command": command, "exit": result.returncode, "result": data})
        (work / "commands.json").write_text(json.dumps(transcript, indent=2) + "\n")
        if result.returncode:
            raise RuntimeError(f"{command}: {data}")
        return data

    def launch(number):
        log = (work / f"game-{number}.log").open("w")
        log_handles.append(log)
        process = subprocess.Popen([
            str(args.game.resolve()), "--world", "AE2Smoke", "--creative", "--seed", "42",
            "--test-terrain", "--offscreen", "--control-socket", str(socket),
            "--render-distance", "3", "--width", "960", "--height", "640",
            "--game-dir", str(game_dir), "--resources", str(args.resources.resolve()),
            "--dev-mod", str(ROOT / "target/content/ae2"),
        ], cwd=game_dir, env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(process)
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            if process.poll() is not None:
                raise RuntimeError(f"game exited {process.returncode}; see {log.name}")
            try:
                ctl("wait-ready")
                return process
            except (RuntimeError, subprocess.TimeoutExpired):
                time.sleep(0.1)
        raise RuntimeError(f"game did not become ready; see {log.name}")

    def slots():
        return ctl("container", 8, 73, 7)["slots"]

    def nbt():
        return "\n".join(ctl("command", "data get block 8 73 7")["chat"])

    try:
        first = launch(1)
        ctl("command", "fill 5 72 3 11 72 8 stone")
        ctl("command", "fill 5 73 3 11 78 8 air")
        ctl("tp", 8.5, 73, 4.5)
        ctl("command", "clear @s")
        ctl("setblock", 8, 73, 7, "ae2:charger[facing=north]")
        ctl("setblock", 8, 74, 7, "ae2:crank[facing=up]")
        ctl("give", "ae2:certus_quartz_crystal", 2)
        ctl("wait", 4)
        ctl("select", "ae2:certus_quartz_crystal")
        ctl("wait", 3)
        ctl("interact", 8, 73, 7)
        ctl("wait", 3)
        assert slots() == [{"count": 1, "item": "ae2:certus_quartz_crystal"}]
        assert ctl("inventory")["counts"]["ae2:certus_quartz_crystal"] == 1
        ctl("select", 9)
        ctl("wait", 3)
        for turn in range(10):
            ctl("interact", 8, 74, 7)
            ctl("wait", 20)
            if turn in (0, 8):
                assert f"internalCurrentPower: {(turn + 1) * 160}.0d" in nbt()
        for _ in range(100):
            if slots() == [{"count": 1, "item": "ae2:charged_certus_quartz_crystal"}]:
                break
            ctl("wait", 20)
        else:
            raise AssertionError("charger did not finish within 2000 ticks")
        assert "internalCurrentPower: 0.0d" in nbt()
        print("PASS: one-item insertion; 160/1440 AE after turns; charged crystal, energy consumed", flush=True)

        ctl("interact", 8, 73, 7)
        ctl("wait", 3)
        assert slots() == [None]
        ctl("select", "ae2:certus_quartz_crystal")
        ctl("wait", 3)
        ctl("interact", 8, 73, 7)
        ctl("select", 9)
        ctl("wait", 3)
        ctl("interact", 8, 74, 7)
        ctl("wait", 20)
        assert "internalCurrentPower: 160.0d" in nbt()
        ctl("save")
        ctl("quit")
        first.wait(timeout=20)
        launch(2)
        assert slots() == [{"count": 1, "item": "ae2:certus_quartz_crystal"}]
        assert "internalCurrentPower: 160.0d" in nbt()
        ctl("look-at", 8.5, 73.5, 7.5)
        ctl("key", "f1", "down")
        ctl("key", "f1", "up")
        ctl("screenshot", str(work / "charger.png"))
        ctl("quit")
        (work / "result.json").write_text(json.dumps({"passed": True, "scope": ["insert", "crank", "recipe", "retrieve", "restart_slot_and_nonzero_energy"], "world": str(work)}, indent=2) + "\n")
        print(f"PASS: raw crystal + 160 AE survive restart; logs: {work}", flush=True)
    finally:
        for process in processes:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=10)
        for handle in log_handles:
            handle.close()
        socket.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
