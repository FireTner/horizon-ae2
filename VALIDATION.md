# Проверка переноса

Дата: 1 ноября 2026. macOS arm64, Rust 1.99.1, Python 3.12.
AE2 db17504 / Horizon aa27eca / horizon-mixins 3f99a5c.

## Storage backends двух инструментов — текущая проверка

- Standalone **85 PASS**, fmt/rustfmt/clippy `-D warnings` и Python **10 PASS**,
  target/tool-checks-final.log. Native **105 PASS**, target/tool-native-tests-final.json.
  Проверены capacities 4032/16128 для одного типа, 27 component variants у Color,
  blacklist before void, old-content extraction, source recipe first-match,
  реальные Ingredient/tag lookups, known conditions и отказ при unknown/not.
- Codec tests проверяют required admission context, nonempty tool nesting,
  private stored_energy retention, roundtrip и zero-count handler rejection.
  Tool CellLayout задаёт source bytes/type/drain явно; внутренний Tier marker
  используется только для codec identity, не показывается как свойство tool.
- **2000 tool traces / 572000 операций PASS**, docs/tool-cell-oracle.json.
  Компилируются unchanged BasicCellInventory/CellState; profiles и callback
  blacklist — stubs. Сверяется настоящий restricted Rust backend. Actual Java
  tool admission methods, world/recipe ordering/reload этим oracle не проверены.
  Basic/portable regression с новым wrapper: ещё 572000 PASS.
- Native build/check PASS: target/tool-native-build.json,
  target/tool-native-check-final.json. Ранний compiler error выявил Arc<Value>
  у CustomRecipe.json; adapter использует настоящий object view, pins сохранены.
- World smoke PASS: target/smoke/drive-tools-yj8oapg3, отчёт docs/tool-world-smoke.json.
  Matter Cannon с 123 iron nuggets + Color Applicator с 1000 red dye,
  собственная энергия каждого 1234 AE, original inv/Items, front/offline и restart.
  Точный расход Drive/channel + оба tools — **3.015625 AE/tick**. Полные world
  terminal admission/transfer ещё не проверены; соответствующие фильтры — core/native tests.
- В Drive теперь 23 встроенных handlers. Handheld charge/use/shoot/paint,
  dynamic render/API, reload invalidation, unknown addon conditions/class handlers
  и RecipeManager ordering ties остаются открытыми. Source inventory:
  **59 partial / 1370 pending**, complete нет. Свои test processes завершены.

## Общие теги платформы — предыдущая проверка

- Generated c tags берутся из чистого pinned NeoForge checkout 863802b6dfdb1d5e48c2fda093ed9d01df0f4cd9.
  Version mapping: official 21.1.169 changelog → PR 2245 → merge commit;
  ссылки и точный pin в upstream.lock.json/NOTICE.md. AE2/engine pins сохранены.
- **461 tags**: 241 item, 115 block, 12 fluid, 82 worldgen, 6 enchantment,
  5 entity_type. 15 совпадающих путей AE2 накладываются поверх platform layer;
  explicit replace сохраняется. Старые три ручных списка удалены.
  Source SHA-256 каждого файла и лицензия: docs/platform-tags-report.json;
  LGPL-2.1 text присутствует в content/data/ae2/licenses/neoforge-lgpl-2.1.txt.
- Standalone **81 PASS**, fmt/clippy и Python **10 PASS**. Новые tests проверяют
  overlay/replace, optional entries/order, provenance/licence и отказ при dirty
  platform checkout без замены предыдущего generated output.
  Полный лог target/platform-checks-final.log.
- Content prepare/build/check PASS: target/platform-content-final.log,
  target/platform-content-build-final.log, target/platform-content-check-final.json.
  File name лицензии приведено к lowercase для native package path contract.
  Native check PASS: target/platform-native-check.json. Rust runtime не менялся;
  прежние **97 native binary + 5 native library** tests относятся к тому же коду.
