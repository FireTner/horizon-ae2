# Implemented behavior and remaining parity

All claims refer to AE2 19.2.18 and the pinned Horizon revision. Source coverage,
compilation, carried resources, and rule-level tests are different evidence.
The full mod is not yet complete.

## Subsystem boundaries

| Subsystem | Implemented layer | Remaining requirements |
| --- | --- | --- |
| Basic cells | All five tiers, portable/tool layouts, bytes/types/rounding, partitions, distribution/void, simulation/extraction, native codec and Drive mounts | Full malformed/missing-content recovery, addon handlers, workbench and upgrade lifecycle |
| Keys/counters/fuzzy | Component-aware item/fluid identities, signed counters, source fuzzy damage intervals, zero/reset behavior | Registry prototype normalization, arbitrary addon key types, identity-hash ordering and all codec recovery |
| Creative cells | Configured exact keys, unlimited transactions, source listing/status/drain, nesting and native mounts | Config writer/UI, images/descriptions, full handler/addon APIs and source iteration order |
| Portable cells | Source capacities/limits/drain, nested-cell restrictions and component-preserving backend | Handheld terminal/menu, energy use, disassembly, colors/LEDs and full upgrades |
| Tool cells | Matter Cannon/Color Applicator layouts, recipe/tag admission, restricted backend and energy retention | Shooting/painting/cycling, live reload, unknown conditions, complete item APIs and ordering ties |
| Grid/pathing | Nodes/edges, routing/finalized channels, controller validation, compressed/multiblock paths, ad-hoc and dense limits | Full source service identity, priorities/ties, events/listeners, ownership/readiness and every host |
| Energy | Finite/creative stores, priorities, local buffer, simulation/commit, multipliers, overlays, thresholds, merge/split | Complete watcher/addon callbacks, config reload, event ordering and external power capabilities |
| Network storage | Preferred passes, mount priorities, mutation queues, recursion protection and weak links | Full handler filters, all providers/consumers, security and lifecycle |
| Storage service | Node/global provider identities, refresh/unmount, public/private counts, caches and watchers | Java service oracle, live world watcher consumers, iterator/callback/hash-order fidelity and debug UI |
| Drive | Ten actual slots, all 23 built-in backend families, power/channel/priority mounting, source inventory, filtered screen and automation | Priority UI, bulk actions, cell models/LEDs, external capabilities, all destruction/chunk/reload paths |
| Controller | 8,000 public AE, 3 AE/tick idle, source flags and structures, channel supply, 18 state/type models and world conflict/restart checks | Status menu, power funnel, immediate source event/readiness/shape ordering and spatial lifecycle |
| Energy cells | Network storage, original energy/node NBT, finite fullness states/models and local comparator, deferred notifications | Exact TickManager/base settings, material/lifecycle completeness and every saved-BE/explosion/pick/loot path |
| Finite cell items | Fixed capacities/rates, source removal threshold, Charger support, float item-model thresholds and placement/first-drop energy | Complete tooltip/settings/capability APIs, charged creative variants and every loot/recovery path |
| Charger | Source recipe work, local/grid/refill power order, powered items, original inventory, automation and comparator | Live config, capability API, exact geometry/orientation/effects/drop RNG and complete lifecycle |
| Wooden Crank | Rotation/hits/energy, 18-tick turns, busy window, failure/break rules and Charger-side restriction | Placement/survival/fake-player/statistics, orientation, animation and full synchronization |
| CableBus | Five types/17 colors, flags, raw shapes, channel payload rules, storage, center NBT and source quad construction | Native center rendering/outline/contextual movement/piston shapes and same-bundle visual transport; full placement, parts/facades, event/selection/drop lifecycle |
| Platform/content | Pinned original resources, source stack limits, 461 generated NeoForge common tags and recipe preparation | Full platform services, recipe types/conditions/IDs, registries, progression, addons and coexistence checks |

## Components, keys, and inventories

Keys retain component identity rather than only registry IDs. Native unknown typed
components are preserved, including genuine `custom_data`, nested stacks, and
source saved keys. Full effective-component normalization against registry
prototypes and all Minecraft 1.21.1 codec recovery remain open.

