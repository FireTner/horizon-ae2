fn ae2_cable_entity_type(name: &str) -> core::ops::ControlFlow<Option<&'static str>> {
    if name == "ae2:cable_bus" {
        core::ops::ControlFlow::Break(Some("ae2:cable_bus"))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
