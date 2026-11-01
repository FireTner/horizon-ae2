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

PARTIAL.update({
    "me/service/StorageService.java": ("mixins/ae2/service.rs", "node/global providers, duplicate guards, refresh/unmount, lazy public cache with private baseline, synchronous watcher changes; no Java service oracle, debug dump or world watcher consumers"),
    "me/helpers/InterestManager.java": ("mixins/ae2/service.rs", "key interests and watch-all collections through shared watcher handles; stable ID ordering instead of Java hashes"),
    "me/helpers/StackWatcher.java": ("mixins/ae2/service.rs", "deduplicated interests, watch-all, reset and permanent destroy; native host callbacks and Java oracle pending"),
    "blockentity/storage/DriveBlockEntity.java": ("mixins/ae2/drive_glue.rs", "ten basic cell slots, real network provider, priority, power/channel gating, exact idle costs, packed state, original inv persistence and restart; GUI, dynamic render, full lifecycle/capabilities and other cell types pending"),
    "me/storage/DriveWatcher.java": ("mixins/ae2/drive.rs", "preferred existing stack, simulate/commit and status transitions/dirty tracking; immediate native visual callback delivery pending"),
    "blockentity/AEBaseInvBlockEntity.java": ("mixins/ae2/drive_glue.rs", "Drive-only inv.itemN ItemStack load/save with Horizon Items transport mirror; full base-class inventory/drop/capability hooks pending"),
    "me/cells/BasicCellInventory.java": ("mixins/ae2/storage.rs", "capacity, exact/fuzzy partitions, distribution/void, native persistence and Drive mounts; terminal and malformed/missing-content recovery pending"),
    "me/storage/NetworkStorage.java": ("mixins/ae2/mounts.rs", "priority/preference/queues/recursion, 2000 Java traces; consumed by native StorageService/Drive; ME Chest, terminal and handler filters pending"),
    "blockentity/grid/AENetworkedBlockEntity.java": ("mixins/ae2/grid_glue.rs", "Charger, three energy cells and Drive register by dimension, adjacency and proxy.e; full security/events/readiness/other block kinds pending"),
})

PARTIAL.update({
    "menu/implementations/DriveMenu.java": ("mixins/ae2/guest_slots.rs", "ten authoritative filtered slots, limit one, pickup/drag/hotbar/quick-move and own-window policy; special AEBaseMenu actions and external multiplayer pending"),
    "client/gui/implementations/DriveScreen.java": ("mixins/ae2/gui.rs", "original texture, dimensions, slot/inventory geometry and localized labels; priority button/subscreen and slot ghost icons pending"),
    "menu/slot/RestrictedInputSlot.java": ("mixins/ae2/slot_rules.rs", "STORAGE_CELLS built-in family predicate and Drive slot limit only; addon registry and other slot kinds pending"),
    "menu/AEBaseMenu.java": ("mixins/ae2/guest_quick.rs", "Drive-only forward quick-move order and remote client prediction suppression; bulk actions/security/other menus pending"),
    "api/storage/StorageCells.java": ("mixins/ae2/slot_rules.rs", "isCellHandled built-in cell families for Drive menu only; extensible registry and non-basic native cell inventories pending"),
    "blockentity/storage/DriveBlockEntity.java": ("mixins/ae2/drive_glue.rs", "ten basic cell mounts, priority/online/channel/idle state and inv persistence; filtered GUI, native hopper/dropper filters and pre-extraction flush; dynamic render, external capability/full lifecycle and other cell backends pending"),
    "blockentity/inventory/AppEngCellInventory.java": ("mixins/ae2/automation.rs", "native Drive one-cell insertion, component retention, flush before slot mutation and provider refresh; general inventory API, simulate capability and remaining persistence paths pending"),
})

