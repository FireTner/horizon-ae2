// SPDX-License-Identifier: LGPL-3.0-or-later
// Horizon adapter for the first Charger/Crank vertical slice. See docs/PARITY.md
// for the explicit gaps; full-block network energy lives in ae2_grid_glue.

use super::Game;
use super::ae2_charger::{self as rules, Crank, CrankResult};
use horizon_client::containers::{ModBlockEntity, ModContainer};
use horizon_client::crafting::{Ingredient, recipe_result};
use horizon_client::items::ItemStack;
use horizon_core::random::RandomSource;
use horizon_world::nbt::{Compound, JavaString, Tag};

const CHARGER: &str = "ae2:charger";
const CRANK: &str = "ae2:crank";
type Pos = [i32; 3];

pub(super) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && (matches!(block, CHARGER | CRANK) || super::ae2_grid_glue::owns(block)) {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub(super) fn ticks(block: &str, ticks: &mut bool) {
    if matches!(block, CHARGER | CRANK) {
        *ticks = true;
    }
}

pub(super) fn container(block: &str, out: &mut Option<ModContainer>) {
    if block == "ae2:drive" {
        // The wire menu carries 18 slots; the native screen and server hide
        // 10..18. Only the original ten slots are mounted as storage cells.
        *out = Some(ModContainer {
            slots: 18,
            faces: std::array::from_fn(|_| vec![]),
            inputs: vec![],
            outputs: vec![],
            menu: Some("minecraft:generic_9x2".to_owned()),
        });
    }
    if block == CHARGER {
        // Dynamic per-stack hopper filters do not exist at API 40. Disable
        // automation until the engine can enforce ChargerInvFilter correctly.
        *out = Some(ModContainer {
            slots: 1,
            faces: std::array::from_fn(|_| vec![]),
            inputs: vec![],
            outputs: vec![],
            menu: None,
        });
    }
}

fn energy(nbt: &Compound) -> f64 {
    match nbt.get(&JavaString::from("internalCurrentPower")) {
        Some(Tag::Double(v)) if f64::from_bits(*v).is_finite() => {
            f64::from_bits(*v).clamp(0.0, rules::POWER_MAXIMUM)
        }
        _ => 0.0,
    }
}

fn set_energy(nbt: &mut Compound, value: f64) {
    nbt.insert(
        JavaString::from("internalCurrentPower"),
        Tag::Double(value.to_bits()),
    );
}

// Upstream leaves rotation/charge/hits transient. Horizon's `data` slots are
// transient too, unlike its NBT compound. This avoids resuming turns after load.
fn read_crank(be: &ModBlockEntity) -> Crank {
    Crank {
        rotation: be.data.first().copied().unwrap_or(0),
        charge: be.data.get(1).copied().unwrap_or(0),
        hits: be.data.get(2).copied().unwrap_or(0).max(0) as u32,
    }
}
fn write_crank(be: &mut ModBlockEntity, crank: Crank) {
    be.data = vec![crank.rotation, crank.charge, crank.hits as i32];
}

fn facing(game: &Game<'_>, pos: Pos) -> Pos {
    match game.native_block_property(pos, "facing").as_deref() {
        Some("down") => [0, -1, 0],
        Some("up") => [0, 1, 0],
        Some("south") => [0, 0, 1],
        Some("west") => [-1, 0, 0],
        Some("east") => [1, 0, 0],
        _ => [0, 0, -1],
    }
}

fn crank_target(game: &Game<'_>, pos: Pos) -> Option<Pos> {
    let dir = facing(game, pos);
    let target = std::array::from_fn(|i| pos[i] - dir[i]);
    (game.native_block(target) == CHARGER && facing(game, target) != dir).then_some(target)
}

fn ingredient_json(value: &serde_json::Value) -> Option<serde_json::Value> {
    use serde_json::Value;
    match value {
        Value::String(_) => Some(value.clone()),
        Value::Object(o) => o
            .get("item")
            .and_then(Value::as_str)
            .map(|s| Value::String(s.to_owned()))
            .or_else(|| {
                o.get("tag")
                    .and_then(Value::as_str)
                    .map(|s| Value::String(format!("#{s}")))
            }),
        Value::Array(a) => Some(Value::Array(a.iter().filter_map(ingredient_json).collect())),
        _ => None,
    }
}

fn recipe(game: &Game<'_>, stack: &ItemStack) -> Option<ItemStack> {
    game.recipes()
        .custom
        .get("ae2:charger")?
        .iter()
        .find_map(|r| {
            // GuideME results contain components that are not ported. Refuse them
            // rather than silently manufacturing an invalid book or tool.
            if r.json["result"].get("components").is_some() {
                return None;
            }
            let value = ingredient_json(&r.json["ingredient"])?;
            let matcher = Ingredient::parse(&value, game.items())?;
            matcher
                .test(stack)
                .then(|| recipe_result(&r.json["result"]))?
        })
}

pub(super) fn tick(game: &mut Game<'_>, pos: Pos, be: &mut ModBlockEntity) {
    match be.id.as_str() {
        CRANK => {
            let mut crank = read_crank(be);
            if crank.tick()
                && let Some(target) = crank_target(game, pos)
            {
                game.native_with_mod_block_entity(target, |target| {
                    if target.id == CHARGER {
                        let value =
                            (energy(&target.nbt) + rules::POWER_PER_TURN).min(rules::POWER_MAXIMUM);
                        set_energy(&mut target.nbt, value);
                    }
                });
            }
            write_crank(be, crank);
        }
        CHARGER => {
            be.data.resize(1, 0);
            be.data[0] += 1;
            if be.data[0] < rules::CHARGER_INTERVAL as i32 {
                return;
            }
            be.data[0] = 0;
            let mut power = energy(&be.nbt);
            let result = be
                .items
                .first()
                .and_then(Option::as_ref)
                .and_then(|s| recipe(game, s));
            // Consume world RNG only when upstream would roll.
            let random = if power >= rules::POWER_THRESHOLD && result.is_some() {
                game.world.random.next_float()
            } else {
                0.0
            };
            let (working, done) = rules::recipe_work(&mut power, result.is_some(), random, 1.0);
            // ChargerBlockEntity.doWork refills AFTER processing, from the ME
            // grid, at most 800 AE per call. The local crank buffer is private.
            if power < rules::POWER_THRESHOLD {
                power += super::ae2_grid_glue::extract(
                    game.dim.id(),
                    pos,
                    (rules::POWER_MAXIMUM - power).min(800.0),
                    1.0,
                    true,
                );
            }
            be.nbt.insert(
                JavaString::from("HorizonWorking"),
                Tag::Byte(i8::from(working)),
            );
            set_energy(&mut be.nbt, power);
            if done {
                be.items[0] = result;
            }
        }
        _ => {}
    }
}

pub(super) fn right_click(game: &mut Game<'_>, pos: Pos, canceled: &mut bool) {
    if *canceled {
        return;
    }
    match game.native_block(pos).as_str() {
        CHARGER => {
            *canceled = true;
            if let Some(hand) = game.held().cloned() {
                if recipe(game, &hand).is_none() {
                    return;
                }
                let inserted = game
                    .native_with_mod_block_entity(pos, |be| {
                        if be.id != CHARGER {
                            return false;
                        }
                        be.items.resize(1, None);
                        if be.items[0].is_some() {
                            return false;
                        }
                        let mut one = hand.clone();
                        one.count = 1;
                        be.items[0] = Some(one);
                        true
                    })
                    .unwrap_or(false);
                if inserted && let Some(hand) = game.held_mut().as_mut() {
                    hand.count -= 1;
                    if hand.count == 0 {
                        *game.held_mut() = None;
                    }
                }
            } else {
                let taken = game
                    .native_with_mod_block_entity(pos, |be| {
                        if be.id != CHARGER {
                            return None;
                        }
                        be.items.first_mut().and_then(Option::take)
                    })
                    .flatten();
                if let Some(stack) = taken {
                    let dir = facing(game, pos);
                    let at = std::array::from_fn(|i| f64::from(pos[i] + dir[i]) + 0.5);
                    game.spawn_item(stack, at, [0.0, 0.0, 0.0], 10);
                }
            }
        }
        CRANK => {
            *canceled = true;
            let can_turn = crank_target(game, pos).and_then(|target| {
                game.native_with_mod_block_entity(target, |be| rules::can_turn(energy(&be.nbt)))
            });
            let result = game.native_with_mod_block_entity(pos, |be| {
                let mut crank = read_crank(be);
                let result = crank.power(can_turn);
                write_crank(be, crank);
                result
            });
            if result == Some(CrankResult::Break) {
                let air = game.pack.assets.registry.default_state("minecraft:air");
                if let Some(air) = air {
                    game.native_set_block(game.dim.id(), pos, air, 3);
                }
            }
        }
        "ae2:drive" => super::ae2_grid_glue::flush_drives(game),
        _ if super::ae2_grid_glue::owns(&game.native_block(pos)) => {
            // Native energy cells have no vanilla template menu.
            *canceled = true;
        }
        _ => {}
    }
}
