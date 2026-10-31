# Текущая точность и открытые ограничения

Все строки ниже относятся к AE2 19.2.18 / Horizon aa27eca. Наличие исходников,
компиляция и resource carry не доказывают игровую эквивалентность.

| Подсистема | Написано | Что остаётся |
|---|---|---|
| BasicCellInventory | basic/portable/tool layouts, байты/типы/округление, exact/fuzzy partitions, distribution/void, simulate/modulate/extract; native codec, nested restrictions и Drive mounts; original Java basic и tool profile traces | терминал, полное восстановление malformed/missing content, addon handlers и actual upgrade validation |
| AEKey / GenericStack / KeyCounter | компонентные ключи, typed NBT, signed long counts, fuzzyEquals и FuzzySearch, signed deficits/zero pruning | prototype normalization, wrapped items, packet codecs, missing-content placeholders, drop/display API |
| ME topology/pathing | controller validation, BFS priorities, bottleneck ancestry, DFS counts, dense/normal/ad-hoc, P2P rules; 2000 Java differential cases; пять world full blocks, Drive require-channel | остальные block/part nodes, полная grid service identity/merge priorities, security/listeners/boot events |
| StoredEnergyAmount / EnergyService | amount/capacity/simulation, storage events, приоритеты, per-node buffer, public power delay, averages/cache, overlay/passive rules; full-block world binding; 2000 Java traces | addon storage providers, полные watchers/event callbacks, QuartzFiber и passive-generator blocks, config integration, полные watcher/addon oracle scenarios |
| NetworkStorage | mount priorities, preferred passes, mutation queues, recursion guard, weak links; 2000 Java traces/410882 операций; native StorageService/Drive consumers | MEStorage handler filters, ME Chest/terminal и остальные providers |
| StorageService | node/global provider identity, refresh/unmount, lazy public cache/private counts, key/watch-all changes и permanent watcher destroy | Java service oracle, world watcher hosts, full callback/hash iteration/exception fidelity, debug dump |
| DriveBlockEntity / DriveWatcher | десять slots для всех 23 built-in storage handlers, online/priority mounts, REQUIRE_CHANNEL, 2 AE + disk drain, front exclusion, original inv.itemN + Horizon Items mirror, packed status; native hopper/dropper filter/limit/flush и world restart | external capability API, dynamic cell chassis/LEDs/sync, all inventory/drop/chunk paths, reload invalidation и special menu actions |
| MatterCannon / ColorApplicator storage | 512 bytes/1 type и 2048 bytes/27 types, recipe/tag item admission before void, positive-weight first recipe, paintballs/dyes/snowball, 0.5 idle drain, filled-tool nesting и private-energy retention | handheld charging/shooting/painting/cycling, full item APIs/visuals, live recipe/tag reload, unknown addon condition/class adapters и matching recipe iteration ties |
| CreativeCellInventory / Handler | exact configured item/fluid keys, unlimited insert/extract simulate/commit, Integer.MAX_VALUE listing, preferred, TYPES_FULL и zero drain, empty nesting, no-op persist; native mounts и 2000 Java traces | tooltip image/shift list, workbench/config writer, descriptions, full item/handler/addon API и source iteration order |
| PortableCellItem | половина bytes, прежний overhead, item type limits 54/45/36/27/18 и fluid 18, 0.5 idle drain, component-preserving backend/Drive mounts и storage tooltip | handheld terminal/menu, private charging/use/disassembly, energy tooltip, item color/LED и full upgrades lifecycle |
| DriveMenu / DriveScreen | original texture/layout, десять cell slots с лимитом 1, authoritative filters, pickup/shift/drag/hotbar rules, main-before-hotbar shift order и isolation; game restart | priority button/subscreen, ghost icons, AEBaseMenu bulk actions, external multiplayer и addon cell registry |
| EnergyCellBlockEntity | normal/dense/creative block entities, реальные сетевые хранилища, internalCurrentPower и proxy.e persistence | fullness drawing/sync, comparator, energy components у item/drop/placement, redstone/template side effects |
| Charger recipes | таблица ae2:charger, local 1600 AE, threshold 1599, interval 10, random > 0.8, один предмет, сохранение энергии/слота; сеть пополняет private buffer до 800 AE/call | инструменты, config multiplier/rates, hopper фильтры, эффекты и рендер предмета |
| Wooden Crank | rotation/hits/charge, 18 ticks/160 AE, busy window, 11 отказов ломают рукоятку, charger-side restriction | placement/survival, fake-player rule, статистика, анимация и sync |
| Common platform tags | 461 generated c tags NeoForge 21.1.169, exact source hashes/licence, platform-before-AE2 layering/replace и real item predicate/restart checks | runtime membership всех registries/тегов, NeoForge API/events/recipe conditions и полная survival progression |

