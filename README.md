# Applied Energistics 2 for Horizon

A native Rust port of [Applied Energistics 2](https://github.com/AppliedEnergistics/Applied-Energistics-2)
for Minecraft Horizon. The goal is full 1:1 behavior, content, interfaces,
rendering, persistence, and networking with the pinned AE2 release.
The full port is in progress.

The GitHub repository is named `ae2-create`; the mod ID is `ae2`. This project
ports AE2 to Horizon and does not require Create.

## Pinned versions

| Component | Version / revision |
| --- | --- |
| AE2 | 19.2.18, NeoForge, Minecraft 1.21.1 |
| AE2 source | `db17504a86128fdf3dae31f5fb7a112a646e0b93`, `neoforge/v19.2.18` |
| Horizon | 26.3, `aa27eca326db26e999d4e59d7899576016764b1e` |
| horizon-mixins | `3f99a5c60dfa311cc365524c44ae2fd799bfa1de`, schema 6 |
| NeoForge common tags | 21.1.169, `863802b6dfdb1d5e48c2fda093ed9d01df0f4cd9` |

See [upstream.lock.json](upstream.lock.json) and
[compatibility.json](compatibility.json). Java is the behavioral specification;
ported gameplay runs as native Rust and does not require a JVM.

## Implemented layers

- Component-aware item/fluid keys and storage backends for all 23 built-in
  basic, portable, creative, Matter Cannon, and Color Applicator cell profiles.
- ME topology, controller validation, channel routing, energy services,
  storage mounting, priorities, simulation/commit, and merge/split rules.
- Native Charger, energy cells, Drive, and Controller. These include original
  inventories and energy persistence, power/channel gating, a ten-slot Drive
  screen, automation, and comparator behavior for the supported hosts.
- Charging for 16 powered-item profiles and both finite energy-cell items;
  original item energy bars, cell item models, and finite-cell placement/drop
  energy transfer along the implemented lifecycle paths.
- Cable types, colors, channel visuals, multipart storage, center NBT parsing,
  reload identity, source-derived cable quad builders, and native center-only
  CableBus states, rendering, selection, contextual collision and visual transport.
- Original assets and recipes prepared from the pinned AE2 jar, plus the pinned
  NeoForge common tags. Carried data is not evidence of implemented gameplay.

The source inventory tracks all 1,429 Java files. A partially ported class or a
visible item does not establish complete feature parity. Read
[PARITY.md](docs/PARITY.md) for concrete boundaries and
[PROGRESS.md](docs/PROGRESS.md) for the current checkpoint.

## Layout

| Path | Purpose |
| --- | --- |
| `mixins/ae2/` | Shared Rust rules, native adapters, and source injection manifest |
| `src/lib.rs` | Engine-independent rules used by standalone tests |
| `tools/` | Pinned source/content preparation, staging, packaging, oracles, game checks |
| `examples/` | Rust drivers used by differential Java comparisons |
| `docs/` | Scope, parity boundaries, source inventory, and verification reports |
| `target/` | Ignored sources, jars, content, staged engine, builds, and isolated worlds |

Original Horizon, horizon-mixins, Create, installed game resources, player saves,
and launcher settings are not edited by the port tooling.

## Development setup

Use the Rust toolchain in [rust-toolchain.toml](rust-toolchain.toml), Python 3.12,
Git, and a JDK 21 for the Java differential oracles. Native Horizon builds also
need its documented platform graphics/build dependencies and separately prepared
26.3 game resources.

```sh
python3.12 tools/upstream.py fetch
python3.12 tools/upstream.py fetch-jar
python3.12 tools/upstream.py fetch-engines
python3.12 tools/upstream.py fetch-platform
python3.12 tools/check.py
```

Set these variables to your own paths when running native development tools:

```sh
export HORIZON_CHECKOUT=/path/to/pinned/horizon
export HORIZON_MIXINS=/path/to/pinned/horizon-mixins
export HORIZON_RESOURCES=/path/to/prepared/26.3
export HORIZON_PORT_PACK=/path/to/horizon-port-pack
export HORIZON_MOD_TOOL=/path/to/cargo-horizon-mod
export HORIZON_CTL=/path/to/horizon-ctl
export JAVA_HOME=/path/to/jdk21
```

`target/local-env.sh`, when present, is an ignored machine-local convenience file;
it is not a portable setup script. Never point build output or smoke worlds at
an installed game or a player save.

Prepare content from the verified jar into a new owned target tree:

```sh
python3.12 tools/prepare_content.py \
  --jar target/jars/appliedenergistics2-19.2.18.jar \
  --resources "$HORIZON_RESOURCES" \
  --port-pack "$HORIZON_PORT_PACK"
python3.12 tools/horizon.py check
python3.12 tools/horizon.py test
python3.12 tools/horizon.py build
```

Build the data archive with `$HORIZON_MOD_TOOL build` from `target/content/ae2`.
Consult each tool's `--help` for its exact interface. Content preparation starts
from the original jar; repeatedly fitting an already merged content tree is not
supported.

## Verification

```sh
python3.12 tools/check.py
python3.12 tools/cable_render_oracle.py --java-home "$JAVA_HOME"
python3.12 tools/node_data_oracle.py --java-home "$JAVA_HOME"
python3.12 tools/smoke_power.py \
  --ctl "$HORIZON_CTL" --resources "$HORIZON_RESOURCES"
```

[VALIDATION.md](VALIDATION.md) distinguishes standalone rules, Java stub-host
oracles, native compilation/tests, and actual isolated game scenarios. Java
oracles do not execute the entire Minecraft/NeoForge mod. Native compilation
does not establish correct gameplay or rendering.

## Packaging

```sh
python3.12 tools/package.py
"$HORIZON_MOD_TOOL" check target/ae2-0.1.0.hnmod --json
```

The deterministic `.hnmod` contains the native source bundle, pins, licenses,
and corresponding port source. The `.hmod` contains separately prepared data
and assets. Both have mod ID `ae2` and must match the pinned engine/tooling.
Launcher installation/rebuild and external multiplayer are separate checks;
local built-client smoke tests do not imply they have passed.

## Remaining scope

The complete port still requires native cable/part/facade lifecycle and client
integration, terminals, ME Chest and other hosts, crafting CPUs/jobs, patterns,
networks and tunnels, external capabilities, security/ownership, wireless,
spatial I/O, tools and menus, missing machine recipes, survival progression,
world generation, visual/audio effects, addon APIs, and complete save/load,
chunk/dimension, multiplayer, and platform behavior. The detailed plan is in
[PLAN.md](docs/PLAN.md).

## Licensing

Derived AE2 behavior is LGPL-3.0-or-later; its API carries the original MIT
notice. Fabric-derived square-coordinate rules retain their Apache-2.0 notice.
Adapted tooling keeps its MIT notice. See [NOTICE.md](NOTICE.md), [LICENSE](LICENSE),
and [LICENSES](LICENSES).

AE2 assets are extracted locally from the original jar. They are not included
in this source repository. This project is not affiliated with Team Applied
Energistics, Horizon, Mojang, or Microsoft.
