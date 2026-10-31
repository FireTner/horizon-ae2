# Текущая точка полного переноса

Цель активна: полностью перенести AE2 19.2.18 на Horizon, 1:1, без субагентов.
Цель не уменьшена до текущего набора механик. Все 1429 исходных Java-файлов
остаются в реестре; сейчас 57 partial, 1372 pending, полностью закрытых нет.

## Последний результат

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
- Drive: десять slots для 21 basic/portable/creative cell types, 2 AE + disk idle draw, front exclusion,
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
- 81 core + 97 native binary + 5 native library + 8 Python тестов прошли.
  Native build и Drive GUI smoke прошли; final check evidence — VALIDATION.md
  и target/creative-native-check-final.json.
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
3. Продолжить tool backends: Matter Cannon и Color Applicator admission filters;
   creative workbench/tooltip/API и portable handheld UI/energy.
4. Для energy cells остаются fullness/sync/comparator, item/drop/placement
   components; для Charger — tools/FX/inventory filter; для Crank — ориентация,
   sync/анимация. Остальные задачи полного порта остаются в PLAN/PARITY.

## Локальные команды и данные

`source target/local-env.sh` задаёт инструменты на этой машине. Собственные
pinned checkout: target/upstream/{ae2,horizon,horizon-mixins}. Original engine
и horizon-create не менять. Ресурсы читать из HORIZON_RESOURCES, тестовые saves
только под target. Все тестовые процессы последнего smoke завершены.

- tools/check.py: fmt, clippy, 81 core tests, 8 tooling tests, inventory check.
- tools/horizon.py test: 97 native tests (Game + typed NBT); warnings об unused
  API остаются до подключения всех потребителей API, не скрыты.
- tools/horizon.py test --library: 5 tests, в том числе три server-click теста.
- tools/pathing_oracle.py --cases 2000: JDK 21+, default local Homebrew JDK.
- tools/cells_oracle.py --cases 2000: 572000 exact operations, actual Java class.
- tools/cells_oracle.py --creative --cases 2000: ещё 572000 exact operations.
- tools/smoke.py --ctl "$HORIZON_CTL" --resources "$HORIZON_RESOURCES": новый
  изолированный мир; последняя успешная папка target/smoke/charger-1ta7ucsr.

## Условие паузы по лимиту

Пользователь явно велел проверять account limits примерно раз в пять минут
и поставить goal на паузу при остатке <=10% в релевантном окне. Использовать
Codex get_usage_limits, не token counter goal. Последний подтверждённый остаток
19% (81% consumed), проверка 1 ноября 2026. Не использовать reset credit и не покупать quota.
При пороге зафиксировать текущую точку, вызвать update_goal(paused), сообщить
возвращённый статус и прекратить работу. Сейчас условие не наступило.
