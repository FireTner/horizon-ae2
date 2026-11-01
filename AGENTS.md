# Applied Energistics 2 for Horizon

User requirement: complete 1:1 parity with the pinned AE2, not an AE2-like replacement.
Work without subagents. Read README.md, docs/PLAN.md, docs/PARITY.md,
upstream.lock.json and compatibility.json before editing.

Keep this repository separate from horizon-create, Horizon and horizon-mixins.
Never edit their original sources, player resources, saves or launcher settings.
Export only committed pinned engine sources into ignored target trees. Reference
AE2 Java is the specification; do not approximate missing behaviour silently.
NeoForge's generated common c tags come from the pinned read-only checkout in
target/upstream/neoforge. Preserve its licence/provenance and resource layering;
never replace missing platform tags with ad-hoc item lists.

Rules live in mixins/ae2/*.rs and are exposed by src/lib.rs for standalone tests.
glue.rs owns Horizon types. Only modules actually consumed by the adapter belong
in horizon-mixin.toml. keys.rs, storage.rs and cells.rs feed the native cell
tooltip/codec adapter. energy.rs/grid.rs/power.rs feed network.rs and the
grid_glue.rs world adapter (Charger + three energy-cell kinds + Drive + Controller).
controller.rs supplies source 8000 public AE/idle 3, node profile and visual rules.
controller_states.rs wraps only owned Controller content builds and expands the
single physical cube row to its actual 18 state/type variants across effective
JSON/NBT tables. Preserve vanilla ids and foreign shape/model references. This
is not a general custom-state API. Renderer model schema requires named values.
Controller status/type currently update at server post_tick; immediate shape/
ready/events, external power funnel, Network Status Menu and full lifecycle
remain open. Do not mark Controller fully ported on these tests alone.
item_energy.rs/item_energy_glue.rs feed all 16 built-in powered items into Charger;
charger.rs preserves local/direct-grid/overflow/refill order. Command registry hooks
add only supported component IDs, preserve vanilla types and validate energy numbers.
item_energy_view.rs implements the source green 13-pixel bar, preserving count/cooldown.
component_sync.rs lives in the native client library, sends AE2 typed component
maps through a versioned wire-only envelope and unwraps before normal decoding.
It must preserve real custom_data, nested items, original saved keys and click hashes.
inventory_codec.rs is shared by Drive/Charger for original optional ItemStacks,
count validation/default omission and source-versus-native echo selection.
Loaded positive stack counts are not the same as normal input slot limits.
inventory_signal.rs/glue feed the native Containers.signal hook only for
Drive/Charger: source 10/1 slots and slot limit one, never menu padding or a
silent fullness clamp. Air/zero are empty. Preserve the existing malformed gate;
do not present it as source recovery. Other hosts keep native signal rules.
charger_inventory.rs reconciles original inv.item0 with native Items by value;
unchanged echoes cannot restore stale contents/power. Malformed original inv stays
blocked until an explicit repair; never overwrite it from a changed native echo.
Charger slot edits flush the original format immediately; working stays transient.
Handheld actions, config reload, item energy tooltip and upgrade callbacks remain open.
mounts.rs/service.rs/drive.rs are consumed by drive_glue.rs and belong in the
native manifest. Drive currently has ten real cell slots/providers, supports
all twenty-three built-in basic/portable/creative/tool cell types, original
inv persistence and power/channel gating; its filtered ten-slot screen works,
while priority UI and special bulk actions remain open. automation*.rs enforces
the same ten-slot limits/filters for native hoppers/droppers and flushes cells
before slot mutation; a general external capability API is still pending.
slot_rules.rs is shared with native library menu policy; guest_slots.rs scopes
server rules by authoritative player/window/block identity, never by chest size.
backend.rs dispatches finite/creative inventory rules; cells.rs owns component
codecs, including 63-slot nullable config limits. upgrade_rules.rs supplies finite
host slot/card limits and per-slot counts; cells.rs reads canonical slot/item
upgrade entries. Workbench/writer/callbacks and addon upgrades remain open.
tool_filters.rs/tool_glue.rs
expand recipe/tag admission into a restricted finite backend; never load a tool
as an unrestricted basic cell. Live data reload and handheld tool actions remain open.
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
tools/item_stack_limits.py owns the 58 built-in unstackable registrations and
audits their pinned factories/builder/inheritance. prepare_content.py sets their
limit to one and the other carried item definitions to 64, preserving other fields.
These are registration defaults, not component-specific stack overrides or full item APIs.

Verify cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings,
cargo test --locked, Python 3.12 tools/test_tooling.py and tools/upstream.py audit
--check. Horizon changes also require tools/horizon.py check against the pinned
engine. tools/item_stack_limits.py --check must match its committed source audit.
Build and isolated game verification are separate evidence, not implied
by unit tests. Log meaningful engine gaps and known deviations in docs/PARITY.md.
Do not report a partial class or a carried resource as complete feature parity.

All generated/intermediate files stay under target/. Update VALIDATION.md with
what actually ran, including limits. No automatic push, publication or deployment.

During the active full-port goal, check account usage with the Codex app tool
about every five minutes. The user reset quota and reinstated an automatic stop:
pause the goal when any relevant available usage window has 15% or less remaining.
This replaces the previous removal of the 10% stop. Never reset or purchase quota.
Unknown/missing windows do not mean zero remaining.