- Real world smoke PASS: target/smoke/platform-tags-kwyjn5mu, копия отчёта
  docs/platform-tags-world-smoke.json. Через настоящие `/execute if/unless items`
  проверены семь item tags, включая original AE2 ender_dust: **14 positive /
  14 negative** checks до/после restart. Свои процессы завершены.
- Это проверка common tag data и native layering. Membership всех тегов/registry,
  NeoForge API/events/recipe condition semantics и all survival crafting остаются
  открытыми. Source inventory: 57 partial / 1372 pending, complete нет.

## Creative storage backend — предыдущая проверка

- Standalone **81 PASS**, fmt/rustfmt/clippy `-D warnings` и Python **8 PASS**,
  target/creative-checks-final.log. Native **97 PASS**,
  target/creative-native-tests-final.json. Общий backend enum сохраняет прежние
  finite правила и добавляет creative configured-key storage.
- CreativeCellInventory.java и CellState.java компилируются неизменёнными:
  **2000 traces / 572000 операций PASS**, docs/creative-cell-oracle.json.
  Проверяются simulate/commit insert/extract, per-key listing cap Integer.MAX_VALUE,
  preferred, nested/empty rules, TYPES_FULL, zero drain и reopen. Key/config
  infrastructure stubbed; hash iteration, actual component codecs и UI не проверены.
- BasicCellInventory повторно сверён с исходным CellState: ещё **572000 PASS**,
  docs/cells-oracle.json. Native тесты также проверяют component codec, первые
  63 nullable config slots/лишние entries, empty nesting, реальный Network/StorageService
  mount, извлечение 10^12, zero disk drain, packed status и offline unmount.
- Native build и check PASS: target/creative-native-build.json,
  target/creative-native-check-final.json. Warnings про unused API и strip toolchain
  остаются; engine/source pins сохранены.
- World smoke PASS: target/smoke/drive-creative-djhqrho5, отчёт
  docs/creative-world-smoke.json. В slot 0 — mixed iron/water config, в slot 9 —
  пустая creative cell; общий Drive/channel drain **2.015625 AE/tick**, без disk
  overhead. Config, offline и restart сохранены. Этот smoke не извлекает ресурсы
  через ещё отсутствующий terminal; mount/extract подтверждает native test выше.
- Portable regression на новом enum: PASS, target/smoke/drive-portable-fi8or0cv.
  Exact drain, содержимое, private energy и restart сохранились.
- Source inventory: **57 partial / 1372 pending**, complete нет. Creative tooltip
  image/shift list, Workbench/config writer, descriptions и полный handler/item API
  остаются открытыми. Тестовые клиенты завершены.

## Portable storage backend и original cell oracle — предыдущая проверка

- Standalone **78 PASS**, fmt/rustfmt/clippy `-D warnings` и Python **8 PASS**;
  target/portable-checks-final.log. Native **91 PASS**, target/portable-native-tests.json.
  Новые тесты покрывают все десять portable profiles, половину bytes, сохранённый
  overhead, item limits 54/45/36/27/18, fluid 18 и 0.5 AE/tick; codec сохраняет
  stored_energy/energy_capacity и отклоняет подмену basic/portable identity.
- **2000 Java traces / 572000 операций PASS**, zero mismatches:
  docs/cells-oracle.json. BasicCellInventory.java копируется и компилируется
  неизменённым; infrastructure, exact partition matcher и cell descriptors stubbed.
  Сверяются insert/extract simulate/commit, byte/type metadata, every-key counts,
  exact whitelist/inverter/distribution/void, nested/blacklist callbacks и reopen.
  Actual fuzzy matcher, Minecraft component/upgrade validation и world/NeoForge
  этим oracle не проверяются. Profile source hashes включены отдельно.
- Native build PASS: target/portable-native-build.json; native check:
  target/portable-native-check-final.json. Engine pin сохранён.
