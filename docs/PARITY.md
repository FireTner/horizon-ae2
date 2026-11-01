# Текущая точность и открытые ограничения

Все строки ниже относятся к AE2 19.2.18 / Horizon aa27eca. Наличие исходников,
компиляция и resource carry не доказывают игровую эквивалентность.

| Подсистема | Написано | Что остаётся |
|---|---|---|
| BasicCellInventory | basic/portable/tool layouts, байты/типы/округление, exact/fuzzy partitions, distribution/void, simulate/modulate/extract; native codec, nested restrictions и Drive mounts; original Java basic и tool profile traces | терминал, полное восстановление malformed/missing content, addon handlers и полный upgrades lifecycle |
| ItemUpgradeInventory / UpgradeInventory | исходный `slot/item` container component, physical slot limits, supported cards и per-slot installed counts с host clamp для 22 finite storage items; исходный component сохраняется | writer/Workbench/callbacks/cache invalidation, addon upgrade registry и остальные hosts; полный Minecraft codec и malformed recovery |
| AEKey / GenericStack / KeyCounter | компонентные ключи, typed NBT, signed long counts, fuzzyEquals и FuzzySearch, signed deficits/zero pruning | prototype normalization, wrapped items, packet codecs, missing-content placeholders, drop/display API |
| ME topology/pathing | controller validation, BFS priorities, bottleneck ancestry, DFS counts, dense/normal/ad-hoc, P2P rules; 2000 Java differential cases; шесть world full blocks, реальный Controller и Drive require-channel | остальные block/part nodes/CableBus, полная grid service identity/merge priorities, security/listeners/boot events, расширенный original-Java multi-controller oracle |
| ControllerBlock / ControllerBlockEntity | public 8000 AE/default storage priority 0, idle 3, CANNOT_CARRY/DENSE flags, реальные controller positions/pathing, original NBT, 18 state/type variants, source column/interior/parity и powered-before-conflict status; native power/Charger/restart и 4000 verbatim Java visual cases | Network Status Menu, external power funnel/capabilities, immediate shape/ready/event timing, full chunk/drop/spatial lifecycle, material/emissive parity и Java/NeoForge world oracle |
| StoredEnergyAmount / EnergyService | amount/capacity/simulation, storage events, приоритеты, per-node buffer, public power delay, averages/cache, overlay/passive rules; full-block world binding; 2000 Java traces | addon storage providers, полные watchers/event callbacks, QuartzFiber и passive-generator blocks, config integration, полные watcher/addon oracle scenarios |
| NetworkStorage | mount priorities, preferred passes, mutation queues, recursion guard, weak links; 2000 Java traces/410882 операций; native StorageService/Drive consumers | MEStorage handler filters, ME Chest/terminal и остальные providers |
| StorageService | node/global provider identity, refresh/unmount, lazy public cache/private counts, key/watch-all changes и permanent watcher destroy | Java service oracle, world watcher hosts, full callback/hash iteration/exception fidelity, debug dump |
| DriveBlockEntity / DriveWatcher | десять slots для всех 23 built-in storage handlers, online/priority mounts, REQUIRE_CHANNEL, 2 AE + disk drain, front exclusion, original inv.itemN + Horizon Items mirror, packed status; native hopper/dropper filter/limit/flush и world restart | external capability API, dynamic cell chassis/LEDs/sync, all inventory/drop/chunk paths, reload invalidation и special menu actions |
| MatterCannon / ColorApplicator storage | 512 bytes/1 type и 2048 bytes/27 types, recipe/tag item admission before void, positive-weight first recipe, paintballs/dyes/snowball, 0.5 idle drain, filled-tool nesting и private-energy retention | handheld shooting/painting/cycling, full item APIs/visuals, live recipe/tag reload, unknown addon condition/class adapters и matching recipe iteration ties |
| CreativeCellInventory / Handler | exact configured item/fluid keys, unlimited insert/extract simulate/commit, Integer.MAX_VALUE listing, preferred, TYPES_FULL и zero drain, empty nesting, no-op persist; native mounts и 2000 Java traces | tooltip image/shift list, workbench/config writer, descriptions, full item/handler/addon API и source iteration order |
| PortableCellItem | половина bytes, прежний overhead, item type limits 54/45/36/27/18 и fluid 18, 0.5 idle drain, component-preserving backend/Drive mounts и storage tooltip | handheld terminal/menu, private use/disassembly, energy tooltip, item color/LED и full upgrades lifecycle |
| DriveMenu / DriveScreen | original texture/layout, десять cell slots с лимитом 1, authoritative filters, pickup/shift/drag/hotbar rules, main-before-hotbar shift order и isolation; game restart | priority button/subscreen, ghost icons, AEBaseMenu bulk actions, external multiplayer и addon cell registry |
| EnergyCellBlockEntity | normal/dense/creative block entities, реальные сетевые хранилища, internalCurrentPower и proxy.e persistence; finite fullness 0..4/original models и double comparator, deferred notifications в block-ticking chunks; creative single state без analog output; finite stored_energy player placement/first matching loot stack, live same-tick flush | exact TickManager/event/readiness/base settings lifecycle, all explosion/saved-BE/pick/loot-command paths, tooltip/creative charged variant и full material parity |
| Charger | таблица ae2:charger, local 1600 AE, threshold 1599, interval 10, random > 0.8, один предмет, сохранение энергии/слота; powered branch для 16 AEBasePoweredItems + 2 finite cell items с исходными rates и local/grid/overflow; сеть пополняет private buffer до 800 AE/call; native hopper/dropper automation по ChargerInvFilter со всех сторон | live config multiplier/rates, external capability API, полное inv/drop lifecycle, эффекты и рендер предмета |
| Wooden Crank | rotation/hits/charge, 18 ticks/160 AE, busy window, 11 отказов ломают рукоятку, charger-side restriction | placement/survival, fake-player rule, статистика, анимация и sync |
| Common platform tags | 461 generated c tags NeoForge 21.1.169, exact source hashes/licence, platform-before-AE2 layering/replace и real item predicate/restart checks | runtime membership всех registries/тегов, NeoForge API/events/recipe conditions и полная survival progression |
| EnergyCellBlockItem / item predicate | fixed capacities 200000/1600000, rates 800/1600, source inject/extract/simulate/overflow and 0.00001 removal, native Charger, no powered-item bar; original .2/.4/.6/.8 float model thresholds through render-only projection; stored_energy placement and first matching drop with live pre-break flush | complete tooltip/base settings/capability APIs; all saved-BE/explosion/pick/loot-command lifecycle; nonfinite/malformed native recovery |

