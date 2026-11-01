# Текущая точка полного переноса

Цель активна: полный перенос AE2 19.2.18 на Horizon, 1:1, без субагентов.
Пользователь возобновил goal; текущая работа — CableBus/кабели после EnergyCellBlockItem.
Цель не уменьшена до текущего набора механик. Все 1429 исходных Java-файлов
остаются в реестре; сейчас 96 partial, 1333 pending, полностью закрытых нет.

## Последний результат

- CableBus saved-host adapter: original cable.id/gn + six side/facade keys,
  buffered energy/owner persistence и raw unknown data retention. Source
  node-data equality сверена с Java на 20064 случаях. Owner-only edits не
  сбрасывают node; external item/other gn changes сбрасывают, собственный save
  обновляет observed snapshot; live mismatch блокирует stale overwrite.
  Unsupported side parts не активируют host до переноса их собственных nodes.
  Physical registration/placement/render и full registry/part lifecycle открыты;
  source explicit-load events ещё нужно отделить от polling/own-save echoes.
  120 core / 172 native game / 11 Python и native check/build PASS; реальная
  typed NBT запись/чтение и связка gn buffers с graph identity проверены.
  Final built-client power regression power-_xs22713 — четыре сценария PASS,
  включая exact save/restart; это не placed CableBus test.

- CableBus/cable core: все 85 цветных item profiles; source type/size/variant
  ordering, 17 RGB/tint palettes, flags и connection lengths; raw geometry
  для glass/covered/smart/dense, visual channel scaling и cable suffix packets.
  Lazy center/side/facade storage воспроизводит различие setPart(null) и
  setFacade(null). Java oracle — 278956 случаев PASS, из них 216608 shapes,
  20000 visual counts, 20000 payloads и 20000 storage operations. Серверный
  binding поддерживает цвета, exposed faces и сохранение node identity при
  перекраске; dense trunk обслуживает 12 Drive profiles, normal bottleneck
  ограничен восемью каналами. 119 core / 163 native game / 11 Python,
  native check/build и 14-check Controller world regression PASS. Это
  core/native profiles, не размещённые кабели.
  Native saved-host decoder добавлен следующим слоем; physical placement,
  per-part lifecycle, full NBT registry codecs, renderer/facades/client sync остаются.

- EnergyCellBlockItem: исходные fixed capacities/rates, отдельный 0.00001
  threshold, inject/extract/simulate/overflow; Charger поддерживает ещё два
  block items, игнорируя чужие capacity/upgrades. Item models — source float
  fifths, без powered-item bar и persistent render projection; NaN/±Infinity
  не теряются в JSON bridge. Player placement/first matching loot stack
  переносят stored_energy без proxy.e; live snapshot guard сохраняет same-tick
  расход и не затирает новую external NBT; actual placement создаёт новый node
  identity даже при совпадении position/kind/saved value. 111 core / 156 native game / 14 native
  library / 7 assets / 11 Python PASS; new Java 100000 operations и прежний
  base-powered 400000 operations PASS. World energy-cell-item-z5e22fr2 —
  13 checks PASS: оба Charger -> survival placement/drop, zero/tiny/cap clamp,
  level destroy, count-two, десять моделей без mutation и exact restart/drop.
  Tooltip/base settings/capabilities и все loot/unloaded/explosion paths остаются.

- Energy/Dense Cell: исходные пять fullness states, original models и double
  comparator только по local stored power. Creative Cell — одно состояние без
  analog output; dispenser/facing/triggered behaviour убрано у всех трёх cells.
  Отложенные notifications/pending NBT учитывают block-ticking chunks; callbacks
  выполняются после освобождения NETWORKS. Initial load воспроизводит source
  MIN_AMOUNT. 108 core / 151 native game / 13 native library / 11 Python PASS;
  source visual oracle — 100000 cases / 200000 comparisons PASS. World
  energy-cell-display-6loo8yw4 — 40 checks PASS, включая real comparator/dust,
  same-fullness analog changes, Controller drain и exact restart.
  Charging item-charging-474fwc4q regression PASS. Полный TickManager/lifecycle,
  item/drop energy components, item models/tooltip и capabilities ещё нужны.

- Controller world backend и real state registry: 8000 public AE/default storage
  priority 0, idle 3, CANNOT_CARRY/DENSE_CAPACITY, controller positions в настоящем
  Graph/pathing и original NBT. 18 state/type variants, source column/interior/
  parity и powered-before-conflict. 106 core / 148 native game / 12 native library /
  11 Python PASS. Source visual oracle — 4000 cases/8000 comparisons PASS (ready/
  loaded stub host, не Java world). Library проверяет vanilla IDs и чужие door/
  button/shape/model references после расширения states. World controller-pz6q6dun
  PASS: ровно 3 AE/tick, реальный Charger, все render types, width 8 conflict ->
  7 recovery, indirect-controller conflict и exact restart ledger. Charging
  item-charging-8r8yerow и Drive GUI drive-gui-nrm5z8ej regressions PASS.
  Network Status Menu, external power/API и полный event/readiness/chunk/spatial
  lifecycle/material parity остаются открытыми. Connected model screenshot просмотрен.