- World smoke PASS: target/smoke/drive-portable-6xwx3vus, копия отчёта
  docs/portable-world-smoke.json. Portable item 256k + fluid 1k в slots 0/9
  потребляют вместе с Drive/channel ровно **3.015625 AE/tick**; проверены front
  isolation, offline/restart и содержимое 123 iron/1000 water. После restart тот
  же расход. Все значения stored_energy у обоих дисков (inv и Items mirror)
  во всех трёх записанных состояниях проверены отдельно: ровно 1234 AE.
- Перенесён storage backend и capacity tooltip. Handheld terminal/menu,
  зарядка/применение/разборка, energy tooltip и color/LED не объявляются готовыми.
  Source inventory: **54 partial / 1375 pending**, complete нет.
- Basic-cell regression на новом клиенте: PASS, target/smoke/drive-317pf1x_.
  Exact basic idle costs, original inv, front/offline и restart сохранены.

## Drive automation — предыдущая проверка

- Native **87 PASS**: два новых теста проверяют настоящий используемый insert
  helper, component identity, stacked remainder, occupied/rejected/zero inputs.
  Отчёт target/drive-automation-native-tests.json. Standalone **76 + 8 Python** PASS.
- Native build PASS: target/drive-automation-native-build.json. Final native
  check — target/drive-automation-native-check-final.json.
- Game smoke PASS: target/smoke/drive-automation-0pywd8ek, копия отчёта
  docs/drive-automation-smoke.json. Настоящие hoppers покрывают up + четыре
  горизонтальные стороны, dropper — нижнюю сторону. Front ME исключён только
  для сети: item insertion там разрешён, как в AEBaseInvBlockEntity.
- Stone отвергается; offline insert/extract сохраняет cell component с 123 iron.
  Неудачный pull в hopper с пятью несовместимыми стеками возвращает тот же диск.
  Стек из шести cells заполняет пять свободных слотов по одной штуке и оставляет
  одну в источнике: десять физических slots и скрытый transport padding соблюдены.
  Full Drive и remainder пережили restart. Свои процессы smoke закрывает.
- Повторный GUI smoke после automation hooks: PASS,
  target/smoke/drive-gui-jb_ak94l; Drive shift/filter, vanilla barrel и restart.
- Перед slot mutation — flush живых ячеек, container changed сразу обновляет
  mounts. World mutation через terminal и dirty backend ещё не проверяется этим
  smoke: терминал не реализован. General external capability/симуляция и Create
  integration, all chunk/drop paths остаются открытыми.

## Drive GUI — предыдущая проверка

- Standalone: **76 PASS**, fmt/rustfmt/clippy `-D warnings` PASS, Python **8 PASS**,
  inventory **51 partial / 1378 pending**, complete нет. Лог target/drive-gui-final-checks.log.
- Native binary tests: **85 PASS**, target/drive-gui-native-tests.json.
  Native library tests: **5 PASS**, target/drive-gui-library-second.json;
  три из них вызывают настоящий server menu click engine с реальным Items catalog.
  Проверены reject/cap=1, drag, hotbar swap, forward shift order, hidden padding
  и actor/window metadata isolation; ни один resource test не пропущен.
- Native build PASS: target/drive-gui-native-build.json; final check:
  target/drive-gui-native-check-final.json. Ранее при планировании выявлены
  ограничения HEAD/RETURN для функций с closures и syntactic helper result types;
  новая интеграция использует поддерживаемые HEAD hooks, engine pin не менялся.
- Game GUI smoke PASS: target/smoke/drive-gui-4o68o5bf, копия отчёта
  docs/drive-gui-smoke.json. Original drive.png осмотрен на screenshot:
  176x201, два столбца/пять строк, правильное положение inventory и текста.
  Проверены pickup/shift, отказ stone, порядок main inventory перед hotbar,
  сохранение предметов и restart; следующая vanilla barrel сохраняет свои правила.
