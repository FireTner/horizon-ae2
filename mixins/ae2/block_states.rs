// SPDX-License-Identifier: LGPL-3.0-or-later
// Complete the owned effective resource root with the original AE2 cube states.
// The content API initially contributes one iron-block row. No synthetic blocks
// or template states enter the resulting registry, and vanilla ids never move.
use super::{ContentMod, Result, ae2_controller};
use horizon_world::nbt::{self, Compound, JavaString, Limits, Tag};
use serde_json::{Value, json};
use std::{cell::Cell, fs, path::Path};

thread_local! {
    static BUILDING: Cell<bool> = const { Cell::new(false) };
}
struct BuildGuard;
impl Drop for BuildGuard {
    fn drop(&mut self) {
        BUILDING.with(|flag| flag.set(false));
    }
}

pub(super) fn build(
    base: &Path,
    mods: &[ContentMod],
    out: &Path,
) -> core::ops::ControlFlow<Result<Vec<String>>> {
    let declarations: Vec<_> = mods
        .iter()
        .flat_map(|m| &m.content.blocks)
        .filter_map(|block| Definition::from_block(&block.id).map(|kind| (kind, block)))
        .collect();
    if declarations.is_empty() {
        return core::ops::ControlFlow::Continue(());
    }
    if BUILDING.with(|flag| flag.replace(true)) {
        return core::ops::ControlFlow::Continue(());
    }
    let _guard = BuildGuard;
    let result = (|| {
        for (_, block) in &declarations {
            if block.template != "minecraft:iron_block"
                || block.shapes.is_some()
                || !block.default_state.is_empty()
            {
                return Err(format!(
                    "AE2 state adapter requires its pinned single-row iron template: {}",
                    block.id
                )
                .into());
            }
        }
        let warnings = super::build(base, mods, out)?;
        expand(
            out,
            &declarations
                .iter()
                .map(|(kind, _)| *kind)
                .collect::<Vec<_>>(),
        )?;
        Ok(warnings)
    })();
    core::ops::ControlFlow::Break(result)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Definition {
    Controller,
    EnergyCell,
    DenseEnergyCell,
    CreativeEnergyCell,
}
impl Definition {
    fn from_block(id: &str) -> Option<Self> {
        match id {
            "ae2:controller" => Some(Self::Controller),
            "ae2:energy_cell" => Some(Self::EnergyCell),
            "ae2:dense_energy_cell" => Some(Self::DenseEnergyCell),
            "ae2:creative_energy_cell" => Some(Self::CreativeEnergyCell),
            _ => None,
        }
    }
    fn block(self) -> &'static str {
        match self {
            Self::Controller => ae2_controller::BLOCK,
            Self::EnergyCell => "ae2:energy_cell",
            Self::DenseEnergyCell => "ae2:dense_energy_cell",
            Self::CreativeEnergyCell => "ae2:creative_energy_cell",
        }
    }
    fn variants(self) -> usize {
        match self {
            Self::Controller => ae2_controller::VARIANTS,
            Self::CreativeEnergyCell => 1,
            _ => 5,
        }
    }
    fn properties(self, index: usize) -> Value {
        match self {
            Self::Controller => properties(index),
            Self::CreativeEnergyCell => json!({}),
            _ => json!({"fullness": index.to_string()}),
        }
    }
    fn schema(self) -> Value {
        match self {
            Self::Controller => {
                json!({"state":{"named":ae2_controller::STATES}, "type":{"named":ae2_controller::TYPES}})
            }
            Self::CreativeEnergyCell => json!({}),
            _ => json!({"fullness":{"integer":[0,4]}}),
        }
    }
    fn commands(self) -> Value {
        match self {
            Self::Controller => json!([
                ["state", ae2_controller::STATES],
                ["type", ae2_controller::TYPES]
            ]),
            Self::CreativeEnergyCell => json!([]),
            _ => json!([["fullness", ["0", "1", "2", "3", "4"]]]),
        }
    }
}

