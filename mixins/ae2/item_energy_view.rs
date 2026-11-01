// SPDX-License-Identifier: LGPL-3.0-or-later
// AEBasePoweredItem bar overrides; render through native item-decoration layout.
use super::{Game, HudBatch, hud_layout, hud_overlays};
use horizon_client::items::ItemStack;

pub(super) fn bar(stack: &ItemStack) -> Option<(i32, u32)> {
    if super::ae2_item_energy::PoweredKind::from_item(&stack.item)?.is_energy_cell() {
        return None; // EnergyCellBlockItem does not inherit AEBasePoweredItem's bar.
    }
    let item = super::ae2_item_energy_glue::load(stack).ok().flatten()?;
    Some((item.energy.bar_width(), 0x00ff00))
}
pub(super) fn decorations(
    game: &Game<'_>,
    batch: &mut HudBatch,
    stack: &ItemStack,
    x: f32,
    y: f32,
    count_text: &Option<String>,
) -> bool {
    let Some(bar) = bar(stack) else {
        return false;
    };
    let info = game.items().get(&stack.item);
    let cooldown =
        horizon_client::cooldowns::group(stack, info.and_then(|item| item.use_cooldown.as_ref()));
    let decorations = hud_overlays::ItemDecorations {
        count: stack.count,
        count_text: count_text.clone(),
        bar: Some(bar),
        cooldown: game
            .cooldowns
            .percent(&cooldown, game.clock.fraction() as f32),
    };
    // Same count/cooldown/pose painter as Game::draw_item_decorations; only the
    // source item-defined bar differs. Never encode energy as damage/components.
    let mut draws = vec![hud_layout::GuiDraw::Push(vec![
        hud_layout::PoseOp::Translate(x, y),
    ])];
    draws.extend(hud_overlays::item_decorations(
        0,
        0,
        &decorations,
        &game.gui.atlas.font,
    ));
    draws.push(hud_layout::GuiDraw::Pop);
    hud_layout::paint(batch, &game.gui.atlas, &draws);
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    use horizon_world::nbt::Tag;
    #[test]
    fn powered_bar_is_always_visible_green_and_never_changes_stack_identity() {
        let mut stack = ItemStack::new("ae2:portable_item_cell_1k", 1);
        assert_eq!(bar(&stack), Some((0, 0x00ff00)));
        stack.data_mut().set_other(
            "ae2:energy_capacity",
            Some(Tag::Double(20000.0f64.to_bits())),
        );
        stack
            .data_mut()
            .set_other("ae2:stored_energy", Some(Tag::Double(10000.0f64.to_bits())));
        let before = stack.clone();
        assert_eq!(bar(&stack), Some((7, 0x00ff00)));
        assert_eq!(stack, before);
        stack
            .data_mut()
            .set_other("ae2:stored_energy", Some(Tag::Double(20000.0f64.to_bits())));
        assert_eq!(bar(&stack), Some((13, 0x00ff00)));
        assert_eq!(bar(&ItemStack::new("minecraft:diamond_pickaxe", 1)), None);
        assert_eq!(bar(&ItemStack::new("ae2:item_storage_cell_1k", 1)), None);
        assert_eq!(bar(&ItemStack::new("ae2:energy_cell", 1)), None);
        assert_eq!(bar(&ItemStack::new("ae2:dense_energy_cell", 1)), None);
    }
}
