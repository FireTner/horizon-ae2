fn ae2_cable_entity_tag(entity: &BlockEntity) -> core::ops::ControlFlow<Option<Tag>> {
    if let BlockEntity::Mod(be) = entity
        && be.id == "ae2:cable_bus"
    {
        core::ops::ControlFlow::Break(Some(crate::ae2_cable_sync::encode(be)))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
