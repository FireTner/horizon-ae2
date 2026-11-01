# Текущая точка полного переноса

Цель: полностью перенести AE2 19.2.18 на Horizon, 1:1, без субагентов.
После завершения этапа Charger automation пользователь запросил паузу для тестирования.
Цель не уменьшена до текущего набора механик. Все 1429 исходных Java-файлов
остаются в реестре; сейчас 71 partial, 1358 pending, полностью закрытых нет.

## Последний результат

- Charger automation (`322cc27`): hopper/dropper на всех шести гранях,
  исходный ChargerInvFilter, вставка по одному, сохранение компонентов и
  остатка source-loaded stack, rollback неудачного извлечения и немедленный
  flush inv.item0. Повреждённый original остаётся blocked до explicit repair,
  в том числе после restart. Все 16 powered items и реальные recipe tags
  участвуют в фильтре; unsupported result components не меняют membership.
  98 core / 135 native / 10 Python PASS. Контрольная точка, игровые проверки
  и ограничения зафиксированы в VALIDATION.md. Размер стопок в native item
  catalog, comparator fullness и general capability API остаются открытыми.

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

1. Продолжить source/oracle verification энергосервиса (watchers/addons), закрыть известные
   lifecycle/overlay/events gaps и реальную передачу состояния клиенту.
2. Закрыть priority UI/bulk actions Drive, external capabilities, sync/рендер и все persistence
   paths до терминалов. Перенести ME Chest, terminal UI и CableBus/parts,
   ownership/readiness и полную модель событий/приоритетов GridConnection.
3. Handheld tool charging/use/painting/shooting, admission reload invalidation;
   creative workbench/tooltip/API и portable handheld UI/energy.
4. Для energy cells остаются fullness/sync/comparator, item/drop/placement
   components; для Charger — полный item API/FX, comparator и capability lifecycle; для Crank — ориентация,
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
