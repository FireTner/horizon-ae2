fn ae2_ctrl_mods(mods: &mut Vec<NativeMod>) {
    let binding = [
        crate::game::ae2_ctrl::BINDING.into(),
        "Place Part on Opposite Side".into(),
        crate::game::ae2_ctrl::DEFAULT_KEY.into(),
    ];
    if let Some(entry) = mods.iter_mut().find(|m| m.id == "ae2") {
        if !entry
            .keys
            .iter()
            .any(|key| key[0] == crate::game::ae2_ctrl::BINDING)
        {
            entry.keys.push(binding);
        }
    } else {
        mods.push(NativeMod {
            id: "ae2".into(),
            title: "Applied Energistics 2 0.1.0".into(),
            description: "Applied Energistics 2 for Horizon (port in progress)".into(),
            keys: vec![binding],
        });
    }
}