- Comparator Drive/Charger: source slot limit 1, десять/один реальные slots,
  без transport padding; original inv precedence/strict malformed gate и air
  emptiness. 101 core / 143 native / 11 Python PASS, native check/build PASS.
  Minecraft 1.21.1 official bytecode inspection: ContainerAdapter contract,
  AbstractContainerMenu/Mth arithmetic и ItemStack.isEmpty. World
  inventory-signal-0l2hpc17 PASS: реальные comparator/dust, все 11 уровней Drive,
  Charger raw ingredient/powered, direct count-two, malformed gate, notifications
  от hopper/manual/Drive GUI, vanilla barrel и restart. Other hosts, source
  recovery/world oracle и redstone propagation >15 ещё открыты.

- Исходные max-stack properties: все 58 unstackable registrations имеют лимит 1,
  остальные 217 carried item definitions — 64. Подготовка контента не наследует
  случайный лимит template item. Pinned source audit включает фабрики AEItems,
  encoded-pattern builder, WrappedGenericStack и vanilla TieredItem inheritance.
  Native world item-stacks-oad53z4_ PASS: все 58 через реальные hoppers + 3
  stackable controls, full refusal, component retention/restart и player give split.
  Charger regression rn68d0mn и Drive GUI xrbuwn6s PASS. 98 core / 11 Python,
  fmt/clippy/audits/native check и content hmod build/check PASS. Rust payload
  не менялся; прежние native 135/library 9 tests повторно не запускались.
  Остальные item properties/actions и component-specific overrides ещё открыты.

- Charger automation (`322cc27`): hopper/dropper на всех шести гранях,
  исходный ChargerInvFilter, вставка по одному, сохранение компонентов и
  остатка source-loaded stack, rollback неудачного извлечения и немедленный
  flush inv.item0. Повреждённый original остаётся blocked до explicit repair,
  в том числе после restart. Все 16 powered items и реальные recipe tags
  участвуют в фильтре; unsupported result components не меняют membership.
  98 core / 135 native / 10 Python PASS. Контрольная точка, игровые проверки
  и ограничения зафиксированы в VALIDATION.md. General capability API остаётся
  открытым; default stack limits и Drive/Charger comparator закрыты следующими этапами.

- Текущий этап shared source inventory codec завершён и проверен: 95 core,
  130 native, 10 Python, fmt/clippy/audit и native check/build PASS. World Drive
  target/smoke/drive-inventory-hkskab5i: zero-count refusal, blocked echo retention,
  restart/repair, direct count-two and exact drain PASS. Charger regression
  target/smoke/charger-inventory-3zuq22ud и GUI target/smoke/drive-gui-ife9_u0f PASS.
  Пользователь попросил паузу после этого этапа для совместного тестирования.

- Charger original inv.item0 load/save + native echo, direct actor flush,
  count-one omission, transient working, strict malformed retention/recovery gate.
  95 core / 127 native / 10 Python + native check/build PASS. World
  target/smoke/charger-inventory-jb66r9mi and full charging regression
  target/smoke/item-charging-zoi_dbzl PASS: source format, local energy conservation,
  stale echo, original replacement, zero-count refusal, legacy migration,
  restart/retrieval and source recipe output. Full recovery/capabilities/drop paths open.

- Native AE2 component packet bridge сохраняет полную typed component map,
  original custom_data, nested shulker/bundle и click hashes. Original network
  projection отбрасывала mod fields; обнаружено по black energy bars на клиенте.
  Source energy bars теперь 0/7/13 pixels, green; pixel QA и actual componentful
  menu roundtrip PASS в target/smoke/item-bars-6uh8fmi6. Drive GUI regression
  target/smoke/drive-gui-rf9_6dl8 PASS. Full charging regression/restart PASS:
  target/smoke/item-charging-5lt44fkz. 95 core / 123 binary / 9 library tests PASS.
  Native check/build PASS. Java --bars 400000 operations PASS; базовый energy-only
  generator input hash сохранён. Tooltip/creative variants/reequip ещё нужны.

