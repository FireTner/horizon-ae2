#!/usr/bin/env python3
"""Prepare local AE2 content from the pinned jar; never install into player resources."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
LOCK = json.loads((ROOT / "upstream.lock.json").read_text())
MARKER = ".horizon-ae2-generated.json"
FACINGS = {"down": (0, -1, 0), "up": (0, 1, 0), "north": (0, 0, -1),
           "south": (0, 0, 1), "west": (-1, 0, 0), "east": (1, 0, 0)}
CRAFTING = ["network/blocks/crystal_processing_charger.json", "network/blocks/crank.json",
            "network/blocks/energy_energy_cell.json", "network/blocks/energy_dense_energy_cell.json",
            "network/blocks/storage_drive.json"]


def write_json(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n")


def ingredient(value):
    if isinstance(value, list):
        return [ingredient(v) for v in value]
    if isinstance(value, dict):
        if "item" in value:
            return value["item"]
        if "tag" in value:
            return "#" + value["tag"]
        raise ValueError(f"unsupported crafting ingredient: {value}")
    return value


def flatten_recipes(content):
    remap = {}
    for folder in sorted((content / "data").glob("*/recipe")):
        plans = []
        destinations = set()
        for path in sorted(folder.rglob("*.json")):
            rel = path.relative_to(folder)
            flat = ".".join(rel.parts)
            if flat in destinations:
                raise ValueError(f"recipe flattening collision: {folder / flat}")
            destinations.add(flat)
            plans.append((path, folder / flat))
            before = f"{folder.parent.name}:{rel.with_suffix('').as_posix()}"
            after = f"{folder.parent.name}:{Path(flat).stem}"
            remap[before] = after
        # Check all destinations before moving anything (a.b/x vs a/b.x).
        for original, target in plans:
            if original != target:
                original.rename(target)
    return remap


def fit(content, jar):
    blocks = content / "data/ae2/horizon/block"
    states = content / "assets/ae2/blockstates"
    # A dispenser supplies six-facing placement and a block entity. Its menu
    # and inventory are intercepted; template redstone remains a parity gap.
    for name in ["charger", "crank", "drive"]:
        write_json(blocks / f"{name}.json", {
            "template": "minecraft:dispenser",
            "properties": {"light": 0, "hardness": 2.2},
        })
    for name in ["energy_cell", "dense_energy_cell", "creative_energy_cell"]:
        write_json(blocks / f"{name}.json", {
            "template": "minecraft:dispenser",
            "properties": {"light": 0, "hardness": 2.2},
        })
        if name != "creative_energy_cell":
            # All five original model files stay in carried assets. The later
            # draw/sync adapter will select fullness; this stage uses level 0.
            write_json(states / f"{name}.json", {"variants": {"": {"model": f"ae2:block/{name}_0"}}})

    for name in ["charger", "drive"]:
        original = json.loads((states / f"{name}.json").read_text())
        variants = {}
        for face in FACINGS:
            selected = original.get("variants", {}).get(f"facing={face},spin=0")
            if selected is None:
                raise ValueError(f"{name} model missing facing={face},spin=0")
            if name == "drive":
                # Upstream's generated drive.json is empty: DriveBakedModel
                # composes the base, cell chassis and LEDs at runtime. Carry
                # the actual base housing now; dynamic cells/LEDs remain open.
                selected = {**selected, "model": "ae2:block/drive/drive_base"}
            variants[f"facing={face}"] = selected
        write_json(states / f"{name}.json", {"variants": variants})
    # CrankBlockEntityRenderer uses a separate handle model. Static preview only
    # for this milestone; the exact animation needs render/sync work.

    with zipfile.ZipFile(jar) as archive:
        for name in CRAFTING:
            path = f"data/ae2/recipe/{name}"
            data = json.loads(archive.read(path))
            data["key"] = {k: ingredient(v) for k, v in data["key"].items()}
            write_json(content / path, data)

    # Common tags absent from the base game, needed by restored early recipes.
    for tag, values in {"ingots/iron": ["minecraft:iron_ingot"],
                        "ingots/copper": ["minecraft:copper_ingot"],
                        "rods/wooden": ["minecraft:stick"]}.items():
        path = content / "data/c/tags/item" / (tag + ".json")
        data = json.loads(path.read_text()) if path.exists() else {"values": []}
        # Preserve optional object entries and existing order.
        data.setdefault("values", [])
        data["values"].extend(v for v in values if v not in data["values"])
        write_json(path, data)

    remap = flatten_recipes(content)
    write_json(content / "ae2-recipe-map.json", remap)
    write_json(content / MARKER, {"schema": 1, "upstream": LOCK["revision"], "jar_sha512": LOCK["jar"]["sha512"]})
    return remap


def carry(jar, resources, port_pack, work):
    jars = work / "pack"
    jars.mkdir()
    shutil.copy2(jar, jars / jar.name)
    run = lambda *args: subprocess.run([str(port_pack), *map(str, args)], check=True)
    run("import", jars, work / "port")
    (work / "port/jars").mkdir(exist_ok=True)
    shutil.copy2(jar, work / "port/jars" / jar.name)
    run("inventory", work / "port", "--offline")
    run("carry", work / "port", "--resources", resources, "ae2")
    return work / "port/mods/ae2/carry"


def prepare(jar, carried, resources, port_pack, out):
    if hashlib.sha512(jar.read_bytes()).hexdigest() != LOCK["jar"]["sha512"]:
        raise ValueError("jar differs from pinned AE2 19.2.18; do not mix source and content versions")
    if out.exists() and not (out / MARKER).is_file():
        raise ValueError(f"refusing to replace a directory not generated by this tool: {out}")
    out.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".ae2-content-", dir=out.parent) as temporary:
        work = Path(temporary)
        source = carried or carry(jar, resources, port_pack, work)
        stage = work / "ae2"
        shutil.copytree(source, stage)
        fit(stage, jar)
        backup = work / "previous"
        if out.exists():
            out.rename(backup)
        try:
            stage.rename(out)
        except BaseException:
            if backup.exists():
                backup.rename(out)
            raise
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, required=True)
    parser.add_argument("--carried", type=Path, help="reuse horizon-port-pack carry output")
    parser.add_argument("--resources", type=Path, help="prepared Horizon 26.3 resources")
    parser.add_argument("--port-pack", type=Path, default=Path(os.environ.get("HORIZON_PORT_PACK", "horizon-port-pack")))
    parser.add_argument("--out", type=Path, default=ROOT / "target/content/ae2")
    args = parser.parse_args()
    if not args.carried and not args.resources:
        parser.error("give --resources or --carried")
    print(prepare(args.jar.resolve(), args.carried.resolve() if args.carried else None,
                  args.resources, args.port_pack, args.out.resolve()))


if __name__ == "__main__":
    main()