- Повторный backend smoke на клиенте с GUI: PASS,
  target/smoke/drive-kvmdpegz. Exact idle costs, original inv.item0/item9,
  counts 123/1000, front isolation и restart снова прошли.
- Не закрыты: priority button/subscreen, ghost icons, специальные bulk actions
  AEBaseMenu, external multiplayer, hopper access и runtime chassis/LED renderer.
  GUI принимает все 23 исходных built-in типа ячеек; storage backend подключает
  только десять basic item/fluid cells. Native warnings о неиспользованных API
  и toolchain strip warning остаются видимыми. Тестовые процессы завершены.

## StorageService и Drive backend — предыдущая проверка

- Standalone: **74 PASS**, fmt/clippy с `-D warnings` и Python **8 PASS**.
- Native: **85 PASS**, в том числе полное подключение mounts/service/drive,
  original inv codec, offline/online remount, network merge/split и отсутствие
  mounts в отброшенном сервисе при сохранённом external capability handle.
  Отчёт `target/drive-native-tests-integration.json`.
- Native build PASS: `target/drive-native-build.json`. Check актуального дерева:
  `target/drive-native-check-final.json`; лог standalone: target/drive-final-checks.log.
- Drive world smoke PASS: `target/smoke/drive-5dq1vz2t/result.json`, копия
  отчёта docs/drive-world-smoke.json. Точный расход за 26 тиков:
  empty — 52.40625 AE (2 + 2/128 за тик); item 1k + fluid 4k — 91.40625 AE
  (2 + 0.5 + 1 + 2/128). После restart повторяется тот же расход.
- Исходный inv.item0/item9 загружен и сохранён, количества 123 iron и 1000
  water сохранены. Front не подключается, отключение не стирает диски.
  Первый smoke ошибочно был запущен до окончания build и закономерно не увидел
  новый backend; он завершился, повторный запуск выполнен после cargo_exit=0.
- Видимая housing использует исходный drive_base. Screenshot осмотрен:
  runtime chassis/LEDs ещё отсутствуют. Меню не открывается; GUI, terminal
  transfers, filtered automation, полный chunk/drop/multiplayer ещё не проверены.
- StorageService cache/watchers имеют Rust-проверки, но пока не Java oracle.
  Java storage oracle ниже относится только к NetworkStorage.
- Source inventory: 1429 файлов, **46 partial / 1383 pending**, complete нет.
- Charger power regression после новой Dimension/Drive интеграции: PASS,
  `target/smoke/power-wkijuezf`; finite/creative/front/restart снова проверены.
  Content build: target/drive-content-build.log. Все тестовые клиенты завершены.

## Энергосервис и full-block world adapter — предыдущая проверка

- Standalone: **63 PASS**, clippy с `-D warnings` PASS, fmt PASS.
- Native: **64 PASS**, check/build PASS. Отчёты:
  `target/energy-native-tests.json`, `target/energy-native-build.json`.
- Python: **8 PASS**; source inventory: 1429 файлов, 40 partial/1389 pending,
  полностью закрытых классов нет.
- `tools/smoke_power.py`: PASS, `target/smoke/power-6r_xgmd1/result.json`.
  Finite cell 2000 AE завершает charged crystal и оставляет суммарно 400 AE;
  front Charger не соединяется; creative питает без crank. Save/restart
  сохраняет 1600 AE Charger + 400 AE finite cell, затем рецепт расходует 1600.
- Повторный `tools/smoke.py` (ручной crank): PASS,
  `target/smoke/charger-1ta7ucsr`. Тестовые клиенты завершены.
- Контент пересоздан с original normal/dense recipes, actual hmod build PASS,
  лог `target/power-content-build.log`. Static fullness пока 0.
- В новом адаптере изменение NBT/блоков внутри тика учитывается перед записью.
  Неизменённый NBT не восстанавливает извлечённую за этот тик энергию.