#[derive(Clone, Copy)]
struct Layout {
    first: usize,
    old_len: usize,
    kind: Definition,
}
impl Layout {
    fn extra(self) -> usize {
        self.kind.variants() - 1
    }
    fn id(self, old: u64) -> u64 {
        if old > self.first as u64 && old < self.old_len as u64 {
            old + self.extra() as u64
        } else {
            old
        }
    }
    fn remap(self, value: &mut Value) {
        match value {
            Value::Number(n) => {
                if let Some(id) = n.as_u64() {
                    *n = self.id(id).into();
                }
            }
            Value::Array(a) => {
                for v in a {
                    self.remap(v);
                }
            }
            Value::Object(o) => {
                for v in o.values_mut() {
                    self.remap(v);
                }
            }
            _ => {}
        }
    }
    fn rows(self, value: &mut Value) -> Result<()> {
        let rows = value
            .as_array_mut()
            .ok_or("AE2 state table is not an array")?;
        if rows.len() != self.old_len {
            return Err("AE2 state table lengths disagree".into());
        }
        let original = rows[self.first].clone();
        rows.splice(
            self.first..=self.first,
            std::iter::repeat_n(original, self.kind.variants()),
        );
        Ok(())
    }
}

fn properties(index: usize) -> Value {
    json!({"state": ae2_controller::STATES[index / ae2_controller::TYPES.len()],
           "type": ae2_controller::TYPES[index % ae2_controller::TYPES.len()]})
}
fn read_json(root: &Path, name: &str) -> Result<Value> {
    Ok(serde_json::from_slice(&owned(root, name)?)?)
}
fn owned(root: &Path, name: &str) -> Result<Vec<u8>> {
    let path = root.join(name);
    if fs::symlink_metadata(&path)?.file_type().is_symlink() {
        return Err(format!("refusing to rewrite linked base resource {name}").into());
    }
    Ok(fs::read(path)?)
}

fn expand(root: &Path, definitions: &[Definition]) -> Result<()> {
    let facts = read_json(root, "facts.json")?;
    let rows = facts["states"].as_array().ok_or("no facts states")?;
    let mut initial = Vec::new();
    for &kind in definitions {
        let matches: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r[0].as_str() == Some(kind.block()))
            .map(|(i, _)| i)
            .collect();
        if matches.len() != 1 {
            return Err(format!("expected one initial state: {}", kind.block()).into());
        }
        initial.push((matches[0], kind));
    }
    initial.sort_by_key(|(id, _)| *id);
    let mut offset = 0;
    let mut layouts = Vec::new();
    for (first, kind) in initial {
        let layout = Layout {
            first: first + offset,
            old_len: rows.len() + offset,
            kind,
        };
        offset += layout.extra();
        layouts.push(layout);
    }
    if rows.len() + offset > super::MAX_STATES as usize {
        return Err("AE2 states exceed the engine palette limit".into());
    }
    let mut outputs = Vec::new();
    for name in [
        "facts.json",
        "gameplay-facts.json",
        "block-behaviours.json",
        "model-states.json",
        "render-definitions.json",
        "command-registries.json",
        "mob-data.json",
        "worldgen/block_flags.json",
        "worldgen/blocks/structure-blocks.json",
    ] {
        let mut doc = read_json(root, name)?;
        for &layout in &layouts {
            rewrite_json(layout, name, &mut doc)?;
        }
        outputs.push((name, serde_json::to_vec(&doc)?));
    }
    for name in [
        "occlusion.nbt",
        "cached-collision.nbt",
        "movement-properties.nbt",
        "worldgen/blocks/block-properties.nbt",
        "worldgen/blocks/cached-support.nbt",
        "worldgen/blocks/cached-collision.nbt",
    ] {
        let bytes = owned(root, name)?;
        let (mut named, _) = nbt::read_named(&bytes, Limits::tables())?;
        for &layout in &layouts {
            if rewrite_nbt(layout, &mut named.tag)? == 0 {
                return Err(format!("no state arrays expanded in {name}").into());
            }
        }
        outputs.push((name, nbt::write_named(&named, Limits::tables())?));
    }
    // Validate/encode every table before changing any of the owned files. The
    // engine writes its resource stamp only after this whole build succeeds.
    for (name, bytes) in outputs {
        fs::write(root.join(name), bytes)?;
    }
    Ok(())
}