Malformed/unknown/duplicate/nonpositive or over-capacity cell entries are refused
without mutation. This is stricter than source missing-content recovery. Native
tooltip key lookup currently covers known item IDs and vanilla water/lava;
custom fluids and the external fluid capability API remain open.

Upgrades use the original slotted item-container form, for example
`[{slot:0,item:{id:"ae2:void_card",count:1}}]`, not a flat card list. Slot/card
limits follow pinned registrations; stack count does not multiply installed cards.
Energy cards use their physical count as in source. Writer/callback/workbench
behavior and capacity expansion on real inventory edits remain incomplete.

Drive and Charger share optional-stack inventory decoding: empty compound,
omitted default count one, original `inv.itemN`, and a native `Items` transport
mirror. Positive loaded counts above normal slot limits are preserved. Normal
menu/automation insertion still has source limit one. Malformed source inventory
stays blocked until explicit repair; a changed native echo cannot erase it.
Unchanged echoes cannot restore spent power or removed contents.

Drive has ten real slots and a native `generic_9x2` transport screen with eight
hidden, prohibited padding slots. Its policy is keyed by authoritative actor,
window, and actual block. Padding is not extra storage, and this is not AE2's
wire protocol. Priority UI, ghost icons, special bulk actions and external
multiplayer are still incomplete.

## Power, lifecycle, and controller state

The native world adapter polls loaded block entities before/after level ticks.
It preserves same-tick energy use and detects NBT changes, but polling is not the
entire source event/readiness/chunk model. Equal saved data alone does not prove
object identity after every replacement path. Finite cell item placement explicitly
forgets the previous node even at an identical position/kind/saved amount.

Local node buffer capacity defaults to 25 AE per node. Source energy is restored
from node `e` only when the tag is Double. Native power validation refuses
negative/nonfinite values along guarded paths; this does not establish the same
recovery as every Java NaN/infinity/negative-number branch.

EnergyCellBlockItem has fixed 200,000/1,600,000 AE capacities and 800/1,600 rates;
foreign capacity/upgrades are preserved but do not override that profile. Its
removal threshold is 0.00001, unlike powered items' 0.0001. It does not have the
base powered-item green bar. Its original model uses float fifths; placed finite
cells use the separate double quarter/fullness formula and local comparator.

The render-only cell model projection never writes `block_state` into real stacks,
packets, saves or click hashes. A private render bridge carries NaN/infinity
through an otherwise lossy JSON view. Java model predicate bytecode and native
resolver tests cover those cases; arbitrary malformed native recovery is open.

Placement imports finite cell `stored_energy`. Implemented destruction paths
copy positive local energy into only the first matching drop, without `proxy.e`.
Live guards do not overwrite newer external NBT or restore same-tick expenditure.
All saved-BE/command/explosion/pick/loot/base-settings paths still require work.

Controller status/type updates currently run after the server tick. Original
immediate shape, readiness, listener and spatial ordering are not fully reproduced.
Finite-cell comparator/fullness notifications wait for ticking chunks and execute
external callbacks after releasing the network lock.

Current native configuration uses source defaults, power multiplier/charge factor
one, and default channel mode. Rule APIs accept more configurations; live suppliers,
reload and all upgrade callbacks are not fully connected.

## Cable profiles, persistence, and geometry

Cable profiles preserve five types and 17 colors, enum ordering, independent
variant/size min/max, palette/tints, connection lengths, raw boxes and channel
scaling. Dense flags replace `PREFERRED`; they do not add to it. Normal/dense
capacity is eight/32 before mode scaling. Transparent/equal colors permit
in-world edges; full blocks use transparent grid color.

Multipart storage keeps center, six side references, and six facade references.
`setPart(null)` allocates its array; `setFacade(null)` removes without allocation.
This storage rule is not an implemented facade item or complete part registry.

The saved-host adapter reads `cable.id`, `cable.gn`, six side names and
`facadeDown/Up/North/South/West/East`. It never guesses a fluix glass cable from
the host block ID. It retains raw unknown host/part/facade/service fields.
Unsupported side parts keep the host inactive because they need their own nodes
and services. Facade BlockState codecs and a full source host writer remain open.

