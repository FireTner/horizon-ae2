# Applied Energistics 2 для Horizon

Полное портирование [Applied Energistics 2](https://github.com/AppliedEnergistics/Applied-Energistics-2)
на Minecraft Horizon. Цель — поведение, предметы, интерфейсы, графика и сохранения
**1:1 с выбранной версией AE2**. Это начало переноса; полноценной ME-сети пока нет.

Эталон: **AE2 19.2.18, NeoForge, Minecraft 1.21.1**, тег `neoforge/v19.2.18`,
коммит `db17504a86128fdf3dae31f5fb7a112a646e0b93`. Выбрана та же ветка Minecraft,
что и в соседнем horizon-create. Horizon: **26.3**, коммит `aa27eca`; mixins:
`3f99a5c`. Переход на другую версию оригинала — отдельное явное решение.

Используется тот же подход, что в horizon-create: Java служит спецификацией,
правила переписываются на Rust, native mixins подключают их к закреплённой
копии движка, ресурсы подготавливаются из оригинального jar. JVM не нужна.

Что уже написано:

- Ядро обычных ячеек хранения всех пяти размеров: байты, типы, округление,
  симуляция, извлечение, точные/fuzzy фильтры, inverter/equal distribution/void.
  Добавлены компонентные item/fluid keys, GenericStack, signed KeyCounter и
  native codec исходных компонентов ячеек. Tooltip использует реальные значения,
  локализацию и цвета оригинала. **Drive backend и базовый экран подключены;
  priority UI, специальные действия меню и терминал ещё не перенесены.**
- `StoredEnergyAmount`, EnergyService и per-node buffer: приоритеты хранилищ,
  симуляция, события, power delay, overlay/passive rules. В мире подключены
  обычная/плотная/творческая energy cell и Charger. CableBus/QuartzFiber parts
  и остальные устройства пока не подключены. Правила сервиса и overlay сверены
  с Java: 2000 сценариев, 462175 операций, без расхождений.
- Первый игровой срез Charger + Wooden Crank: один предмет, рецепт из
  таблицы игры, энергия в NBT, оборот 18 тиков/160 AE, проверка раз в 10 тиков,
  порог 1599 AE и завершение при randomFloat > 0.8. Charger получает до 800 AE
  за doWork из настоящего сетевого сервиса и не подключается передней гранью.
- Ядро ME-графа: топология, controller validation, ad-hoc, три очереди
  pathfinding, bottlenecks, multiblock и compressed channels. На **2000 схемах**
  значения всех узлов/соединений совпали с исполняемым оригинальным Java-кодом
  при одинаковом порядке входных узлов/связей. World/power adapter пока подключает пять типов full blocks; CableBus,
  controllers и channel consumers в мире ещё не подключены.
- Ядро `NetworkStorage`: приоритеты, два прохода вставки, очереди mount/unmount
  и защита от рекурсии; 2000 сценариев / 410882 операции совпали с Java.
  StorageService подключает десять дисков Drive, priority/online mounts,
  ленивый кэш и подписки; диски сохраняются в исходном `inv.itemN`. В игре
  проверены точный расход энергии, item/fluid cells и restart. GUI Drive показывает
  исходную текстуру и десять filtered slots: pickup/shift, drag/hotbar rules,
  сохранение и изоляция от обычных контейнеров проверены. Priority UI,
  динамический рендер и терминалы ещё не перенесены.
- Drive automation: native hoppers/droppers вставляют ячейки со всех шести
  сторон, соблюдают фильтр/лимит и сохраняют компоненты при извлечении и отказе.
  Полный Drive оставляет лишнюю ячейку в источнике; offline и restart проверены.
  Внешний capability API и совместимость с Create ещё не проверены.
- Backend десяти portable item/fluid cells: половина ёмкости, исходные
  type limits и 0.5 AE/tick на диск, codec/tooltip и mounts Drive. Обычные и
  переносные профили сверены с исходным BasicCellInventory: 2000 сценариев,
  572000 операций без расхождений. Handheld UI/применение ещё не перенесены; Charger charging подключена.
- Creative cell backend: configured keys, unlimited insert/extract, исходный
  listing cap, статус и нулевой расход диска; Drive online/offline mounts,
  codec и restart. Ещё 2000 Java сценариев/572000 операций PASS. Tooltip image,
  Workbench и полный item API пока не перенесены.
- Tool storage backends: Matter Cannon (512 bytes/1 type) и Color Applicator
  (2048 bytes/27 types), исходные recipe/tag admission rules, 0.5 idle drain,
  сохранение энергии и разрешённое nesting заполненных tools. Drive подключает
  все 23 встроенных storage handlers. Charger charging подключена; handheld shooting/painting ещё нужны.
- Исходные upgrade container components читаются как `slot/item`, с разными
  slot/card limits обычных, portable и tool cells. Количество установленных
  карт считается по слотам и ограничивается host. Workbench/upgrade writer,
  callbacks и addon registry ещё не перенесены.
- Энергия всех 16 powered items в Charger: original stored_energy/capacity,
  per-item rates/energy cards, local-first/grid/overflow и общее пополнение буфера.
  Ядро AEBasePoweredItem сверено с unchanged Java на 400000 операций, а дословный
  Charger.doWork в stub host — ещё на 200000 вызовов. Command registry знает
  шесть поддержанных AE2 component IDs; energy codecs принимают только числа.
  Workbench callbacks/config reload, energy tooltip и handheld actions ещё нужны.
- Энергия отображается исходной зелёной полосой 0..13 pixels. Полный component
  map AE2 stacks теперь пересекает native packet sync, включая nested shulker/
  bundle и контейнерные click hashes; source/saved keys сохраняются. Pixel QA,
  actual charged-cell menu roundtrip и Drive GUI regression пройдены.
- Charger сохраняет исходный `inv.item0`, синхронизирует native Items echo,
  записывает slot edits сразу и держит working transient. Source updates,
  malformed retention, native-save migration и restart проверены. Full recovery,
  hopper/capabilities и все drop paths ещё открыты.
- Общий source inventory codec Drive/Charger проверяет count до native clamp,
  сохраняет optional/default форму и защищает blocked original от изменения echo.
  Прямой positive count из NBT сохраняется отдельно от limit-one обычной вставки.
  Проверены block/unblock/restart, точный drain и регрессии обоих устройств/GUI.
- Подготовка контента, source-only `.hnmod`, закреплённые исходники и
  проверяемый реестр **всех 1429 Java-файлов** оригинала. В реестре нет
  полностью закрытых классов; 69 отмечены как частично перенесённые.
- Подготовка также переносит 461 исходный общий тег NeoForge из pinned source,
  с лицензией/хэшами и порядком platform → AE2. World item predicates и restart
  проверены. Три прежних ручных списка заменены исходными данными платформы.

Полный объём и последовательность работ: [docs/PLAN.md](docs/PLAN.md).
Текущие отличия: [docs/PARITY.md](docs/PARITY.md).
Фактически выполненные проверки: [VALIDATION.md](VALIDATION.md).

## Работа с проектом

Нужны Rust 1.99.1, Python 3.12, Git, закреплённые checkout Horizon и
horizon-mixins. Инструменты не меняют исходный checkout Horizon и игровые данные.

```sh
python3.12 tools/upstream.py fetch
python3.12 tools/upstream.py fetch-jar
python3.12 tools/upstream.py fetch-engines
python3.12 tools/upstream.py fetch-platform
python3.12 tools/upstream.py audit --check
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3.12 tools/test_tooling.py

# Rule-level oracle, нужен JDK 21+; игровые зависимости заменены заглушками.
python3.12 tools/pathing_oracle.py --java-home /path/to/jdk21 --cases 2000
python3.12 tools/energy_oracle.py --java-home /path/to/jdk21 --cases 1000
python3.12 tools/energy_oracle.py --java-home /path/to/jdk21 --overlay --cases 1000
python3.12 tools/mounts_oracle.py --java-home /path/to/jdk21 --cases 2000
python3.12 tools/cells_oracle.py --java-home /path/to/jdk21 --cases 2000
python3.12 tools/cells_oracle.py --java-home /path/to/jdk21 --creative --cases 2000
python3.12 tools/cells_oracle.py --java-home /path/to/jdk21 --tools --cases 2000
python3.12 tools/item_energy_oracle.py --cases 2000
python3.12 tools/item_energy_oracle.py --bars --cases 2000
python3.12 tools/charger_oracle.py --cases 2000

python3.12 tools/horizon.py check
python3.12 tools/horizon.py build
python3.12 tools/horizon.py test --library
```

Контент: `horizon-port-pack` из той же версии движка и подготовленные ресурсы
Horizon 26.3. Jar загружается с Modrinth и проверяется по SHA-512 из lock-файла.

```sh
python3.12 tools/prepare_content.py \
  --jar target/jars/appliedenergistics2-19.2.18.jar \
  --resources /path/to/prepared/26.3 \
  --port-pack /path/to/horizon-port-pack

# Сначала использовать отдельный тестовый game-dir и HORIZON_HOME.
HORIZON_HOME="$PWD/target/run/home" HORIZON_NO_AUDIO=1 \
  target/horizon-cargo/fast/horizon-client \
  --world AE2Dev --creative \
  --game-dir "$PWD/target/run/game" \
  --resources /path/to/prepared/26.3 \
  --dev-mod "$PWD/target/content/ae2"

python3.12 tools/package.py

# Воспроизводимый игровой smoke: новый мир, затем restart и проверка NBT.
python3.12 tools/smoke.py --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_power.py --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_drive.py --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_drive_gui.py --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_drive_automation.py --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_drive.py --portable --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_drive.py --creative --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_drive.py --tools --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_platform_tags.py --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
python3.12 tools/smoke_item_charging.py --ctl /path/to/horizon-ctl \
  --resources /path/to/prepared/26.3
```

`target/ae2-0.1.0.hnmod` содержит исходники native-мода и лицензии, без
Minecraft/AE2 assets. Контент хранится отдельно в `target/content/ae2`.
Установка/пересборка через launcher ещё требует отдельной проверки.

`fetch-engines` создаёт независимые закреплённые checkout под `target/upstream`;
существующие чужие репозитории не меняет. На этой машине пути к установленным
инструментам и ресурсам также записаны в игнорируемый `target/local-env.sh`.
`fetch-platform` закрепляет исходники generated c tags NeoForge 21.1.169;
prepare_content.py требует этот чистый checkout (параметр `--platform`).

```text
mixins/ae2/     правила, Horizon-адаптер, manifest и маленькие helpers
  keys.rs      компонентные ключи, fuzzy, signed counters
  storage.rs   ячейки и их ограничения
  cells.rs     native NBT/component codec, cell persistence, tooltip
  grid.rs      ME topology/pathing
  power.rs     EnergyService / per-node buffer / overlay rules
  mounts.rs    NetworkStorage priorities, queues и recursion guard
  service.rs   StorageService providers/cache/watchers
  drive.rs     ten-cell provider, idle drain, status
  drive_glue.rs native Drive slots, mounts, original inv persistence
  network.rs   full-block connections, split/merge и перенос энергии
  grid_glue.rs server level hooks, NBT и per-dimension state
src/lib.rs     те же правила как тестируемая библиотека
tools/         подготовка, инвентаризация, сборка, упаковка и проверки
docs/          план всего переноса, список отличий, исходный реестр
target/        игнорируемые исходники, jar, контент, сборки и тестовый мир
```

Код, производный от AE2: LGPL-3.0-or-later; уведомление MIT для производных
правил API — LICENSES/AE2-API-MIT. Build tooling: MIT.
Модели/текстуры AE2: CC BY-NC-SA 3.0; они не входят в репозиторий или `.hnmod`.
Подробности и авторство: [NOTICE.md](NOTICE.md). Проект не является официальным
портом Team Applied Energistics, Horizon, Mojang или Microsoft.
