fn ae2_cable_atlas(root: &Path) -> core::ops::ControlFlow<BTreeMap<String, Vec<SpriteSource>>> {
    thread_local! {static ACTIVE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
    if ACTIVE.with(|v| v.replace(true)) {
        return core::ops::ControlFlow::Continue(());
    }
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            ACTIVE.with(|v| v.set(false));
        }
    }
    let _guard = Guard;
    let mut value = atlases(root);
    if let Some(sources) = value.remove("ae2:cable") {
        value
            .entry("minecraft:blocks".into())
            .or_default()
            .extend(sources);
    }
    core::ops::ControlFlow::Break(value)
}