## CableBus / кабели: текущая граница

`cable.rs` переносит five cable types, 17 source colors/prefixes/palette, ordinal
connection lengths, independent variant/size min/max, source node flags,
connection shape calls и visual channels. Dense flags **заменяют** PREFERRED;
normal max 8 и dense max 32 следуют grid mode. `part_storage.rs` сохраняет центр,
шесть side parts и шесть facades с lazy allocation: setPart(null) выделяет
массив, setFacade(null) делегирует remove и не выделяет его. Это storage rule,
не готовый multipart host или facade item; native saved-host parsing описан ниже.

`network.rs` принимает cable center profiles, применяет transparent/equal color
совместимость только к in-world edges, exposed side masks и сохраняет node ID
при смене цвета того же типа кабеля. Full blocks сохраняют transparent profile.
`Kind::from_block("ae2:cable_bus")` намеренно не угадывает цвет/тип: центр должен
читаться из host parts. Native saved-host parsing подключён к grid_glue;
настоящие CableBus registration/placement/part nodes,
callbacks, facade collision, partial selection/breaking, renderer, packets и
chunk/event lifecycle ещё не подключены. Мир по-прежнему видит прежние шесть
node kinds. Cable profiles в binding tests не являются игровым CableBus.

Java oracle `docs/cable-oracle.json`: 278956 случаев. API enums/AEColor и
CableBusStorage компилируются целиком без изменений; методы CablePart и
subtype getBoxes/цветовая совместимость — дословные fragments. Host/grid,
part connection lengths, power, neighbor density, base packet prefix, codecs
и renderer stubbed. Geometry сравнивает raw addBox arguments (включая NaN),
не normalized VoxelShapes. Сравниваются только bytes CablePart после base
prefix, не полный host update или Minecraft protocol. Truncated suffix
отклоняется Rust до изменения state; Java reader может менять state частично.
Это более строгий malformed gate, не заявленная эквивалентность recovery.