## Временные ограничения адаптера (не целевое поведение)

- Новый `cells.rs` использует исходные AE2 поля `id`, `components`, `#t`, `#` и
  компоненты `ae2:storage_cell_inv`, `ae2:storage_cell_config_inv`, `ae2:upgrades`.
  Неизвестные типизированные component tags сохраняются через Horizon NBT.
  Полное сравнение эффективных компонентов с registry prototypes ещё не
  реализовано; исправления/нормализация известных компонентов Horizon могут
  отличаться от Minecraft 1.21.1. Это остаётся задачей component adapter.
- Если компонент ячейки повреждён, содержит неизвестный registry key, duplicate,
  nonpositive count или превышает ёмкость, adapter возвращает ошибку и оставляет
  исходный ItemStack без изменений. Оригинал умеет восстанавливать missing
  content placeholders и некоторые некорректные состояния иначе; эта ветка
  ещё не перенесена. Нельзя выдавать текущий отказ за полный fault-tolerant codec.
- В native tooltip resolver сейчас знает предметы из Items и vanilla fluids
  water/lava; реестр custom fluids и native fluid capability ещё не подключены.
- В мире сейчас подключены Charger, три energy cell и Drive. CableBus,
  controller, терминалы и остальные nodes ещё не подключены. Drive требует
  канал; ад-hoc limit и отключение при нехватке канала проверены тестом binding.
  Full blocks соединяются соседними открытыми гранями;
  Charger и Drive исключают front. Grid service merge/split переносит per-node buffer,
  сохраняет сервис с pivot при разделении и более крупный при слиянии. Полные
  grid priorities, ties, listeners, ownership/readiness и event ordering pending.
- Java oracle использует оригинальный PathingCalculation и извлечённые без
  изменений pathing-методы GridNode/GridConnection. Окружение, события и коллекции
  заменены заглушками. Входы — связные сети с одним online controller и одинаковым
  порядком узлов/рёбер; controller conflict/ad-hoc отдельно покрыты Rust-тестами.
  Это не доказательство world/NeoForge/multiplayer parity. Java identity-hash
  порядок отдельных ties и реальный порядок регистрации grid nodes пока не
  воспроизводятся целиком; выводы oracle относятся к заданному входному порядку.

- Динамический `ChargerInvFilter` нельзя выразить статическими `ModContainer`
  API 40: разные правила вставки/выемки по текущему предмету. Автоматизация
  Charger временно отключена; ручная вставка и извлечение поддерживаются.
- EnergyService/GridEnergyStorage/StoredEnergyAmount/EnergyOverlayGrid сверены
  с неизменённым Java: 1000 single-grid и 1000 overlay traces, 462175 операций.
  Game/event/collection infrastructure stubbed; finite hosts используют исходный
  StoredEnergyAmount. Нет watcher callbacks/addon handlers/creative и NeoForge
  lifecycle oracle. Equal-priority provider/overlay ties используют stable IDs
  вместо Java identity hashes, поэтому traces избегают таких ties. Native
  adapter пока использует default config.
