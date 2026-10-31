#!/usr/bin/env python3
"""Fetch pinned AE2 sources/jar and inventory every Java file for parity tracking."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
LOCK = json.loads((ROOT / "upstream.lock.json").read_text())
SOURCE = ROOT / "target/upstream/ae2"

# A partial implementation never marks a whole Java class as ported.
PARTIAL = {
    "me/cells/BasicCellInventory.java": ("mixins/ae2/storage.rs", "capacity, exact partitions, distribution/void; no fuzzy or component persistence adapter"),
    "items/storage/BasicStorageCell.java": ("mixins/ae2/storage.rs", "tier parameters only; no game item or tooltip adapter"),
    "items/storage/StorageTier.java": ("mixins/ae2/storage.rs", "all five byte and idle drain constants; no component suppliers"),
    "me/energy/StoredEnergyAmount.java": ("mixins/ae2/energy.rs", "storage, simulation, availability events; no grid service adapter"),
    "blockentity/networking/EnergyCellBlockEntity.java": ("mixins/ae2/energy.rs", "display level formula only"),
    "blockentity/misc/ChargerBlockEntity.java": ("mixins/ae2/charger.rs", "recipe branch only; no grid/tool power or config adapter"),
    "blockentity/misc/CrankBlockEntity.java": ("mixins/ae2/charger.rs", "server rotation/charge/hits; no renderer, statistics or client sync"),
    "blockentity/grid/AENetworkedPoweredBlockEntity.java": ("mixins/ae2/charger.rs", "Charger Crankable only; not the networked base class"),
    "block/misc/ChargerBlock.java": ("mixins/ae2/glue.rs", "single item insert/retrieve; no tools, particles, full orientation or exact drop motion"),
    "block/misc/CrankBlock.java": ("mixins/ae2/glue.rs", "click dispatch and charger target; no placement/survival, fake-player rule or animation"),
    "blockentity/misc/ChargerRecipes.java": ("mixins/ae2/glue.rs", "recipe lookup; component-bearing results refused, automation disabled"),
}


PARTIAL.update({
    'me/cells/BasicCellInventory.java': ("mixins/ae2/storage.rs", 'capacity, exact/fuzzy partitions, distribution/void, native contents persistence; no Drive/terminal mount or malformed recovery'),
    'api/stacks/AEKey.java': ("mixins/ae2/keys.rs", 'typed component identity, primary matching, fuzzyEquals; no packet codec or missing-content fallback'),
    'api/stacks/AEItemKey.java': ("mixins/ae2/cells.rs", 'component-aware key creation and item roundtrip; complete prototype normalization and drop/display APIs pending'),
    'api/stacks/AEFluidKey.java': ("mixins/ae2/cells.rs", 'fluid identity and typed generic NBT persistence; no fluid handler or wire codec'),
    'api/stacks/GenericStack.java': ("mixins/ae2/cells.rs", 'id/components/#t/# layout and signed long amounts; wrapped items and missing-content recovery pending'),
    'api/stacks/KeyCounter.java': ("mixins/ae2/keys.rs", 'signed counts, reset, add/remove, zero pruning, fuzzy lookup; deterministic iteration differs from Java identity hashes'),
    'api/stacks/VariantCounter.java': ("mixins/ae2/keys.rs", 'zero-preserving signed counts and fuzzy candidate selection; no lazy zero-deleting iterator'),
    'api/stacks/FuzzySearch.java': ("mixins/ae2/keys.rs", 'exact descending damage intervals; native key ordering instead of Java hash tie-breaker'),
    'api/config/FuzzyMode.java': ("mixins/ae2/keys.rs", 'all five modes, names and Java float breakpoint calculation; no packet enum codec'),
    'util/prioritylist/IPartitionList.java': ("mixins/ae2/storage.rs", 'exact/fuzzy whitelist/blacklist filters'),
    'util/prioritylist/FuzzyPriorityList.java': ("mixins/ae2/storage.rs", 'cell partitions use FuzzySearch interval rules'),
    'items/contents/CellConfig.java': ("mixins/ae2/cells.rs", 'read storage_cell_config_inv; workbench UI and config writer pending'),
    'api/networking/GridFlags.java': ("mixins/ae2/grid.rs", 'all seven flags in pathing core; no native world binding'),
    'api/networking/pathing/ChannelMode.java': ("mixins/ae2/grid.rs", 'infinite/default/x2/x3/x4 capacities; config and saved forced modes pending'),
    'me/GridConnection.java': ("mixins/ae2/grid.rs", 'edge invariants, direction, route/count/finalize; no grid merge priorities or service events'),
    'me/GridNode.java': ("mixins/ae2/grid.rs", 'topology, ancestry/bottlenecks/channel finalize; no world lifecycle, power or ownership persistence'),
    'me/pathfinding/PathingCalculation.java': ("mixins/ae2/grid.rs", 'three-queue BFS, bottlenecks, compressed/multiblock channels, iterative DFS; 2000 original-Java differential cases; no world binding'),
    'me/pathfinding/ControllerValidator.java': ("mixins/ae2/grid.rs", 'controller-only connectivity, seven-block bound and cross rejection'),
    'me/pathfinding/AdHocChannelUpdater.java': ("mixins/ae2/grid.rs", 'node/edge counts in topology core'),
    'me/pathfinding/ChannelFinalizer.java': ("mixins/ae2/grid.rs", 'final counts and changed-id lists; native listeners pending'),
    'me/service/PathingService.java': ("mixins/ae2/grid.rs", 'ad-hoc limits/errors, controller branches, channel power formula; ticking/config/achievements/service integration pending'),
    'me/cells/BasicCellHandler.java': ("mixins/ae2/cells.rs", 'native bytes/types/partition tooltip; content image and handler registry pending'),
    'core/localization/Tooltips.java': ("mixins/ae2/cells.rs", 'cell usage ratio colors and translatable text only'),
})


PARTIAL.update({
    "me/service/EnergyService.java": ("mixins/ae2/power.rs", "provider/requester priorities, simulation, pruning/events, averages, cached power, status delay, threshold IDs; no full world watcher/event integration or addon providers"),
    "me/energy/GridEnergyStorage.java": ("mixins/ae2/power.rs", "25 AE/node default, proportional removal, proxy.e saved shares; config UI integration pending"),
    "me/service/EnergyOverlayGrid.java": ("mixins/ae2/power.rs", "capacity-ordered overlay components, refcounted links, strongest passive generator; QuartzFiber world parts and Java identity tie ordering pending"),
    "me/storage/NetworkStorage.java": ("mixins/ae2/mounts.rs", "mount priorities, two-pass preference, recursive guard, queued changes and deferred listing; 2000 original-Java differential traces; StorageService/provider lifecycle, Drive/terminal world integration pending"),
    "blockentity/networking/EnergyCellBlockEntity.java": ("mixins/ae2/grid_glue.rs", "full-block energy storage and dimension network adapter, internalCurrentPower persistence; fullness rendering/comparator/item energy lifecycle pending"),
    "blockentity/networking/CreativeEnergyCellBlockEntity.java": ("mixins/ae2/power.rs", "infinite provider, priority and world full-block node binding; client visuals pending"),
    "blockentity/grid/AENetworkedBlockEntity.java": ("mixins/ae2/grid_glue.rs", "four full-block kinds register by dimension, adjacency and proxy.e persistence; full node security/events/readiness/other block types pending"),
    "me/InWorldGridNode.java": ("mixins/ae2/network.rs", "adjacent full-block sides, Charger front exclusion, removal/reload/split/merge; multipart/color/security/lifecycle ordering pending"),
    "blockentity/misc/ChargerBlockEntity.java": ("mixins/ae2/glue.rs", "recipe branch plus private buffer refilled from real ME EnergyService, at most 800 AE per doWork; tools/config/sync/filter/visuals pending"),
})


def git(source, *args):
    return subprocess.check_output(["git", "-C", str(source), *args], text=True).strip()


def verify(source):
    if git(source, "rev-parse", "HEAD") != LOCK["revision"]:
        raise ValueError("AE2 source revision differs from upstream.lock.json")
    if git(source, "status", "--porcelain", "--untracked-files=no"):
        raise ValueError("AE2 reference contains modified tracked files")


def fetch(source):
    if not source.exists():
        source.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "clone", "--depth", "1", "--branch", LOCK["ref"],
                        LOCK["repository"], str(source)], check=True)
    verify(source)


def fetch_engines():
    compat = json.loads((ROOT / "compatibility.json").read_text())
    for name, url, revision in [
        ("horizon", compat["horizon_repository"], compat["horizon_revision"]),
        ("horizon-mixins", compat["horizon_mixins_repository"], compat["horizon_mixins_revision"]),
    ]:
        source = ROOT / "target/upstream" / name
        if not source.exists():
            source.parent.mkdir(parents=True, exist_ok=True)
            with tempfile.TemporaryDirectory(prefix=".engine-", dir=source.parent) as temporary:
                stage = Path(temporary) / name
                subprocess.run(["git", "init", str(stage)], check=True)
                subprocess.run(["git", "-C", str(stage), "remote", "add", "origin", url], check=True)
                subprocess.run(["git", "-C", str(stage), "fetch", "--depth", "1", "origin", revision], check=True)
                subprocess.run(["git", "-C", str(stage), "checkout", "--detach", revision], check=True)
                stage.rename(source)
        if git(source, "rev-parse", "HEAD") != revision or git(source, "status", "--porcelain", "--untracked-files=no"):
            raise ValueError(f"{name}: expected clean pinned checkout at {revision}")
        print(f"{name}: {revision}")


def fetch_jar():
    spec = LOCK["jar"]
    target = ROOT / "target/jars" / spec["url"].rsplit("/", 1)[1]
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.exists() and hashlib.sha512(target.read_bytes()).hexdigest() == spec["sha512"]:
        return target
    request = urllib.request.Request(spec["url"], headers={"User-Agent": "horizon-ae2/0.1.0"})
    with urllib.request.urlopen(request, timeout=60) as response:
        data = response.read()
    if len(data) != spec["size"] or hashlib.sha512(data).hexdigest() != spec["sha512"]:
        raise ValueError("AE2 jar hash/size mismatch")
    with tempfile.NamedTemporaryFile(dir=target.parent, delete=False) as handle:
        temporary = Path(handle.name)
        handle.write(data)
    temporary.replace(target)
    return target


def inventory(source):
    verify(source)
    java_root = source / "src/main/java/appeng"
    files = []
    for path in sorted(java_root.rglob("*.java")):
        relative = path.relative_to(java_root).as_posix()
        partial = PARTIAL.get(relative)
        files.append({
            "path": path.relative_to(source).as_posix(),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "family": relative.split("/")[0],
            "status": "partial" if partial else "pending",
            **({"port": partial[0], "scope": partial[1]} if partial else {}),
        })
    resources = {}
    for base in ["src/main/resources", "src/generated/resources"]:
        paths = [p for p in (source / base).rglob("*") if p.is_file()]
        resources[base] = {
            "files": len(paths),
            "recipes": sum("recipe" in p.relative_to(source / base).parts for p in paths),
            "textures": sum(p.suffix == ".png" for p in paths),
        }
    families = {}
    for entry in files:
        family = families.setdefault(entry["family"], {"pending": 0, "partial": 0})
        family[entry["status"]] += 1
    return {"schema": 1, "upstream": LOCK, "meaning": "Source inventory, not runtime registration or a feature completion percentage. Every partial class still has outstanding work.",
            "java_files": len(files), "families": families, "resources": resources, "files": files}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["fetch", "fetch-jar", "fetch-engines", "audit"])
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--check", action="store_true", help="fail if docs/upstream-inventory.json is stale")
    args = parser.parse_args()
    if args.action == "fetch":
        fetch(args.source)
    elif args.action == "fetch-jar":
        print(fetch_jar())
    elif args.action == "fetch-engines":
        fetch_engines()
    else:
        data = inventory(args.source)
        rendered = json.dumps(data, indent=2, sort_keys=True) + "\n"
        out = ROOT / "docs/upstream-inventory.json"
        if args.check:
            if not out.exists() or out.read_text() != rendered:
                raise ValueError("source inventory is stale; run tools/upstream.py audit")
        else:
            out.write_text(rendered)
        print(f"{data['java_files']} Java files; {len(PARTIAL)} partial classes; all others pending")


if __name__ == "__main__":
    main()