- Java energy oracle: single-grid **1000 traces / 202028 операций PASS**,
  overlay **1000 traces / 260147 операций PASS**. EnergyService,
  EnergyOverlayGrid, GridEnergyStorage и StoredEnergyAmount компилируются
  без изменений из pinned source; game/event/collection infrastructure stubbed.
  Хэши и область: docs/energy-oracle.json, docs/energy-overlay-oracle.json.
  Выявлен и исправлен сброс генератора отдельного overlay при чужом link change;
  удаление активного генератора очищает его до следующего start tick.
- NetworkStorage: **2000 Java traces / 410882 операции PASS**, точное сравнение
  целых чисел после каждой операции. Сверены priorities, preferred passes,
  duplicate mounts, callback queues, self-recursion, listing delay. Host bins
  и key/counter infrastructure — заглушки; cells/provider lifecycle/security
  и world Drive этим oracle не проверяются. Отчёт: docs/mounts-oracle.json.
- Native test/build относятся к фактически подключённым модулям. mounts.rs
  пока не входит в native manifest и проверен standalone/oracle.
  Лог актуальных standalone проверок: target/mounts-final-checks.log.
  Full lifecycle, chunk/dimension/multiplayer, все остальные nodes, addon stores
  и client visuals перечислены в PARITY как незавершённые.

Воспроизведение: `source target/local-env.sh`, затем Python 3.12
`tools/check.py`, `tools/horizon.py test` и `tools/smoke_power.py --ctl
"$HORIZON_CTL" --resources "$HORIZON_RESOURCES"`.

## Предыдущий слой: ключи, ячейки и ME-pathing — история

- Standalone core: **38 тестов**, включая 11 topology/pathing tests,
  component-aware keys, signed counters, restore и fuzzy partitions.
- Native adapter: **29 тестов PASS**; `tools/horizon.py test` проверяет codec в настоящем Horizon,
  включая NBT Byte/Int, unknown component tags, большие signed количества,
  nested-cell restriction и атомарный отказ от повреждённых данных.
  Итоговые числа и полный stdout — `target/horizon-test.log`.
- Native `check` и `build`: PASS. Новые compiler warnings относятся к ещё
  неиспользуемым методам API до подключения Drive/энергосети; warnings не скрыты.
- Original-Java oracle: **2000/2000 совпадений**, ноль расхождений в totals и
  на каждом node/connection. Оригинальный `PathingCalculation.java` компилируется
  без изменений; pathing-методы GridNode/GridConnection извлекаются из pinned
  source. Отчёт с hash исходников: `docs/pathing-oracle.json`.
- Oracle не запускает Minecraft/NeoForge: infrastructure/services заменены
  заглушками. Это rule-level evidence только для связных controller-online
  графов с одинаковым порядком входных узлов и связей. Ad-hoc/conflict/long chains
  покрыты Rust-тестами, не этим Java oracle.
- Повторный игровой smoke на новом клиенте: PASS; свежие логи
  `target/smoke/charger-5i2a_704/`. Tooltip hook компилируется, но визуальная
  проверка hover/цветов/локализаций в окне ещё не выполнена.
- Python tooling tests: 8 PASS. Полный реестр: 1429 файлов, 33 partial,
  1396 pending; законченных классов по-прежнему нет.

Воспроизведение:

```sh
python3.12 tools/check.py
python3.12 tools/horizon.py test
python3.12 tools/pathing_oracle.py --java-home /path/to/jdk21 --cases 2000
```

JDK в этой сессии: установленный Homebrew OpenJDK 21.0.12.1; системные Java/Rust
не устанавливались и не перенастраивались. Для engine check/build используются
собственные pinned checkout в target/upstream.

## Первый игровой срез — история проверок