Raw NBT item-ID changes, including externally changed colors, replace the part;
direct source painting instead recolors its existing node. Non-owner node data
changes trigger replacement; absent/empty/owner-only data compare equal. Own
saves update the observed snapshot, and persistence refuses a newer unmatched
center rather than overwriting it. `gn.p` is an Int on save; positive buffer is
written to `gn.e`, and zero removes that field.

Java `GridNode.savedData` represents its previous load/grid transition. The polling
adapter compares against observed/own-save data. Explicit BE-load events after
same-value edits, complete readiness/destruction and service callbacks still need
a source-faithful event layer. Unsupported IDs, side parts, malformed YesNo tags
and negative/nonfinite energy use a stricter refusal policy than source recovery.

Cable quad construction preserves source float32 operation order, UV rotations/
flips/custom rectangles, texture IDs, colors/fullbright, core overrides, straight
shortcuts, internal connections and machine end caps. Dense groups use wrapping
`(channels+3)/4`; straight dense segments extend to -0.01/16.01 pixels. The smart
machine-cap path retains its bright vertex color for the following small segment.

`Quad.cull` is emitter bookkeeping; source cable BakedQuads are returned unculled.
Applying that field as ordinary world-face culling would be incorrect. Texture
suffix `transparent` differs from item prefix `fluix`. The world collector must
resolve connection types with the source minimum and use actual graph power and
visual channels. All side/facade rendering and source packet behavior remain
completion requirements.

The native center stage registers all 32 light/waterlogged states. It renders
original cable quads through the entity quad batch and appends the original part
atlas source through an AE2-owned atlas extension. Selection/raycast and the
SceneCollisions contextual movement/piston shape paths consume center geometry.
Other direct collision callers still use the static glass-core fallback; full
support/occlusion/fluid/light behavior and all collision callers remain open.

An owned effective resource registry appends the real `ae2:cable_bus` block
entity type without renumbering vanilla types. Chunk and block updates carry
center NBT and a versioned, transient visual payload derived from graph channels.
The client unwraps it only at packet receipt; save decoding preserves arbitrary
source NBT. This same-bundle transport is not AE2's complete source packet or
external-client negotiation. Full lighting/AO, attachment/facade geometry,
survival placement, partial breaking and drops remain unfinished.

## Native networking, recipes, and platform limits

AE2 stack components travel through a versioned wire-only native envelope that
preserves genuine custom data and unwraps before normal decoding. It retains
nested stack data and stable click hashes. It requires the same native bundle;
external multiplayer negotiation and arbitrary addon component APIs remain open.

Original recipe subdirectories are flattened because the native loader currently
requires it. The source-to-native ID map is recorded and collisions are rejected.
Advancements, guidebook/UI references and other source-ID consumers need a proper
engine adapter. Unsupported recipe types remain carried data and do not execute.

Charger execution refuses results with unsupported components; ingredient-based
automation admission does not silently invent a prohibition. Random samples are
taken from native world RNG only under source conditions, but whole-world RNG
call order has not been compared tick-for-tick with Java. Current dropped items'
position/velocity do not yet match the source random spawn rules.

Tool admission expands known recipe/tag conditions. Unknown conditions, including
inside negation, are refused. Native recipe-table order is used for ties; source
RecipeManager ordering and live reload remain unverified.

The 461 NeoForge common tags are pinned and layered before AE2 tags with original
replace/provenance rules. World checks cover selected item predicates/restart,
not every registry/tag. No NeoForge JVM runtime or general platform API is bundled.

The port and Create use separate IDs/helpers. Joint mixin planning has passed;
a joint running-game/capability validation has not. Launcher installation/rebuild,
external LAN/dedicated clients, all survival progression and large-network
performance remain separate unfinished requirements.

## Verification limits

Differential reports preserve source hashes, exact methods, inputs and scope.
Java stub-host oracles prove the covered rules with the listed substitutes;
they do not prove the complete mod, registry codecs, game events or GPU behavior.
Native tests and actual isolated game checks are documented separately in
`../VALIDATION.md`. A successful package check validates the archive contract,
not the gameplay of every carried item.