- Powered items подключены к Charger: all 16 IDs, source components/rates,
  local/direct-grid/overflow/refill, шесть command component registrations.
  94 core / 121 native / 10 Python PASS, native check/build PASS.
  AEBasePoweredItem oracle: 400000 operations; дословный Charger.doWork
  в stub host: 200000 calls, exact doubles — PASS. World
  target/smoke/item-charging-vjhsnfav: conservation, all 16 insert/charge,
  Crank/front restriction, partial energy + contents restart/retrieval PASS.
  Это не handheld actions и не NeoForge/world oracle. Defaults config пока fixed;
  actual config reload, tooltip/bar/creative tab и upgrade writer/callbacks открыты.

- Исправлен оригинальный upgrade component reader: `slot/item`, разные
  physical slot limits шести storage host families и совместимость карт из
  InitUpgrades. Installed counts считаются по слотам и clamp-ятся; raw Energy
  Card iteration отдельно. Старые плоские тестовые fixtures были неверными.
  Nonpositive count проверяется до Horizon decoder (тот clamp-ит к одному).
  Duplicate/malformed components пока строго отклоняются с сохранением stack.
  Writer, Workbench, listeners/cache invalidation и addon registry остаются.
  87 core / 110 native / 10 Python PASS; check/build PASS. Новый мир
  target/smoke/drive-upgrades-o7q_ih7r подтвердил canonical component retention,
  исходные contents/drain и restart; world terminal operations не покрыты.

- Native component-aware item/fluid keys, GenericStack и codec компонентов ячеек
  хранения; прежний Java pathing oracle: 2000/2000 совпадений.
- EnergyService / GridEnergyStorage / EnergyOverlay: provider priorities,
  per-node buffer, power warm-up, cached/average power, storage events и overlay.
  2000 Java traces / 462175 операций PASS; исправлены scoped overlay
  invalidation и снятие active generator при удалении узла.
- World network adapter: Charger, normal/dense/creative energy cell и Drive,
  соединения по граням, merge/split, перенос per-node buffer, сохранение NBT.
  Charger исключает front, получает до 800 AE за doWork из общего сервиса.
- Реальный игровой smoke PASS: finite cell 2000 AE → рецепт → остаток 400 AE;
  передняя грань питание не проводит; creative питает без crank; restart
  сохраняет finite и private buffer без восстановления потраченной энергии.
- StorageService: provider identity/refresh/unmount, lazy cache/private baseline,
  key/watch-all interests и permanent watcher destroy, 6 новых core tests.
- Drive: десять slots для всех 23 built-in cell handlers, 2 AE + disk idle draw, front exclusion,
  REQUIRE_CHANNEL, priority/online mounts, packed state, original inv.itemN и
  Items mirror. External capabilities и dynamic chassis/LEDs ещё открыты.
- Drive GUI: original PNG/layout, десять visible slots, filter + cap1,
  authoritative pickup/shift/drag/hotbar policy, main-before-hotbar order,
  изоляция от vanilla menu. Game smoke и restart PASS:
  target/smoke/drive-gui-4o68o5bf. Priority UI, ghost icons и bulk actions pending.
- Drive automation: реальные hoppers/droppers, all six insertion faces,
  filter/cap1, offline, failed pull + component retention, full Drive/remainder,
  restart PASS: target/smoke/drive-automation-0pywd8ek. Slot mutation flush и
  container changed provider refresh подключены; general capability API pending.
- Portable backend: половина bytes, original overhead, item limits
  54/45/36/27/18, fluid 18, 0.5 idle drain; native codec/tooltip/mounts.
  World smoke target/smoke/drive-portable-6xwx3vus PASS: exact 3.015625 AE/tick,
  counts 123/1000, front/offline/restart, own stored_energy=1234 сохраняется.
  Handheld UI/charge/use/disassembly и color/LED pending.
- BasicCellInventory original Java oracle: 2000 traces/572000 операций PASS,
  docs/cells-oracle.json; actual key/fuzzy/component/upgrade/world infrastructure stubbed.
- Creative backend: configured exact keys, unlimited insert/extract,
  Integer.MAX_VALUE listing, TYPES_FULL/zero drain/empty nesting/no-op persist.
  2000 original Java traces/572000 операций PASS, native codec + real mounts/
  offline tests PASS. World config/power/restart smoke:
  target/smoke/drive-creative-djhqrho5; portable regression drive-portable-fi8or0cv.
  Config parser теперь соблюдает 63 nullable slots и игнорирует trailing entries.
- Platform c tags: pinned NeoForge 21.1.169 / 863802b, 461 generated files,
  layer/replace, licence/source hashes. Удалены прежние ручные три tag lists.
  Content build/check и 14 positive + 14 negative world predicates/restart PASS:
  target/smoke/platform-tags-kwyjn5mu. Остальные platform APIs/conditions pending.
