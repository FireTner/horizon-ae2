# Полный перенос AE2

Эталон и граница версии зафиксированы в upstream.lock.json. Требование 1:1
сохраняется для всех этапов; временные ограничения не становятся спецификацией.
Нельзя закрывать этап по одному успешному крафту или по количеству скопированных
ресурсов. В docs/upstream-inventory.json перечислены каждый Java-файл, его
SHA-256, подсистема и фактический объём начатого переноса.

## Архитектура

- Правила Rust не зависят от Horizon. Инвентари, энергохранилища, граф сети,
  рецепты, задания автокрафта и доступ к миру разделяются маленькими контрактами.
- Адаптер Horizon отвечает за authoritative server, жизненный цикл измерений,
  чанки, компоненты/NBT, сообщения, ввод и рендер. Состояние не должно смешиваться
  между мирами, измерениями или игроками.
- Данные/модели/текстуры берутся из jar той же версии; carry-отчёт показывает
  ограничения движка, а не степень переноса Java. Исходный baseline сохранён
  в baseline-port-pack.json: 78 блоков + 275 предметов в data carry, 24 блока
  без шаблона, 42 block entity, 39 меню, 55 экранов, 6 типов рецептов требуют кода.
- Сначала сравнение с Java и unit-тестами, затем интеграция и тесты в игре.
  Для доказательства 1:1 нужен Java/NeoForge oracle на закреплённой версии.
  Rule-level Java oracle уже проверяет pathing, энергию/overlay и network storage;
  world/NeoForge oracle ещё нужен, его нельзя заменять названием теста.

## Этапы и критерии завершения

1. **Воспроизводимость и первый игровой срез (начат).** Pin источников/движка,
   перечень всего кода, подготовка контента и native bundle; capacity ячеек,
   energy amount, ручной Charger. Закрытие: launcher install и isolated game
   тесты сохранения/загрузки, слотов, ручного привода; устранённые либо явно
   оставленные незакрытыми API gaps. Это не полноценная ME-система.
2. **Ключи, компоненты и сохранение.** AEKey/AEItemKey/AEFluidKey, GenericStack,
   KeyCounter, сравнение компонентов, partition/fuzzy, вложенные ячейки,
   upgrade inventory, data component codecs и миграции, creative/portable cells,
   Cell Workbench. Проверки: NBT roundtrip, миллионы предметов, сложные компоненты,
   полные ячейки, фильтры, перезагрузка и запрет потери/дублирования.
3. **ME-граф.** Grid/Node/Connection/Storage/Service, сетевые события, цвета,
   CableBus/части/фасады, соединения на границах чанков, merge/split, ad-hoc,
   pathfinding, dense/обычные/умные кабели и каналы, контроллерные структуры,
   security/ownership и сетевые индикации. Сценарии из ChannelTests,
   QnbTestPlots и памяти оригинала; не заменять лимиты общим числом устройств.
4. **Энергия и первые машины.** EnergyService, реальные storage events,
   PowerMultiplier/конфиг, Energy Acceptor, Energy/Dense/Creative Cell,
   Vibration Chamber, Crystal Resonance Generator, Charger tools/network branch,
   Inscriber, upgrades, стороны/инвентари, работа при отключении питания.
   Общий item/fluid/energy capability adapter должен позволять интеграцию с Create.
5. **Хранение и терминалы.** StorageService/mounts/priorities, Drive (10 слотов),
   ME Chest, storage bus, view cells, терминалы предметов/жидкостей, поиск,
   сортировка, отрисовка, тени/ghost slots, ручной/shift/drag ввод, синхронизация
   и отслеживание содержимого. Проверки: одновременные клиенты, смена приоритета,
   отключение/перенос диска, компоненты, отказ сети без потери предметов.
6. **Ввод/вывод и части.** Import/Export Bus, Interface, IO Port, Level Emitter,
   Formation/Annihilation Plane, redstone режимы, fuzzy/crafting/capacity/speed
   cards, P2P для ME/items/fluids/energy/redstone/light и прочих upstream типов,
   subnetworks, quantum bridges, wireless access/terminals/boosters, memory card,
   network tool. Все типы частей из upstream parts входят в объём.
