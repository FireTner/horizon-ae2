fn ae2_block_states_build(
    base: &std::path::Path,
    mods: &[crate::mod_content::ContentMod],
    out: &std::path::Path,
) -> core::ops::ControlFlow<Result<Vec<String>>> {
    ae2_block_states::build(base, mods, out)
}
