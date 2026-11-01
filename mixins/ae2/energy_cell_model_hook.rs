fn ae2_energy_cell_model_append(
    models: &crate::item_model::ItemModels,
    out: &mut crate::item_model::ResolvedItem,
    stack: &crate::item_model::StackView,
    ctx: &crate::item_model::Context,
    env: &dyn crate::item_model::ItemEnv,
) -> core::ops::ControlFlow<()> {
    ae2_energy_cell_model::append(models, out, stack, ctx, env)
}