7. **Автокрафт 1:1.** Crafting/processing/smithing/stonecutting patterns,
   Pattern Encoding Terminal, substitutions, fuzzy recipes, планирование/циклы/
   остатки/недостаток ресурсов, Crafting CPU multiblock и storage/co-processors,
   Pattern Provider, Molecular Assembler, ожидание processing outputs, monitor,
   отмена/перезапуск/перезагрузка. Сравнение с AutoCraftingTestPlots и
   InvalidPatternTestPlot, включая failed jobs, без выдачи ресурсов из ничего.
8. **Мир и остальные механики.** Метеориты/компас/sky stone/прессы, кварцевые
   почки и их деградация, рост кристаллов/ускорители, fluix-преобразования,
   decorative sets, Sky Stone Tank, Condenser/matter balls/singularities,
   Tiny TNT, entropy manipulator, staff/cannon/color applicator, остальные
   инструменты, предметы, loot/advancements/теги/рецепты и команды оригинала.
9. **Spatial.** Pylon multiblock, spatial cells/IO, anchors, region swaps,
   entities, block entities и жидкости, spatial dimension, загрузка чанков,
   сохранение и безопасный отказ. Нужны механизмы движка, не визуальная имитация.
10. **Клиент и внешняя совместимость.** Все GUI/tooltip/HUD, dynamic models,
    анимации, sounds/particles, guidebook/GuideME, hotkeys, recipe viewer hooks,
    конфиг/локализации/accessibility, multi-player authority/permissions и
    integration-пакеты оригинала. Для Java API — эквивалентный native контракт,
    JVM-бинарная совместимость не предполагается.
11. **Приёмка 1:1.** Source inventory без необъяснённых pending, oracle traces
    по подсистемам, survival progression от метеорита до автокрафта/spatial,
    локальная игра и dedicated/LAN с минимум двумя клиентами, restart/chunk
    unload/dimension travel, upgrade/migration старых тестовых миров, совместный
    запуск с horizon-create, perf/large networks, воспроизводимая сборка и установка.

AEKey/GenericStack/component persistence и GridNode/Connection/Pathing уже
начаты: native codec проверен в Horizon, pathing сверён на 2000 входах с
оригинальным Java. Энергетический сервис и первый world adapter уже подключают Charger и три
energy cells; finite/creative power и restart проверены в игре.
StorageService и Drive basic-cell backend уже подключены: online/priority mounts,
idle/channel costs и исходный inv persistence проверены, в том числе restart.
Drive GUI уже показывает original layout и десять filtered cell slots; серверные
pickup/drag/hotbar/shift правила и игровой restart проверены. Следующий слой —
priority UI, special bulk actions, внешний capability API и полный lifecycle сохранения,
ME Chest, затем terminal UI; CableBus и полные ownership/events остаются открытыми.
Native hoppers/droppers уже соблюдают cell filters/лимит 1 и pre-extraction flush;
all-face, failed transfer, full Drive и restart smoke прошли.
Portable item/fluid cells уже используют отдельные half-byte/type/drain профили
в Drive, исходные компоненты и capacity tooltip. Original BasicCellInventory
oracle: 2000 traces/572000 операций PASS; handheld UI/charge/use/disassembly
остаются открытыми. Creative backend уже подключён к Drive, сверён с original
Java и проверен native integration/world persistence. Следующие storage backends
— Matter Cannon и Color Applicator с их source admission filters.
Для их recipes/tag predicates уже восстановлены исходные c tags зависимости
NeoForge 21.1.169. 461 файла с provenance/licence, overlay/replace тестами и
world item-predicate/restart smoke; остальные API/conditions платформы ещё нужны.
Рендер и сетевое
состояние Charger/Crank остаются открытыми задачами первого этапа.
