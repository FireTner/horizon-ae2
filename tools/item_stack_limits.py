#!/usr/bin/env python3
"""Pinned AE2 item registration limits; source audit, not a Java runtime oracle.

Derived from AEItems, EncodedPatternItemBuilder and WrappedGenericStack (LGPL-3.0-or-later;
the builder API is MIT). Vanilla TieredItem sets durability, which also sets limit one.
Only max-stack metadata is ported here, not the remaining item components/actions.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
UNSTACKABLE = frozenset({
    "entropy_manipulator", "wireless_terminal", "wireless_crafting_terminal",
    "charged_staff", "color_applicator", "matter_cannon", "network_tool", "memory_card",
    "creative_storage_cell", "view_cell", "guide", "wrapped_generic_stack",
    "crafting_pattern", "processing_pattern", "smithing_table_pattern", "stonecutting_pattern",
    *(f"spatial_storage_cell_{size}" for size in (2, 16, 128)),
    *(f"{kind}_storage_cell_{tier}k" for kind in ("item", "fluid") for tier in (1, 4, 16, 64, 256)),
    *(f"portable_{kind}_cell_{tier}k" for kind in ("item", "fluid") for tier in (1, 4, 16, 64, 256)),
    *(f"{material}_{tool}" for material in ("certus_quartz", "nether_quartz", "fluix")
      for tool in ("axe", "hoe", "shovel", "pickaxe", "sword")),
    *(f"{material}_{tool}" for material in ("certus_quartz", "nether_quartz")
      for tool in ("wrench", "cutting_knife")),
})


def source_audit(source):
    base = source / "src/main/java/appeng"
    paths = ["core/definitions/AEItems.java", "api/ids/AEItemIds.java",
             "api/crafting/EncodedPatternItemBuilder.java", "items/misc/WrappedGenericStack.java"]
    text = {p: (base / p).read_text() for p in paths}
    for material in ("Quartz", "Fluix"):
        for suffix, vanilla in (("AxeItem", "AxeItem"), ("HoeItem", "HoeItem"),
                                ("SpadeItem", "ShovelItem"), ("PickaxeItem", "PickaxeItem"),
                                ("SwordItem", "SwordItem")):
            path = f"items/tools/{material.lower()}/{material}{suffix}.java"
            body = (base / path).read_text()
            if not re.search(rf"class {material}{suffix} extends {vanilla}\b", body):
                raise ValueError(f"unsupported tool inheritance: {path}")
            paths.append(path)
    ids = dict(re.findall(r'ResourceLocation\s+(\w+)\s*=\s*id\("([^"]+)"\)', text[paths[1]]))
    definitions = text[paths[0]]
    # The pinned definitions are one-line field declarations. Fail closed if the
    # factory/builder assumptions used by this deliberately narrow audit drift.
    for helper in ("makePortableItemCell", "makePortableFluidCell"):
        body = definitions.split(" " + helper + "(", 1)[1].split("\n    }", 1)[0]
        if "p.stacksTo(1)" not in body:
            raise ValueError(f"unsupported {helper} properties")
    if "new Item.Properties().stacksTo(1)" not in text[paths[2]]:
        raise ValueError("unsupported encoded-pattern default properties")
    if "super(properties.stacksTo(1))" not in text[paths[3]]:
        raise ValueError("unsupported wrapped-stack properties")
    tools = ("AxeItem", "HoeItem", "SpadeItem", "PickaxeItem", "SwordItem")
    direct = ("stacksTo(1)", "durability(", "makePortable", "encodedPatternItemBuilder", "WrappedGenericStack>")
    expected = set()
    for row in definitions.splitlines():
        if "public static final ItemDefinition<" not in row:
            continue
        tiered = any(f"{material}{tool}>" in row for material in ("Quartz", "Fluix") for tool in tools)
        if tiered or any(token in row for token in direct):
            key = re.search(r"AEItemIds\.(\w+)", row)
            if key is None or key[1] not in ids:
                raise ValueError(f"unsupported item registration: {row}")
            expected.add(ids[key[1]])
    if expected != UNSTACKABLE:
        raise ValueError(f"source limits differ: missing={sorted(expected - UNSTACKABLE)}, extra={sorted(UNSTACKABLE - expected)}")
    return {"schema": 1, "unstackable_count": len(expected), "default_max_stack": 64,
            "unstackable": sorted(expected),
            "source_sha256": {p: hashlib.sha256((base / p).read_bytes()).hexdigest() for p in paths},
            "platform_evidence": ["https://docs.neoforged.net/docs/1.21.1/items/",
                                  "https://docs.neoforged.net/docs/1.21.1/items/tools/"],
            "scope": "static pinned registration audit; TieredItem durability implies limit one; not a runtime item-properties/world oracle"}


def main():
    import upstream
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=upstream.SOURCE)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    upstream.verify(args.source)
    report = {"revision": upstream.LOCK["revision"], **source_audit(args.source)}
    path = ROOT / "docs/item-stack-source-audit.json"
    data = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.check:
        if not path.is_file() or path.read_text() != data:
            raise ValueError("item stack source audit is stale; run tools/item_stack_limits.py")
    else:
        path.write_text(data)
    print(f"{len(UNSTACKABLE)} unstackable AE2 registrations match pinned source")


if __name__ == "__main__":
    main()
