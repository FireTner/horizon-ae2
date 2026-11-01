fn ae2_energy_cell_render_stack_view(
    stack: &crate::items::ItemStack,
) -> core::ops::ControlFlow<StackView> {
    ae2_energy_cell_render_bridge::view(stack)
}
