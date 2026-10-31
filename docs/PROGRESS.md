# Текущая точка полного переноса

Цель активна: полностью перенести AE2 19.2.18 на Horizon, 1:1, без субагентов.
Цель не уменьшена до текущего набора механик. Все 1429 исходных Java-файлов
остаются в реестре; сейчас 40 partial, 1389 pending, полностью закрытых нет.

## Последний результат

- Native component-aware item/fluid keys, GenericStack и codec компонентов ячеек
  хранения; прежний Java pathing oracle: 2000/2000 совпадений.
- EnergyService / GridEnergyStorage / EnergyOverlay: provider priorities,
  per-node buffer, power warm-up, cached/average power, storage events и overlay.
  2000 Java traces / 462175 операций PASS; исправлены scoped overlay
  invalidation и снятие active generator при удалении узла.
- Первый world network adapter: Charger и normal/dense/creative energy cell,
  соединения по граням, merge/split, перенос per-node buffer, сохранение NBT.
  Charger исключает front, получает до 800 AE за doWork из общего сервиса.
- Реальный игровой smoke PASS: finite cell 2000 AE → рецепт → остаток 400 AE;
  передняя грань питание не проводит; creative питает без crank; restart
  сохраняет finite и private buffer без восстановления потраченной энергии.
- 63 core + 64 native + 8 Python тестов прошли. Native check/build прошли.
  Новый power smoke: target/smoke/power-6r_xgmd1; повторный ручной Charger/Crank
  smoke: target/smoke/charger-1ta7ucsr. Все тестовые процессы завершены.
- NetworkStorage priorities/preferred passes, queues, recursion и weak links:
  6 новых core tests, 2000 Java traces / 410882 операции PASS. Это библиотека,
  ещё не world Drive/ME Chest. Native manifest намеренно её не подключает.

## Следующий связующий слой

1. Продолжить source/oracle verification энергосервиса (watchers/addons), закрыть известные
   lifecycle/overlay/events gaps и реальную передачу состояния клиенту.
2. Подключить Drive/ME Chest к storage mounts, затем terminal UI. Не превращать
   их в обычный chest с визуальным названием ME. Перенести CableBus/parts,
   ownership/readiness и полную модель событий/приоритетов GridConnection.
3. Для energy cells остаются fullness/sync/comparator, item/drop/placement
   components; для Charger — tools/FX/inventory filter; для Crank — ориентация,
   sync/анимация. Остальные задачи полного порта остаются в PLAN/PARITY.

## Локальные команды и данные

`source target/local-env.sh` задаёт инструменты на этой машине. Собственные
pinned checkout: target/upstream/{ae2,horizon,horizon-mixins}. Original engine
и horizon-create не менять. Ресурсы читать из HORIZON_RESOURCES, тестовые saves
только под target. Все тестовые процессы последнего smoke завершены.

- tools/check.py: fmt, clippy, 63 core tests, 8 tooling tests, inventory check.
- tools/horizon.py test: 64 native tests (Game + typed NBT); warnings об unused
  API остаются до подключения всех потребителей API, не скрыты.
- tools/pathing_oracle.py --cases 2000: JDK 21+, default local Homebrew JDK.
- tools/smoke.py --ctl "$HORIZON_CTL" --resources "$HORIZON_RESOURCES": новый
  изолированный мир; последняя успешная папка target/smoke/charger-1ta7ucsr.

## Условие паузы по лимиту

Пользователь явно велел проверять account limits примерно раз в пять минут
и поставить goal на паузу при остатке <=10% в релевантном окне. Использовать
Codex get_usage_limits, не token counter goal. Последний подтверждённый остаток
38% (62% consumed), проверка 1 ноября 2026. Не использовать reset credit и не покупать quota.
При пороге зафиксировать текущую точку, вызвать update_goal(paused), сообщить
возвращённый статус и прекратить работу. Сейчас условие не наступило.
