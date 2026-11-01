fn ae2_energy_cell_drop_record(
    pending: &mut crate::game::spill::Pending,
    pos: [i32; 3],
    entity: &horizon_client::containers::BlockEntity,
    block: &str,
) {
    if let Some(patch) = crate::game::ae2_energy_cell_item::export(entity, block) {
        pending.copied.insert(pos, (block.to_owned(), patch));
    }
}