`cable_bus.rs` читает original `cable`, down/up/north/south/west/east и
facadeDown/Up/North/South/West/East, source hasRedstone (YesNo default 2).
Центр распознаётся по одному из 85 cable item IDs; buffered energy читается из
его `gn.e` только типа Double (EnergyService.addNode), owner — только
`gn.p` Int. Grid-local buffer ограничен 25 AE на node при default config. Корневой `proxy` не используется. Raw
part/facade/host/unknown service fields сохраняются; facade BlockState codec
ещё не исполняется. Host с неподдержанной side part не подключается к сети,
поскольку у неё должен быть собственный node/service. Это явная временная
граница; центр не заменяет все части одним узлом.

На raw NBT reload другой item ID (включая цвет) пересоздаёт part/node как
CableBusContainer.loadPart, в отличие от direct changeColor. Изменившиеся gn
данные пересоздают node; одно поле p не влияет. Отсутствующие/empty/p-only gn
равны. Собственные writes обновляют observed snapshot, поэтому не вызывают
новый reload. Persist проверяет точное совпадение текущего центра с observed
и не перезаписывает новую external NBT правку; меняет только cable.gn.
Unknown service data сохраняются до переноса их writers; это не полный
Grid.saveNodeData или CableBusContainer.writeToNBT. В частности, весь host
пока не пересериализуется через source part/facade registry.

Неподдержанные IDs, side parts, malformed/non-Int hasRedstone, invalid ordinal
и negative/nonfinite gn energy приводят к отказу с сохранением исходных данных.
Это более строгий временный gate: source умеет игнорировать unknown items и
часть malformed tags, что этим adapter не воспроизведено. `node_data.rs`
сверён с verbatim GridNode.areTagsEqualIgnoringPlayerId на 20064 случаях
(`docs/node-data-oracle.json`), но CompoundTag/value equality в Java oracle —
map/string stubs. Native opaque Tag equality не доказывает всех Minecraft NBT
NaN/nested/normalization cases; full readiness/destroy/service callbacks открыты.
GridNode.savedData в Java хранит данные предыдущего load/перехода grid,
а текущий polling adapter сравнивает новый gn с последним observed/own-save
snapshot. Отдельного native события explicit BE load (`/data` с unchanged gn
после расхода, chunk reload/readiness) ещё нет. Полное соответствие момента
reset требует event/readiness port, не только перенесённой функции equality.
Physical CableBus registration/collision/render/placement по-прежнему нужны:
этот decoder сам по себе не делает кабель доступным игроку.

`cube_builder.rs`/`cable_render.rs` переносят cable-only геометрию исходных
CubeBuilder/CableBuilder/CableBusBakedModel, включая все five types × 17 colors,
core override, straight shortcut, constrained attachment connections, machine
caps, texture names, odd/even channel layers, vertex RGB/fullbright и порядок
quads. Dense channel groups используют wrapping (channels+3)/4; straight dense
сегмент выступает до -0.01/16.01 pixels против z-fighting. Source smart machine
cap оставляет bright vertex color после reset texture/fullbright; small segment
сохраняет этот цвет. Математика float32 и порядок вычитаний не упрощены.

