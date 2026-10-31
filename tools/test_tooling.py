#!/usr/bin/env python3
"""Regression tests for resource conversion, publication boundaries and packaging."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
import tomllib
import unittest
from unittest.mock import patch
import zipfile

import package
import prepare_content as content


class ContentTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def fixture(self):
        carried = self.root / "carry"
        content.write_json(carried / "assets/ae2/blockstates/charger.json", {
            "variants": {f"facing={face},spin=0": {"model": "ae2:block/charger"} for face in content.FACINGS}
        })
        content.write_json(carried / "assets/ae2/blockstates/drive.json", {
            "variants": {f"facing={face},spin=0": {"model": "ae2:block/drive"} for face in content.FACINGS}
        })
        content.write_json(carried / "data/ae2/recipe/charger/a.json", {
            "type": "ae2:charger", "ingredient": {"item": "ae2:certus_quartz_crystal"},
            "result": {"id": "ae2:charged_certus_quartz_crystal"}
        })
        content.write_json(carried / "data/c/tags/item/ingots/iron.json", {
            "replace": False, "values": [{"id": "other:iron", "required": False}]
        })
        jar = self.root / "test.jar"
        with zipfile.ZipFile(jar, "w") as z:
            for name in content.CRAFTING:
                z.writestr("data/ae2/recipe/" + name, json.dumps({
                    "type": "minecraft:crafting_shaped", "key": {"a": {"tag": "c:ingots/iron"}},
                    "pattern": ["a"], "result": {"id": "ae2:charger", "count": 1}
                }))
        return carried, jar

    def test_flatten_refuses_collision_before_mutating_files(self):
        first = self.root / "data/ae2/recipe/a/b.json"
        second = self.root / "data/ae2/recipe/a.b.json"
        content.write_json(first, {"first": True})
        content.write_json(second, {"second": True})
        with self.assertRaisesRegex(ValueError, "collision"):
            content.flatten_recipes(self.root)
        self.assertTrue(first.exists())
        self.assertTrue(second.exists())

    def test_custom_recipes_kept_and_crafting_normalized_with_id_map(self):
        carried, jar = self.fixture()
        remap = content.fit(carried, jar)
        self.assertEqual(remap["ae2:charger/a"], "ae2:charger.a")
        data = json.loads((carried / "data/ae2/recipe/charger.a.json").read_text())
        self.assertEqual(data["ingredient"], {"item": "ae2:certus_quartz_crystal"})
        restored = json.loads((carried / "data/ae2/recipe/network.blocks.crank.json").read_text())
        self.assertEqual(restored["key"]["a"], "#c:ingots/iron")
        tag = json.loads((carried / "data/c/tags/item/ingots/iron.json").read_text())
        self.assertEqual(tag["values"], [{"id": "other:iron", "required": False}, "minecraft:iron_ingot"])
        self.assertFalse(tag["replace"])

    def test_native_hosts_have_block_entities_and_valid_six_facing_models(self):
        carried, jar = self.fixture()
        content.fit(carried, jar)
        for name in ["charger", "crank", "drive", "energy_cell", "dense_energy_cell", "creative_energy_cell"]:
            block = json.loads((carried / f"data/ae2/horizon/block/{name}.json").read_text())
            self.assertEqual(block["template"], "minecraft:dispenser")
        states = json.loads((carried / "assets/ae2/blockstates/charger.json").read_text())
        self.assertEqual(set(states["variants"]), {f"facing={face}" for face in content.FACINGS})
        drive = json.loads((carried / "assets/ae2/blockstates/drive.json").read_text())
        self.assertEqual(set(drive["variants"]), {f"facing={face}" for face in content.FACINGS})
        self.assertTrue(all(v["model"] == "ae2:block/drive/drive_base" for v in drive["variants"].values()))

    def test_wrong_jar_is_rejected_without_touching_output(self):
        carried, jar = self.fixture()
        out = self.root / "out"
        out.mkdir()
        (out / "sentinel").write_text("keep")
        with self.assertRaisesRegex(ValueError, "jar differs"):
            content.prepare(jar, carried, None, None, out)
        self.assertEqual((out / "sentinel").read_text(), "keep")

    def test_unowned_output_is_never_replaced(self):
        carried, jar = self.fixture()
        out = self.root / "out"
        out.mkdir()
        lock = {**content.LOCK, "jar": {"sha512": hashlib.sha512(jar.read_bytes()).hexdigest()}}
        with patch.object(content, "LOCK", lock), self.assertRaisesRegex(ValueError, "not generated"):
            content.prepare(jar, carried, None, None, out)

    def test_failed_conversion_does_not_replace_previous_generation(self):
        carried, jar = self.fixture()
        out = self.root / "out"
        out.mkdir()
        (out / content.MARKER).write_text("{}")
        (out / "sentinel").write_text("keep")
        (carried / "assets/ae2/blockstates/charger.json").write_text('{"variants":{}}')
        lock = {**content.LOCK, "jar": {"sha512": hashlib.sha512(jar.read_bytes()).hexdigest()}}
        with patch.object(content, "LOCK", lock), self.assertRaisesRegex(ValueError, "model missing"):
            content.prepare(jar, carried, None, None, out)
        self.assertEqual((out / "sentinel").read_text(), "keep")


class PackageTests(unittest.TestCase):
    def test_source_bundle_is_deterministic_licensed_and_within_horizon_path_contract(self):
        with contextlib.redirect_stdout(io.StringIO()):
            package.package()
            out = package.ROOT / "target/ae2-0.1.0.hnmod"
            first = out.read_bytes()
            package.package()
        self.assertEqual(first, out.read_bytes())
        with zipfile.ZipFile(out) as z:
            manifest = json.loads(z.read("horizon-native.json"))
            self.assertEqual(manifest["id"], "ae2")
            prefix = "mixins/ae2/"
            for name in z.namelist():
                if name == "horizon-native.json":
                    continue
                # Exact admissible path contract in Horizon index/native.rs.
                self.assertTrue(name.startswith(prefix))
                self.assertTrue(name.endswith((".rs", ".toml")) or name.startswith(prefix + "data/"))
                self.assertNotIn("..", name.split("/"))
                self.assertFalse(name.endswith((".png", ".jar", ".class")))
            self.assertIn(b"LESSER GENERAL PUBLIC LICENSE", z.read(prefix + "data/source/LICENSE"))
            self.assertIn(b"MIT License", z.read(prefix + "data/source/tools/LICENSE-MIT"))
            self.assertIn(prefix + "data/source/src/lib.rs", z.namelist())
            # Corresponding source must be runnable as its own Cargo tree.
            self.assertEqual(z.read(prefix + "data/source/mixins/ae2/charger.rs"),
                             z.read(prefix + "charger.rs"))

    def test_bundle_pins_match_and_all_injections_have_helpers(self):
        root = package.ROOT
        mixin = tomllib.loads((root / "mixins/ae2/horizon-mixin.toml").read_text())
        compat = json.loads((root / "compatibility.json").read_text())
        self.assertEqual(mixin["compatibility"]["horizon_revision"], compat["horizon_revision"])
        for declaration in mixin["module"]:
            self.assertTrue((root / "mixins/ae2" / declaration["source"]).is_file())
        for injection in mixin["injection"]:
            self.assertTrue((root / "mixins/ae2" / injection["helper"]).is_file())


if __name__ == "__main__":
    unittest.main(verbosity=2)