- Tools: Matter Cannon/Color Applicator storage profiles + restricted admission
  из реальных recipes/tags, nesting filled tools, private-energy retention.
  2000 tool profile Java traces/572000 операций PASS (fake blacklist/descriptor).
  Native filter/codec tests PASS; world power/content/restart:
  target/smoke/drive-tools-yj8oapg3, exact 3.015625 AE/tick. Handheld actions/reload pending.
- 85 core + 105 native binary + 5 native library + 10 Python тестов прошли.
  Native build и Drive GUI smoke прошли; final check evidence — VALIDATION.md
  и target/tool-native-check-final.json.
  Новый power smoke: target/smoke/power-wkijuezf; повторный ручной Charger/Crank
  smoke: target/smoke/charger-1ta7ucsr. Все тестовые процессы завершены.
- NetworkStorage: 2000 Java traces / 410882 операции PASS. Теперь используется
  native Drive через StorageService. Native tests проверяют power loss,
  merge/split, сохранение объектов дисков и снятие mounts со старого сервиса.
- Drive game smoke PASS: target/smoke/drive-5dq1vz2t. Пустой Drive потребляет
  2.015625 AE/tick; с item 1k и fluid 4k — 3.515625 AE/tick. Исходный inv,
  slot 9, отключение front, сохранение количеств 123/1000 и restart проверены.

## Следующий связующий слой

0. Продолжить real world CableBus и storage hosts; для Controller нужны Network
   Status Menu, capabilities и точный lifecycle. Для energy
   cells остаётся полный TickManager/base-settings lifecycle. Component-specific stack overrides
   и остальные item components/actions также остаются в полном объёме.
1. Продолжить source/oracle verification энергосервиса (watchers/addons), закрыть известные
   lifecycle/overlay/events gaps и реальную передачу состояния клиенту.
2. Закрыть priority UI/bulk actions Drive, external capabilities, sync/рендер и все persistence
   paths до терминалов. Перенести ME Chest, terminal UI и CableBus/parts,
   ownership/readiness и полную модель событий/приоритетов GridConnection.
3. Handheld tool charging/use/painting/shooting, admission reload invalidation;
   creative workbench/tooltip/API и portable handheld UI/energy.
4. Для energy cells остаются все saved-BE/explosion/pick/loot-command paths
   и base settings/capabilities; для Charger — полный item API/FX и capability lifecycle; для Crank — ориентация,
   sync/анимация. Остальные задачи полного порта остаются в PLAN/PARITY.

## Локальные команды и данные

`source target/local-env.sh` задаёт инструменты на этой машине. Собственные
pinned checkout: target/upstream/{ae2,horizon,horizon-mixins,neoforge}. Original engine
и horizon-create не менять. Ресурсы читать из HORIZON_RESOURCES, тестовые saves
только под target. Все тестовые процессы последнего smoke завершены.

- tools/check.py: fmt, clippy, 95 core tests, 10 tooling tests, inventory check.
- tools/horizon.py test: 130 native tests (Game + typed NBT); warnings об unused
  API остаются до подключения всех потребителей API, не скрыты.
- tools/horizon.py test --library: 9 tests, в том числе три server-click теста.
- tools/pathing_oracle.py --cases 2000: JDK 21+, default local Homebrew JDK.
- tools/cells_oracle.py --cases 2000: 572000 exact operations, actual Java class.
- tools/cells_oracle.py --creative --cases 2000: ещё 572000 exact operations.
- tools/cells_oracle.py --tools --cases 2000: ещё 572000, restricted finite backend.
- tools/smoke.py --ctl "$HORIZON_CTL" --resources "$HORIZON_RESOURCES": новый
  изолированный мир; последняя успешная папка target/smoke/charger-1ta7ucsr.

## Лимит

Пользователь после сброса quota вновь установил остановку при остатке <=15%.
Проверять доступные окна get_usage_limits примерно раз в пять минут; при пороге
вызвать update_goal(paused) и остановить работу. Не покупать/сбрасывать quota.
Последняя проверка: used 0%, remaining 100%; прежнее отключение порога отменено.

## Следующие работы

- Item energy tooltip и исходные creative/reequip overrides.
- Charger native automation filters и remaining inv/drop lifecycle; исходный
  ChargerRecipes.allowExtract разрешает любой предмет, не являющийся текущим
  recipe input. Не придумывать запрет извлечения всех partially charged tools.
- Workbench/upgrade writer: onUpgradesChanged меняет capacity multiplier
  portable/cannon/color на 1+8×cards, wireless на 1+cards. Не выполнять этот
  callback просто при чтении сохранённого item.
- Live config (батареи/rates/power multipliers), common capability API.
Цель остаётся полным 1:1 переносом, а не текущим playable subset.

Resolved audit: Drive now uses inventory_codec count prevalidation and blocked
source selection. Count zero cannot be clamped into a live cell, and native echo
changes do not clear a blocked original. All three world regressions passed.