24405 Java cases, 467940 quads и 9358800 float fields совпали по bits
(`docs/cable-render-oracle.json`). Четыре builder/texture класса компилируются
целиком без изменений; QuadEmitter.square и cable-only model composition —
verbatim fragments. Sprite lookup — три контролируемых affine atlas rectangles;
MutableQuadViewImpl лишь записывает emitter values. Actual vanilla packed
vertices/normals/AO/shading, live atlas, GPU, render-state world collection,
attachment/facade models, particles и caches не проверяются/ещё не перенесены.
Quad.cull здесь только emitter metadata: source cable model отдаёт unculled
BakedQuads, это не разрешение отбрасывать грани native world mesher-ом.
Эти два geometry modules пока library-only, не включены в native manifest.
Их наличие в source archive не меняет вид игрового мира. Physical CableBus
registration/geometry/GPU integration остаётся следующим обязательным этапом.

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
- В мире сейчас подключены Charger, три energy cell, Drive и Controller. CableBus,
  терминалы и остальные nodes ещё не подключены. Drive требует
  канал; ад-hoc limit и отключение при нехватке канала проверены тестом binding.
  Full blocks соединяются соседними открытыми гранями;
  Charger и Drive исключают front. Grid service merge/split переносит per-node buffer,
  сохраняет сервис с pivot при разделении и более крупный при слиянии. Полные
  grid priorities, ties, listeners, ownership/readiness и event ordering pending.
- Controller и три energy cells используют original block states. Content API сначала создаёт cube
  row; native mod_content build adapter разворачивает Controller в 18 реальных states,
  finite cells — в пять fullness states каждый, creative — в один property-free state
  и согласованно обновляет девять JSON и шесть NBT tables. Vanilla IDs/rows не
  сдвигаются, последующие mod state/shape/default references переносятся вместе.
  Library проверяет чужой door/button с custom shapes и фактический renderer
  model loader. Original jar assets сохранены. Default — offline/block.
  Пока type/status пересчитываются на server post_tick; source immediate
  updateShape/onReady/listener order, Network Status Menu и power funnel ещё нужны.
  Source grid power importance/merge priority и полная service identity остаются
  общими открытыми задачами adapter.
- Finite Energy Cell status adapter отслеживает локальный запас отдельно от
  proxy.e и сохраняет neighborChangePending, пока chunk не сможет tick.
  После grid extraction/injection отложенно обновляются comparator и fullness;
  даже смена analog level в пределах одного fullness level посылает notification.
  Вместо dispenser используются source cube properties, glass sounds, hardness
  2.2/resistance 11 и no requires-tool. Ни facing/triggered, ни dispenser inventory
  больше не подмешиваются в эти три блока. Source TickManager scheduling,
  onReady/event order, все unload/drop пути и item energy components ещё нужны.
- Java oracle использует оригинальный PathingCalculation и извлечённые без
  изменений pathing-методы GridNode/GridConnection. Окружение, события и коллекции
  заменены заглушками. Входы — связные сети с одним online controller и одинаковым
  порядком узлов/рёбер; controller conflict/ad-hoc отдельно покрыты Rust-тестами.
  Это не доказательство world/NeoForge/multiplayer parity. Java identity-hash
  порядок отдельных ties и реальный порядок регистрации grid nodes пока не
  воспроизводятся целиком; выводы oracle относятся к заданному входному порядку.