fn rewrite_json(layout: Layout, name: &str, doc: &mut Value) -> Result<()> {
    match name {
        "facts.json" => {
            layout.rows(&mut doc["states"])?;
            for i in 0..layout.kind.variants() {
                doc["states"][layout.first + i][4]["properties"] = layout.kind.properties(i);
            }
            layout.remap(&mut doc["defaults"]);
        }
        "gameplay-facts.json" => {
            layout.rows(&mut doc["states"])?;
            let rows = doc["states"].as_array_mut().unwrap();
            for (id, row) in rows.iter_mut().enumerate().skip(layout.first) {
                row["id"] = json!(id);
            }
            for row in &mut rows[layout.first..layout.first + layout.kind.variants()] {
                row["block_entity"] = json!(true);
            }
        }
        "block-behaviours.json" => {
            layout.rows(&mut doc["states"])?;
            for i in 0..layout.kind.variants() {
                let row = &mut doc["states"][layout.first + i];
                let flags = row[1].as_u64().ok_or("no behaviour flags")?;
                row[1] = json!(
                    flags
                        | if layout.kind != Definition::CreativeEnergyCell {
                            u64::from(horizon_world::level::data::state_flags::ANALOG_OUTPUT)
                        } else {
                            0
                        }
                );
            }
        }
        "mob-data.json" => layout.rows(&mut doc["states"])?,
        "worldgen/block_flags.json" => layout.rows(doc)?,
        "model-states.json" => {
            doc[layout.kind.block()]["schema"] = layout.kind.schema();
            doc[layout.kind.block()]["states"] = Value::Array(
                (0..layout.kind.variants())
                    .map(|i| layout.kind.properties(i))
                    .collect(),
            );
        }
        "render-definitions.json" => {
            layout.remap(doc);
            doc[layout.kind.block()]["states"] =
                json!((layout.first..layout.first + layout.kind.variants()).collect::<Vec<_>>());
        }
        "command-registries.json" => {
            doc["blocks"][layout.kind.block()] = layout.kind.commands();
        }
        "worldgen/blocks/structure-blocks.json" => {
            for key in ["rotate", "mirror"] {
                layout.remap(&mut doc[key]);
                layout.rows(&mut doc[key])?;
                for i in 0..layout.kind.variants() {
                    let row = doc[key][layout.first + i]
                        .as_array_mut()
                        .ok_or("no structure rotation row")?;
                    for id in row {
                        *id = json!(layout.first + i);
                    } // Source orientation strategy is NONE.
                }
            }
            layout.rows(&mut doc["full_block"])?;
            doc["states"] = json!(layout.old_len + layout.extra());
        }
        _ => return Err("unhandled AE2 state table".into()),
    }
    Ok(())
}