- NetworkStorage отдельно сверён с неизменённым Java на 2000 traces/410882
  операций. Host inventories — bounded test bins, ключ один; это проверка
  mount/routing/queue правил, не persistence/cell bytes/provider lifecycle.
  Слой подключён к native Drive через StorageService. Операции принимают opaque action
  source и передают его без изменений; native security ещё не реализована.
  Невалидные провайдеры, возвращающие больше запрошенного, отклоняются;
  пользовательские Java equals/exception behaviours не являются текущим
  native API контрактом. Для циклических links используются weak handles.
- Drive открывает экран с original PNG/layout и десятью cell slots. Транспорт
  native protocol — generic_9x2 с восемью скрытыми, запрещёнными padding slots;
  это не дополнительные слоты Drive и не AE2 wire protocol. Сервер определяет
  policy по actor/window/реальному блоку на каждом click packet. Ordinary barrel
  regression PASS. Из 23 допустимых upstream built-in cell families backend пока
  подключает все 23: двадцать basic/portable, creative, Matter Cannon и Color
  Applicator. Для tools required recipe/tag context; fallback unrestricted
  handler запрещён. Это storage backends, не полные handheld tool actions.
  Addon registry, priority UI, ghost slot icons и AEBaseMenu special bulk actions
  ещё не перенесены. Hopper/dropper access со всех шести сторон проверен,
  включая offline, full Drive и failed pull с возвратом компонентов; внешнего
  native capability API и совместного Create smoke ещё нет. Housing — исходный drive_base,
  runtime cell models/LED renderer ещё не подключены.
- Native Drive отражает inv.item0..item9 в Horizon Items для сохранения/слотов;
  это одни и те же ItemStacks, не два набора содержимого. Неизменённый echo
  не перезагружает изменённый backend; malformed inv сохраняется и unmount-ится.
  Это более строгий отказ, чем source recovery, и не доказательство всех ошибок
  codecs. Перед menu packets, hopper slot mutation и player destroy добавлен flush;
  native container changed немедленно обновляет providers. Полные inventory extraction,
  explosion/drop и chunk unload/save paths ещё нужно закрыть до терминалов.
- StorageService пока не имеет Java oracle. Cache/watchers проверены Rust,
  world Drive и native merge/split tests. Уведомления в core синхронные;
  callbacks с мутацией живых Java iterator-коллекций и identity-hash порядок
  пока не воспроизведены. Native world watcher consumers ещё не подключены.
- BasicCellInventory сверён на 2000 traces/572000 операций с неизменённым
  Java-классом. Item/key/component/fastutil/partition/upgrades — stubs, fake
  cell descriptors независимо задают basic/portable profiles. Проверяются exact
  partitions и callbacks blacklist/nested cells, simulate/commit/reopen, byte/type
  metadata и каждое количество. Actual fuzzy matcher, registry/component codecs,
  validation installed upgrades, malformed/missing recovery и NeoForge/world
  lifecycle этим oracle не доказываются. Хэши: docs/cells-oracle.json.
- CreativeCellInventory и CellState компилируются без изменений в отдельном
  oracle: 2000 traces/572000 операций PASS. Key/config infrastructure stubbed;
  listing сравнивается по каждому ключу, не по HashSet iteration order. В native
  конфигурации обрабатываются первые 63 nullable slots, пустые placeholders
  допускают nesting, лишние entries игнорируются как в ConfigInventory.
  Полные config component normalization, writer/UI и actual NeoForge lifecycle
  остаются открытыми. World smoke подтверждает config/power/restart; реальное
  unlimited extraction/mount/unmount проверено native integration, не терминалом.
- Tool profile oracle: ещё 2000 traces/572000 операций с unchanged
  BasicCellInventory/CellState и настоящим restricted Rust backend. Java tool
  classes не компилируются: capacity descriptors и blacklist callback stubbed.
  Actual recipe/tag expansion проверено native tests с реальным Items catalog;
  known `not(tag_empty)` conditions supported, unknown conditions (даже под not)
  не допускают рецепт. Полная NeoForge condition API остаётся открытой.
  Admission snapshot обновляется при загрузке/смене item; live reload invalidation
  ещё нужен. Для overlapping recipes используется порядок native recipe table;
  source RecipeManager ties ещё не сверены. Отчёт docs/tool-cell-oracle.json.
