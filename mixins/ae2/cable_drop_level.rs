fn ae2_cable_drop_level(
    level: &mut crate::game::GameLevel<'_>,
    pos: horizon_core::position::BlockPos,
    state: horizon_world::registry::BlockStateId,
) -> core::ops::ControlFlow<()> {
    if level.facts.get(state).name.as_ref() != "ae2:cable_bus" {
        return core::ops::ControlFlow::Continue(());
    }
    let at = [pos.x, pos.y, pos.z];
    let drops = level
        .world
        .block_entities
        .get(&(pos.x, pos.y, pos.z))
        .and_then(|be| crate::game::ae2_cable_items::drops(be, "ae2:cable_bus"))
        .unwrap_or_default();
    level.pending.clear(at);
    if level.rules.block_drops {
        for stack in drops {
            level.pop_resource(at, stack);
        }
    }
    core::ops::ControlFlow::Break(())
}