PARTIAL.update({
    "items/tools/powered/PortableCellItem.java": ("mixins/ae2/cells.rs", "half-byte inventory layout, tier-dependent item type limit, 18 fluid types, 0.5 idle drain, component-preserving Drive backend and capacity tooltip; handheld menus, charge/use/disassembly and complete item rendering pending"),
    "items/storage/BasicStorageCell.java": ("mixins/ae2/cells.rs", "native storage layout, capacity/type/drain parameters and usage tooltip; use/disassembly and full item color/LED behaviour pending"),
    "core/definitions/AEItems.java": ("mixins/ae2/storage.rs", "portable cell factory capacity/type constants only; full programmatic item registration/properties and remaining definitions pending"),
    "me/cells/BasicCellInventory.java": ("mixins/ae2/storage.rs", "basic/portable layouts, capacity, exact/fuzzy partitions, distribution/void and native persistence; 2000 unchanged-Java traces; actual fuzzy matcher/codecs/world not covered by that oracle; malformed/missing recovery pending"),
    "blockentity/storage/DriveBlockEntity.java": ("mixins/ae2/drive_glue.rs", "ten slots mounting twenty basic/portable cell types, priority/online/channel/idle state, inv persistence, filtered GUI/native automation; dynamic render, external capability/full lifecycle and creative/tool backends pending"),
})


PARTIAL.update({
    "me/cells/CreativeCellInventory.java": ("mixins/ae2/backend.rs", "exact configured-key insert/extract, Integer.MAX_VALUE listing, status/idle/nested/preferred and no-op persistence; 2000 original-Java traces; native description and source hash iteration order pending"),
    "me/cells/CreativeCellHandler.java": ("mixins/ae2/cells.rs", "native creative id dispatch and configured inventory construction with original component fields; tooltip image/settings and extensible handler registry pending"),
    "api/storage/cells/StorageCell.java": ("mixins/ae2/backend.rs", "basic/portable/creative backend dispatch for operations, status/drain/listing and creative nested/preferred rules; descriptions and complete native handler/addon surface pending"),
    "items/contents/CellConfig.java": ("mixins/ae2/cells.rs", "read original component, 63-slot/null/trailing limits, basic type filter and creative mixed-type configuration; workbench inventory API and config writer pending"),
    "blockentity/storage/DriveBlockEntity.java": ("mixins/ae2/drive_glue.rs", "ten slots mounting basic/portable/creative cells, priority/online/channel/idle state, inv persistence, filtered GUI/native automation; dynamic render, external capability/full lifecycle and tool backends pending"),
})


PARTIAL.update({
    "items/tools/powered/MatterCannonItem.java": ("mixins/ae2/tool_glue.rs", "512-byte/one-type inventory, first matching positive-weight ammo recipe or PaintBallItem admission, 0.5 idle drain, component-preserving native Drive backend and storable nested tool; charging/shooting/raycast/permissions/rendering and live recipe reload pending"),
    "items/tools/powered/ColorApplicatorItem.java": ("mixins/ae2/tool_filters.rs", "2048-byte/27-type item inventory, vanilla/tagged dyes, snowball and paintball admission, 0.5 drain, native Drive persistence/nesting; charging/painting/cycle/select/render and live tag reload pending"),
    "api/storage/StorageCells.java": ("mixins/ae2/cells.rs", "native dispatch/handled built-in families for all 23 storage items with tool admission context; extensible addon cell-handler registry and full API pending"),
    "me/cells/BasicCellHandler.java": ("mixins/ae2/cells.rs", "finite/basic/portable/tool layouts and capacity/type/partition tooltip; tooltip images, settings and extensible registry pending"),
    "blockentity/storage/DriveBlockEntity.java": ("mixins/ae2/drive_glue.rs", "ten slots mounting all 23 built-in cell handlers, priority/online/channel/idle state, original inv, filtered GUI/native automation; dynamic render, external capability/full lifecycle, data reload and special menu actions pending"),
})


PARTIAL.update({
    "api/upgrades/UpgradeInventory.java": ("mixins/ae2/upgrade_rules.rs", "finite storage host slot/card tables and per-slot installed counts clamped to host maximum; mutation/cache/listener lifecycle and other hosts pending"),
    "api/upgrades/ItemUpgradeInventory.java": ("mixins/ae2/cells.rs", "read original slot/item ItemContainerContents component for finite storage items, preserve on cell save; writer and changed callbacks pending"),
    "init/internal/InitUpgrades.java": ("mixins/ae2/upgrade_rules.rs", "built-in basic/portable/tool storage host card maxima only; other hosts and dynamic addon registration pending"),
})