- Блоки world adapter опрашивают loaded block entities в pre/post hooks. Это
  сохраняет изменения NBT/блоков внутри тика, но полная source lifecycle/event
  модель и производительность больших сетей ещё не подтверждены. Неизменённый
  NBT не пополняет израсходованную в текущем тике энергию.
- Energy cell content пока показывает fullness=0. Все оригинальные модели
  остаются в assets, выбор по реальному уровню и sync ещё не подключены.
  Рецепты normal/dense energy cells восстановлены из jar, survival chain пока
  не проверена (в том числе Inscriber/processors и внешнее питание).
- Charger принимает только рецептные предметы без неподдержанных компонентов
  результата. Зарядка инструментов и GuideME recipe результата с components
  не подменяются обычным предметом. Конфиг в адаптере пока default multiplier 1.
- `minecraft:dispenser` даёт шесть направлений и block entity, но не `spin`.
  Charger показывает исходную модель с `spin=0`. Placement, outline/collision,
  материал/звуки и нежелательные template behaviours требуют отдельных hooks.
  Crank пока статичен, без correct orientation/render/sync.
- Извлечение из Charger создаёт item entity перед лицом блока с нулевой
  начальной скоростью. В оригинале `Platform.spawnDrops` использует свою
  случайную позицию/скорость. Перенос формы и drop RNG ещё не выполнен.
- Случайный sample Charger берётся из Horizon `world.random.next_float()`
  только когда выполнены условия Java. Порядок вызовов RNG всего мира между
  Java Minecraft и Horizon ещё не сравнивался; tick-for-tick oracle не выполнен.
- Рецепты во вложенных каталогах Horizon не читает. Подготовка flatten-ит имена
  с записью исходного → нового ID в `ae2-recipe-map.json` и отказывает при
  коллизии. Ссылки на исходные recipe IDs (advancements, UI, книги и прочее)
  требуют будущего engine fix/adapter; это временная несовместимость.
- Common c tags теперь берутся из NeoForge source revision 863802b, соответствие
  21.1.169 подтверждено official changelog/PR 2245. Исходники закреплены отдельно
  в upstream.lock.json; native/AE2 pins не менялись. Копируются 241 item, 115 block,
  12 fluid, 82 worldgen, 6 enchantment и 5 entity tags. World test проверяет только
  семь item tags, 14 positive/14 negative predicates с restart, не все 461.
  Source/merged metadata — docs/platform-tags-report.json; Java runtime платформы,
  vanilla namespace overrides и остальные NeoForge services этим не перенесены.
- Остальной carry — каталог данных, где часть блоков отсутствует или использует
  vanilla template. Нельзя считать каждый видимый предмет функциональным.
  Неподдержанные custom recipe types сохраняются как данные, но не выполняются.
- Native bundle и content mod имеют id ae2, helpers/modules уникальны ae2_*.
  Соседний Create не изменяется. Совместная работа двух модов и общие
  item/fluid/energy capabilities пока не проверены.
- Числа в core ограничены валидным игровым доменом. Rust отклоняет отрицательные
  количества типовым u64, non-finite energy не принимается; Java не везде
  проверяет такие неигровые входы. Это защитная граница, не сравнение всех NaN.

## Как подтверждать 1:1

Для каждой механики сохранять путь и hash Java, перенесённые методы, Rust
тесты на границы и состояние, затем oracle-вход/выход и игровой сценарий.
Считать feature завершённой только после механик, client state, persistence,
краевых случаев и сетевой авторитетности. При API gap сначала оставить запись
здесь, затем подготовить отдельный engine patch; исходный checkout не менять.
