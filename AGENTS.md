# Applied Energistics 2 for Horizon

User requirement: complete 1:1 parity with the pinned AE2, not an AE2-like replacement.
Work without subagents. Read README.md, docs/PLAN.md, docs/PARITY.md,
upstream.lock.json and compatibility.json before editing.

Keep this repository separate from horizon-create, Horizon and horizon-mixins.
Never edit their original sources, player resources, saves or launcher settings.
Export only committed pinned engine sources into ignored target trees. Reference
AE2 Java is the specification; do not approximate missing behaviour silently.

Rules live in mixins/ae2/*.rs and are exposed by src/lib.rs for standalone tests.
glue.rs owns Horizon types. Only modules actually consumed by the adapter belong
in horizon-mixin.toml. keys.rs, storage.rs and cells.rs feed the native cell
tooltip/codec adapter. energy.rs/grid.rs/power.rs feed network.rs and the
grid_glue.rs world adapter (currently Charger + three energy-cell kinds + Drive).
mounts.rs/service.rs/drive.rs are consumed by drive_glue.rs and belong in the
native manifest. Drive currently has ten real cell slots/providers, original
inv persistence and power/channel gating; GUI and filtered automation remain open.
Injected payloads cannot use inner attributes/comments, external mod files or
include!. Helpers are tiny, uniquely named ae2_* functions.

Keep AEKey identity component-aware. Do not substitute registry strings for full
item/fluid keys. Preserve simulation/commit semantics, long counts, partial
insertion, byte rounding, type limits, partition/extraction rules and nested-cell
restrictions. Never fake ME channels, energy, crafting jobs or security permissions.

Use upstream LGPL-3.0-or-later for derived behaviour; preserve notices and licences.
Assets come from the pinned jar into ignored target/content only. Do not commit
jars, Minecraft resources, models, textures, exported sources or binaries.
Tooling adapted from horizon-create keeps tools/LICENSE-MIT.

Verify cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings,
cargo test --locked, Python 3.12 tools/test_tooling.py and tools/upstream.py audit
--check. Horizon changes also require tools/horizon.py check against the pinned
engine. Build and isolated game verification are separate evidence, not implied
by unit tests. Log meaningful engine gaps and known deviations in docs/PARITY.md.
Do not report a partial class or a carried resource as complete feature parity.

All generated/intermediate files stay under target/. Update VALIDATION.md with
what actually ran, including limits. No automatic push, publication or deployment.

During the active full-port goal, check account usage with the Codex app tool
about every five minutes. The user explicitly requested pausing the goal when
any relevant available usage window has 10% or less remaining. Never reset or
purchase quota. Unknown/missing windows do not mean zero remaining.
