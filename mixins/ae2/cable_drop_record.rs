fn ae2_cable_drop_record(
    pending: &mut crate::game::spill::Pending,
    pos: [i32; 3],
    entity: &horizon_client::containers::BlockEntity,
    block: &str,
) {
    if pending
        .copied
        .get(&pos)
        .is_some_and(|(id, _)| id == "ae2:cable_bus")
    {
        pending.copied.remove(&pos);
    }
    if let Some(drops) = crate::game::ae2_cable_items::drops(entity, block) {
        let items = horizon_world::nbt::Tag::List(
            drops
                .iter()
                .map(horizon_client::save::item_to_nbt)
                .collect(),
        );
        // Native Pending transport only. CableBus has no block item, so this
        // entry can never be attached to a part as a component patch. clear()
        // already releases it with the other block-entity drop snapshots.
        pending.copied.insert(
            pos,
            (
                "ae2:cable_bus".into(),
                [("ae2:cable_part_drops".into(), items)].into(),
            ),
        );
    }
}
