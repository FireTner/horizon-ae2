# Текущая точность и открытые ограничения

Все строки ниже относятся к AE2 19.2.18 / Horizon aa27eca. Наличие исходников,
компиляция и resource carry не доказывают игровую эквивалентность.

| Подсистема | Написано | Что остаётся |
|---|---|---|
| BasicCellInventory | байты/типы/округление, пять размеров, items/fluids, exact/fuzzy partitions, distribution/void, simulate/modulate/extract; native codec, nested restrictions и Drive mounts | GUI Drive/терминал, полное восстановление malformed/missing content, другие типы storage cells |
| AEKey / GenericStack / KeyCounter | компонентные ключи, typed NBT, signed long counts, fuzzyEquals и FuzzySearch, signed deficits/zero pruning | prototype normalization, wrapped items, packet codecs, missing-content placeholders, drop/display API |
| ME topology/pathing | controller validation, BFS priorities, bottleneck ancestry, DFS counts, dense/normal/ad-hoc, P2P rules; 2000 Java differential cases; пять world full blocks, Drive require-channel | остальные block/part nodes, полная grid service identity/merge priorities, security/listeners/boot events |
| StoredEnergyAmount / EnergyService | amount/capacity/simulation, storage events, приоритеты, per-node buffer, public power delay, averages/cache, overlay/passive rules; full-block world binding; 2000 Java traces | addon storage providers, полные watchers/event callbacks, QuartzFiber и passive-generator blocks, config integration, полные watcher/addon oracle scenarios |
| NetworkStorage | mount priorities, preferred passes, mutation queues, recursion guard, weak links; 2000 Java traces/410882 операций; native StorageService/Drive consumers | MEStorage handler filters, ME Chest/terminal и остальные providers |
| StorageService | node/global provider identity, refresh/unmount, lazy public cache/private counts, key/watch-all changes и permanent watcher destroy | Java service oracle, world watcher hosts, full callback/hash iteration/exception fidelity, debug dump |
| DriveBlockEntity / DriveWatcher | десять basic cells, реальные mounts по online/priority, REQUIRE_CHANNEL, 2 AE + disk drain, front exclusion, original inv.itemN + Horizon Items mirror, packed status; world restart | GUI/menu filters и automation, dynamic cell chassis/LEDs/sync, creative/other cells, all inventory/drop/chunk/capability paths |
| EnergyCellBlockEntity | normal/dense/creative block entities, реальные сетевые хранилища, internalCurrentPower и proxy.e persistence | fullness drawing/sync, comparator, energy components у item/drop/placement, redstone/template side effects |
| Charger recipes | таблица ae2:charger, local 1600 AE, threshold 1599, interval 10, random > 0.8, один предмет, сохранение энергии/слота; сеть пополняет private buffer до 800 AE/call | инструменты, config multiplier/rates, hopper фильтры, эффекты и рендер предмета |
| Wooden Crank | rotation/hits/charge, 18 ticks/160 AE, busy window, 11 отказов ломают рукоятку, charger-side restriction | placement/survival, fake-player rule, статистика, анимация и sync |

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
- Drive пока не открывает меню: его реальные десять слотов можно проверять
  через test control/исходный inv NBT. Unfiltered vanilla menu и hopper access
  отключены до переноса slot filters/limits, всех кликов и sync. Housing —
  исходный drive_base, runtime cell models/LED renderer ещё не подключены.
- Native Drive отражает inv.item0..item9 в Horizon Items для сохранения/слотов;
  это одни и те же ItemStacks, не два набора содержимого. Неизменённый echo
  не перезагружает изменённый backend; malformed inv сохраняется и unmount-ится.
  Это более строгий отказ, чем source recovery, и не доказательство всех ошибок
  codecs. Перед player destroy добавлен flush; полные inventory extraction,
  explosion/drop и chunk unload/save paths ещё нужно закрыть до терминалов.
- StorageService пока не имеет Java oracle. Cache/watchers проверены Rust,
  world Drive и native merge/split tests. Уведомления в core синхронные;
  callbacks с мутацией живых Java iterator-коллекций и identity-hash порядок
  пока не воспроизведены. Native world watcher consumers ещё не подключены.
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