fn rewrite_nbt(layout: Layout, tag: &mut Tag) -> Result<usize> {
    let Tag::Compound(compound) = tag else {
        return Ok(0);
    };
    let mut changed = 0;
    for (key, value) in compound.iter_mut() {
        match (key.to_string_lossy().as_str(), value) {
            ("states", Tag::List(rows)) if rows.len() == layout.old_len => {
                changed += 1;
                let original = rows[layout.first].clone();
                rows.splice(
                    layout.first..=layout.first,
                    std::iter::repeat_n(original, layout.kind.variants()),
                );
                for (i, row) in rows[layout.first..layout.first + layout.kind.variants()]
                    .iter_mut()
                    .enumerate()
                {
                    if let Tag::Compound(row) = row
                        && row.get(&JavaString::from("id"))
                            == Some(&Tag::String(layout.kind.block().into()))
                    {
                        let props: Compound = layout
                            .kind
                            .properties(i)
                            .as_object()
                            .unwrap()
                            .iter()
                            .map(|(key, value)| {
                                (
                                    key.as_str().into(),
                                    Tag::String(value.as_str().unwrap().into()),
                                )
                            })
                            .collect();
                        row.insert("properties".into(), Tag::Compound(props));
                    }
                }
            }
            ("cached_shapes", Tag::IntArray(rows)) if rows.len() == layout.old_len => {
                changed += 1;
                let original = rows[layout.first];
                rows.splice(
                    layout.first..=layout.first,
                    std::iter::repeat_n(original, layout.kind.variants()),
                );
            }
            (_, inner @ Tag::Compound(_)) => changed += rewrite_nbt(layout, inner)?,
            _ => {}
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_mods::Content;
    use std::path::PathBuf;

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            let parent = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_owned();
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = parent.join(format!(
                "ae2-controller-states-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn content(root: &Path, id: &str, declarations: &[(&str, Value)]) -> ContentMod {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join("horizon-mod.toml"), format!("id=\"{id}\"\nname=\"Test\"\nversion=\"0.1.0\"\napi=\"0.2\"\nside=\"both\"\ncapabilities=[\"registries\"]\n")).unwrap();
        for (block, data) in declarations {
            let path = root.join(format!("data/{id}/horizon/block/{block}.json"));
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, serde_json::to_vec(data).unwrap()).unwrap();
        }
        let package = horizon_mods::Package::read(root).unwrap();
        ContentMod {
            id: id.to_owned(),
            content: Content::read(root, &package.manifest).unwrap(),
        }
    }

    #[test]
    fn owned_controller_states_keep_vanilla_ids_and_foreign_shape_mappings() {
        let scratch = Scratch::new();
        let base = PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap());
        let before = fs::read(base.join("facts.json")).unwrap();
        let baseline: Value = serde_json::from_slice(&before).unwrap();
        let ae2 = content(
            &scratch.0.join("ae2"),
            "ae2",
            &[
                ("aaa_before", json!({"template":"minecraft:glass"})),
                (
                    "controller",
                    json!({"template":"minecraft:iron_block","properties":{"hardness":6,"requires_tool":false}}),
                ),
                (
                    "creative_energy_cell",
                    json!({"template":"minecraft:iron_block"}),
                ),
                (
                    "dense_energy_cell",
                    json!({"template":"minecraft:iron_block"}),
                ),
                ("energy_cell", json!({"template":"minecraft:iron_block"})),
                ("zzz_door", json!({"template":"minecraft:oak_door"})),
            ],
        );
        let foreign = content(
            &scratch.0.join("foreign"),
            "zztest",
            &[(
                "shaped",
                json!({"template":"minecraft:stone_button", "default_state":{"powered":"true"},
                "shapes":{"property":"powered","values":{"false":[[0,0,0,0.5,0.5,0.5]],"true":[[0,0,0,1,1,1]]}}}),
            )],
        );
        let out = scratch.0.join("root");
        super::super::build(&base, &[foreign, ae2], &out).unwrap();
        let facts = read_json(&out, "facts.json").unwrap();
        let rows = facts["states"].as_array().unwrap();
        let old = baseline["states"].as_array().unwrap();
        assert_eq!(
            &rows[..old.len()],
            old.as_slice(),
            "vanilla state ids/rows never move"
        );
        assert_eq!(fs::read(base.join("facts.json")).unwrap(), before);
        for (name, value) in baseline["defaults"].as_object().unwrap() {
            assert_eq!(&facts["defaults"][name], value);
        }
        let controller: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r[0] == ae2_controller::BLOCK)
            .collect();
        assert_eq!(controller.len(), 18);
        for (i, (_, row)) in controller.iter().enumerate() {
            assert_eq!(row[4]["properties"], properties(i));
        }
        assert_eq!(
            facts["defaults"][ae2_controller::BLOCK],
            json!(controller[0].0)
        );
        let model = read_json(&out, "model-states.json").unwrap();
        assert_eq!(
            model[ae2_controller::BLOCK]["states"]
                .as_array()
                .unwrap()
                .len(),
            18
        );
        let command = read_json(&out, "command-registries.json").unwrap();
        assert_eq!(
            command["blocks"][ae2_controller::BLOCK],
            json!([
                ["state", ae2_controller::STATES],
                ["type", ae2_controller::TYPES]
            ])
        );
        let gameplay = read_json(&out, "gameplay-facts.json").unwrap();
        for (id, _) in &controller {
            assert_eq!(gameplay["states"][*id]["block_entity"], true);
        }
        for kind in [
            Definition::EnergyCell,
            Definition::DenseEnergyCell,
            Definition::CreativeEnergyCell,
        ] {
            let cell: Vec<_> = rows
                .iter()
                .enumerate()
                .filter(|(_, r)| r[0] == kind.block())
                .collect();
            assert_eq!(cell.len(), kind.variants());
            assert_eq!(facts["defaults"][kind.block()], cell[0].0);
            assert_eq!(model[kind.block()]["schema"], kind.schema());
            assert_eq!(command["blocks"][kind.block()], kind.commands());
            let behaviour = read_json(&out, "block-behaviours.json").unwrap();
            for (i, (id, row)) in cell.iter().enumerate() {
                assert_eq!(row[4]["properties"], kind.properties(i));
                assert_eq!(gameplay["states"][*id]["block_entity"], true);
                let flags = behaviour["states"][*id][1].as_u64().unwrap();
                assert_eq!(
                    flags & u64::from(horizon_world::level::data::state_flags::ANALOG_OUTPUT) != 0,
                    kind != Definition::CreativeEnergyCell
                );
            }
        }
        let (named, _) = nbt::read_named(
            &fs::read(out.join("cached-collision.nbt")).unwrap(),
            Limits::tables(),
        )
        .unwrap();
        let Tag::Compound(collision) = named.tag else {
            panic!("no collision root");
        };
        let Some(Tag::IntArray(mapping)) = collision.get(&JavaString::from("cached_shapes")) else {
            panic!("no mapping");
        };
        assert_eq!(mapping.len(), rows.len());
        let Some(Tag::List(shapes)) = collision.get(&JavaString::from("shapes")) else {
            panic!("no shapes");
        };
        let shaped: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r[0] == "zztest:shaped")
            .collect();
        assert_eq!(shaped.len(), 24);
        for (i, (id, _)) in shaped.iter().enumerate() {
            assert_eq!(mapping[*id] as usize, shapes.len() - 24 + i);
        }
        // Run the actual loaders too: all of the tables must agree, not merely
        // their lengths in a hand-written fixture.
        let assets = crate::assets::Assets::load(&out).unwrap();
        assert_eq!(assets.registry.len(), rows.len());
        let state = assets
            .registry
            .default_state(ae2_controller::BLOCK)
            .unwrap();
        assert_eq!(state.0 as usize, controller[0].0);
        let mut sprites = horizon_client_render::sprites::Sprites::default();
        let models = horizon_client_render::models::catalog::StateModels::load(
            &out,
            assets.registry,
            &[
                ae2_controller::BLOCK,
                "ae2:energy_cell",
                "ae2:dense_energy_cell",
                "ae2:creative_energy_cell",
                "ae2:zzz_door",
                "zztest:shaped",
            ],
            &mut sprites,
        )
        .unwrap();
        for (id, row) in rows
            .iter()
            .enumerate()
            .filter(|(_, row)| Definition::from_block(row[0].as_str().unwrap()).is_some())
        {
            let _ = row;
            assert!(
                !models
                    .collect(
                        horizon_world::registry::BlockStateId(id as u32),
                        horizon_core::position::BlockPos::new(0, 0, 0)
                    )
                    .unwrap()
                    .is_empty()
            );
        }
    }
    #[test]
    fn energy_cell_only_content_builds_without_controller_or_template_states() {
        let scratch = Scratch::new();
        let base = PathBuf::from(std::env::var_os("HORIZON_RESOURCES").unwrap());
        let ae2 = content(
            &scratch.0.join("ae2"),
            "ae2",
            &[("energy_cell", json!({"template":"minecraft:iron_block"}))],
        );
        let out = scratch.0.join("root");
        super::super::build(&base, &[ae2], &out).unwrap();
        let facts = read_json(&out, "facts.json").unwrap();
        let states: Vec<_> = facts["states"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row[0] == "ae2:energy_cell")
            .collect();
        assert_eq!(states.len(), 5);
        assert!(
            states
                .iter()
                .enumerate()
                .all(|(i, row)| row[4]["properties"] == Definition::EnergyCell.properties(i))
        );
        crate::assets::Assets::load(&out).unwrap();
    }
}