| Проверка | Результат |
|---|---|
| `tools/check.py`: cargo fmt, rustfmt всех injected файлов, clippy `-D warnings`, unit tests, Python regression tests, inventory check | PASS: 18 Rust + 8 Python tests |
| Случайная последовательность insert/extract | 10 000 операций, проверка сохранения количества, лимита типов и байтов на каждом шаге |
| Jar AE2 19.2.18 | Modrinth KDnFUmMm, размер 8236552, SHA-512 соответствует upstream.lock.json |
| Source inventory | 1429 Java-файлов с SHA-256; 11 partial, 1418 pending; ни один класс не объявлен полностью готовым |
| `horizon-port-pack check` | PASS; ограничения carry сохранены в docs/baseline-port-pack.json |
| Подготовка из jar + prepared 26.3 | PASS; рецепты Charger/Crank восстановлены, subfolder IDs remap записан, коллизии проверяются |
| `cargo-horizon-mod build` контента и `check ae2.hmod` | PASS, registries API 0.2 |
| `tools/horizon.py check` | PASS, native Charger/Crank adapter компилируется в самом Horizon; compiler warnings отсутствуют |
| `tools/horizon.py build` | PASS, fast profile, запускаемый arm64-клиент |
| Source-only `ae2-0.1.0.hnmod` | PASS: два запуска packager дают идентичные bytes; реальный Horizon package checker принимает архив, exact pins совпадают |
| Одновременный mixin plan с horizon-create | PASS: 20 injections, 8 modules; только планирование, не совместная игровая проверка |
| `tools/smoke.py` | PASS: новый изолированный мир, полный цикл Charger/Crank, извлечение, перезапуск, slot + ненулевые AE сохранены |

В `fast` build были предупреждения **toolchain**: `rust-objcopy` не смог удалить
debug info, потому что установленная копия инструмента не находит libLLVM.dylib.
Cargo завершился с кодом 0, клиент запустился и прошёл игровые проверки. Системный
Rust/LLVM не менялся. Это не основание заявлять «build без предупреждений».

## Наблюдалось в игре (первый этап)

Запуск offscreen на Apple M5/Metal, отдельные `game-dir` и `HORIZON_HOME` под
target; никакие пользовательские миры/options не использовались.

1. Из двух certus quartz crystals ровно один вставляется по клику в Charger;
   второй остаётся у игрока. Реальный slot содержит count 1.
2. Wooden Crank над Charger: после первого завершённого оборота NBT содержит
   160.0 AE, после девятого — 1440.0. После десятого начинается recipe work.
3. Slot становится `ae2:charged_certus_quartz_crystal`, count 1; энергия — 0.0.
   Нет выдачи предмета в обход рецепта: результат найден в таблице `ae2:charger`.
4. Пустая рука извлекает предмет как item entity перед машиной; slot пуст.
5. После вставки второго обычного кристалла и одного оборота: slot raw crystal,
   энергия 160.0. Save → exit → новый процесс клиента: оба значения сохранены.
6. Дополнительная ручная проверка: заряженный кристалл в slot пережил отдельный
   restart; crank rotation/charge/hits отсутствуют в сохранённом NBT, как в Java.

Воспроизводимый сценарий:

```sh
python3.12 tools/smoke.py --ctl /path/to/horizon-ctl --resources /path/to/prepared/26.3
```

Успешный запуск этой сессии: `target/smoke/charger-0i9l4qgu/` — result.json,
commands.json, два игровых лога и charger.png. Ручные пробы — `target/run/`.
Каждый запуск smoke создаёт новый каталог и завершает свои процессы.
Контрольные логи сборки: target/horizon-check.log, target/horizon-build.log.

Не проверены: полный world-level Java/NeoForge oracle, launcher install/rebuild, LAN/dedicated с
вторым игроком, полный совместный запуск с Create, automation/capabilities,
survival progression, chunk unload/dimensions, статистика/FX/анимации. Ядра
storage/energy протестированы отдельно и ещё не подключены к ME-предметам/сети.
Конкретные расхождения остаются в docs/PARITY.md.