PARTIAL.update({
    "items/tools/powered/powersink/AEBasePoweredItem.java": ("mixins/ae2/item_energy.rs", "component energy defaults/override, threshold, simulate/modulate inject overflow/extract, capacity clamp/multiplier/reset; 400000 unchanged-Java operations; native Charger consumes it; tooltip/bar/reequip/creative-tab and live config supplier pending"),
    "api/implementations/items/IAEItemPowerStorage.java": ("mixins/ae2/item_energy_glue.rs", "built-in component power bridge, WRITE admission and per-item charge rates; external/addon item power capabilities and full item APIs pending"),
    "items/tools/powered/ChargedStaffItem.java": ("mixins/ae2/item_energy.rs", "8000 AE default capacity and 32 AE/t Charger profile only; combat, lightning and item visuals pending"),
    "items/tools/powered/EntropyManipulatorItem.java": ("mixins/ae2/item_energy.rs", "200000 AE default capacity and 800 AE/t Charger profile only; tool/block/entity interactions, permissions, recipes and visuals pending"),
    "items/tools/powered/WirelessTerminalItem.java": ("mixins/ae2/item_energy_glue.rs", "1600000 AE default, 800 plus 800 per energy card rate, two-slot upgrade read and native Charger power; terminal/network/menu/linking/usage and changed-upgrade callback pending"),
    "items/tools/powered/PortableCellItem.java": ("mixins/ae2/cells.rs", "all ten storage layouts/backends, source components and native Charger (20000 default, 80 plus 80 per energy card); handheld menus/use/disassembly, changed-upgrade capacity callback and full visuals pending"),
    "items/tools/powered/MatterCannonItem.java": ("mixins/ae2/tool_glue.rs", "restricted storage backend and native Charger power (200000 default, 800 plus 800 per energy card); shooting/raycast/permissions, changed-upgrade capacity callback, visuals and live recipe reload pending"),
    "items/tools/powered/ColorApplicatorItem.java": ("mixins/ae2/tool_filters.rs", "restricted storage backend and native Charger power (20000 default, 80 plus 80 per energy card); painting/cycle/select, changed-upgrade capacity callback, visuals and live tag reload pending"),
    "blockentity/misc/ChargerBlockEntity.java": ("mixins/ae2/glue.rs", "source-ordered powered item/local buffer/grid/overflow branch plus recipe/refill, private storage and Crank integration; all 16 built-in power profiles; live config, hopper filter, full inventory/visual/sync API pending"),
    "util/Platform.java": ("mixins/ae2/item_energy_glue.rs", "isChargeable for the 16 built-in WRITE powered items, maximum > 0 and nonempty stack; all other Platform utilities and addon item registry pending"),
    "init/internal/InitUpgrades.java": ("mixins/ae2/upgrade_rules.rs", "built-in basic/portable/tool storage and wireless host card maxima only; other hosts and dynamic addon registration pending"),
})


PARTIAL.update({
    "api/ids/AEComponents.java": ("mixins/ae2/component_registry.rs", "command registry names for six implemented storage/power components and numeric codecs for stored_energy/energy_capacity; remaining component registrations, full registry-aware codec recovery/network schema pending"),
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


def verify_platform(source):
    spec = LOCK["platform_tags"]
    if git(source, "rev-parse", "HEAD") != spec["revision"]:
        raise ValueError("NeoForge tag source differs from upstream.lock.json")
    if git(source, "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("NeoForge tag reference contains local changes")


def fetch_platform():
    source = ROOT / "target/upstream/neoforge"
    spec = LOCK["platform_tags"]
    if not source.exists():
        source.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix=".platform-", dir=source.parent) as temporary:
            stage = Path(temporary) / "neoforge"
            subprocess.run(["git", "init", str(stage)], check=True)
            subprocess.run(["git", "-C", str(stage), "remote", "add", "origin", spec["repository"]], check=True)
            subprocess.run(["git", "-C", str(stage), "fetch", "--depth", "1", "origin", spec["revision"]], check=True)
            subprocess.run(["git", "-C", str(stage), "checkout", "--detach", spec["revision"]], check=True)
            stage.rename(source)
    verify_platform(source)
    return source


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
    parser.add_argument("action", choices=["fetch", "fetch-jar", "fetch-engines", "fetch-platform", "audit"])
    parser.add_argument("--source", type=Path, default=SOURCE)
    parser.add_argument("--check", action="store_true", help="fail if docs/upstream-inventory.json is stale")
    args = parser.parse_args()
    if args.action == "fetch":
        fetch(args.source)
    elif args.action == "fetch-jar":
        print(fetch_jar())
    elif args.action == "fetch-engines":
        fetch_engines()
    elif args.action == "fetch-platform":
        print(fetch_platform())
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