- Native hopper/dropper Charger использует `ChargerInvFilter` через hooks машин:
  вставка разрешена для chargeable item либо ingredient любого Charger recipe;
  извлечение — `(chargeable && current >= maximum) || !recipe_matches`.
  Поэтому незаряженный powered item без рецепта извлекается сразу, как в Java.
  Рецепт проверяется по реальным native Items tags/recipe data при обращении,
  независимо от поддержки result components. Обычная вставка — slot 0/limit 1;
  source-loaded positive count>1 извлекается по одному без потери остатка.
  `AEBaseInvBlockEntity.getExposedInventoryForSide` не ограничивает грани:
  открыты все шесть, включая переднюю; grid front restriction не применяется.
  Перед изменением синхронизируется source/native echo; успешное изменение
  немедленно сохраняет inv.item0, failed pull возвращает исходный stack.
  Как InternalInventoryItemHandler, extraction не выпускает internal wrapped stacks.
  Malformed original остаётся blocked до explicit repair. General capability
  API/addon power registry и полный Java/NeoForge world lifecycle ещё открыты.
  Comparator adapter теперь использует source slot limit 1 и реальные 1/10
  слотов Charger/Drive, исключая 8 transport padding slots. Original inv count
  имеет приоритет; malformed original даёт нулевой сигнал в рамках существующей
  strict refusal policy, которую нельзя выдавать за полный source recovery.
  Float32 sum/division и Mth.lerpDiscrete сверены с байткодом Minecraft 1.21.1:
  inventory-signal-source-audit.json содержит official artifact URLs/hashes и
  hashes четырёх просмотренных методов. World inventory-signal-world-smoke.json
  проверяет реальные comparator/dust, все уровни заполнения Drive, raw recipe
  item в Charger, hopper/manual/GUI mutations, vanilla isolation и restart.
  Direct source overstack не clamp-ится: rule/native tests сохраняют значения
  выше 15; распространение таких сигналов через все redstone components ещё
  не проверено. Comparator остальных AE2 hosts и full world oracle ещё нужны.
  Исходные default max-stack properties теперь перенесены:
  58 registrations с лимитом 1 (включая patterns/wrapped/tiered tools), остальные
  217 carried item definitions с лимитом 64. Static pinned source audit находится
  в item-stack-source-audit.json; vanilla durability/stack-size связь сверена с
  [NeoForge 1.21.1 items](https://docs.neoforged.net/docs/1.21.1/items/) и
  [TieredItem](https://docs.neoforged.net/docs/1.21.1/items/tools/).
  Все 58 реально распределяются hopper по отдельным слотам без потери count/
  компонентов; full destination и restart проверены в item-stack-world-smoke.json.
  Component-specific max-stack overrides и остальные item properties/API ещё нужны.
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
- Placed energy cells уже используют исходные fullness states и модели,
  выбираемые по локальному заряду. Рецепты normal/dense energy cells
  восстановлены из jar, survival chain пока
  не проверена (в том числе Inscriber/processors и внешнее питание).
- Выполнение Charger recipe пока отказывает результатам с неподдержанными
  components. Automation admission/extraction использует ingredient membership
  и не исключает такой рецепт. GuideME result с components не подменяется
  обычным предметом. Конфиг в адаптере пока default multiplier 1. Полные addon
  ingredient/NeoForge condition codecs и RecipeManager ordering ещё нужны.
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

- Upgrade component reader исправлен: прежняя плоская форма `[{id,count}]`
  не соответствовала `ItemContainerContents.CODEC` у `AEComponents.UPGRADES`.
  Нормальная форма — `[{slot:0,item:{id:"ae2:void_card",count:1}}]`.
  Учитываются реальные слоты host и максимум поддерживаемых карт из InitUpgrades;
  stack count не умножает installed count. Energy-card multiplier отдельно
  считает raw physical cards, как Upgrades; зарядка ещё не подключена.
  Повреждённые формы, nonpositive counts и duplicate slots сейчас дают строгий
  отказ с сохранением исходного stack. Это временная политика, не доказанная
  эквивалентность всем веткам Minecraft codec/recovery. Общий Horizon decoder
  сам clamp-ит нулевые counts; adapter проверяет их до вызова decoder.
  Java cell oracles используют stub upgrades и не доказывают этот новый reader;
  здесь доказательства — сверка pinned source, Rust/native tests и world retention.

- AEBasePoweredItem energy methods сверены с unchanged original class:
  2000 traces/400000 operations, exact double bits (NaN payloads эквивалентны),
  включая simulate/commit, overflow, threshold, capacity override/multiplier/reset,
  overcapacity, negative/nonfinite requests и signed zero. Компонентный map,
  Minecraft item/UI infrastructure — stubs; полный component codec этим не доказан.
- Charger.doWork сверён отдельно: дословный метод из pinned source компилируется
  в stub host вместе с unchanged AEBasePoweredItem, StoredEnergyAmount и
  PowerMultiplier; 2000 traces/200000 calls, exact doubles/working/request order.
  Это не весь block entity: grid dispatch, inventory/config/world lifecycle — stubs,
  recipe lookup всегда возвращает null. Рецепт/RNG этим oracle не проверяется.
- Native powered adapter использует defaults AEConfig и charge factor/power
  multiplier 1; rule core принимает параметры. Live config supplier/reload ещё нет.
  Чтение energy cards меняет rate, но не пересчитывает capacity при загрузке:
  исходный onUpgradesChanged делает это только при изменении инвентаря. Writer/
  callbacks и capacity expansion через редактирование карт ещё не подключены.
- Шесть поддержанных storage/power component IDs добавлены в command registries
  с сохранением vanilla IDs. Два energy codecs проверяют numeric NBT, как
  Codec.DOUBLE; generic container/key/fuzzy codecs и packet registrations ещё
  частичные. Arbitrary addon components не объявляются известными автоматически.

- Item energy bar: исходные 13 pixels, always visible и constant green; Java
  Math.round -> long -> int narrowing -> clamp порядок сохранён, включая NaN и
  большие values. Отдельные 400000 power/bar operations сверены с unchanged
  AEBasePoweredItem (tools/item_energy_oracle.py --bars), exact double bits.
  Count/cooldown decorations используют прежний native painter.
- Визуальная проверка выявила отсутствие AE2 components на frontend: исходный
  Horizon network_stack отбрасывал их. ae2_component_sync передаёт полный native
  component map AE2 stacks в versioned CUSTOM_DATA wire envelope и раскрывает
  его перед штатным item_data decoding. Сохранённые/server/frontend ItemStacks
  сохраняют исходные имена/типы; исходный custom_data вложен без конфликтов.
  Это native Horizon transport. Нужен тот же bundle у участников; external
  multiplayer/negotiation и полный addon component API ещё требуют проверки.
  Nested shulker/bundle packets, typed unknowns, collision with user custom_data,
  malformed wrapper rejection и stable/change-sensitive click hashes проверены
  native library tests. Envelope не добавляется к исходным или сохранённым stacks.

- Charger использует исходный `inv.item0` (AEBaseInvBlockEntity) с native Items
  transport mirror. Source-format updates приоритетны перед прежним echo, обычные
  native slot/energy updates сохраняются обратно; slot edits flush сразу.
  Empty slot — `{}`, default count one опускается, working хранится в transient
  data. Общий Horizon decoder clamp-ит nonpositive counts; adapter проверяет count
  раньше. Malformed inv сохраняется и блокирует работу/клики до explicit repair.
  Это более строгая временная политика, чем Minecraft parseOptional recovery;
  полные missing-content/legacy codecs и destruction/explosion/drop paths открыты.
  Формат optional count и empty stack сверялся с
  [Mojang Item Stack Format](https://www.minecraft.net/en-us/article/minecraft-snapshot-24w09a).

- Drive и Charger используют общий inventory_codec для source optional stacks,
  empty `{}`, omitted default count и original/native echo selection. Drive
  больше не делегирует count-zero clamping general decoder и не снимает blocked
  state при изменении одного native echo. Raw malformed original inv сохраняется
  до explicit repair. Прямая source NBT загрузка положительного count>1 сохраняет
  count: AEBaseInvBlockEntity вызывает setItemDirect без ограничения входного
  слота; один slot даёт один storage handler, не ёмкость/idle вдвое. Нормальная
  insertion через menu/hopper по-прежнему имеет limit one. Полная registry-aware
  codec normalization/missing-content recovery и все legacy варианты ещё нужны.
