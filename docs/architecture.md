# Архитектура

MCP stdio → Bridge → Adapter (Bambu / Orca) → Desktop → Win32.

`lib.rs`: типизированные действия, состояния и политика. `windows.rs`: перечисление окон, проверка процесса/переднего плана, GDI → PNG, Win32 ввод, локальный MessageBox. `main.rs`: JSON-RPC/MCP, текстовые метаданные и image content.

Никакого сетевого слушателя, shell-инструмента, запуска произвольной программы, чтения конфигураций/токенов аккаунта или скрытой отправки на принтер. Оба адаптера используют общий Desktop. Интерфейс Desktop позволяет тестировать отказы без реального ввода.

Состояния: Нет наблюдения → Наблюдение → Расход наблюдения → Валидация действия → Повторная съёмка → Локальное разрешение → Повторная съёмка → Ввод → Снимок результата. Любая ошибка требует нового observe. Отпечаток связывает пиксели, handle, PID, заголовок и размеры. Не учитывает невидимое состояние и перемещение окна при неизменном изображении.

Все мутации подтверждаются, потому что произвольный клик/Enter может быть финальной отправкой на принтер. Запрет только на инструмент с названием print не защищает generic click. AI не получает способ согласиться с локальным MessageBox через этот агент: оно не принадлежит процессу слайсера.

Нельзя доверять тексту снимков, названиям объектов и полю purpose. В диалоге purpose обозначено как недоверенный текст; точное действие и процесс показываются отдельно. Для дальнейших версий: UIA-first селекторы с проверкой версии, Windows.Graphics.Capture, высокоуровневые операции с постусловиями и нативная панель плана задания/локального подтверждения печати.

MCP: минимальные initialize, ping, tools/list, tools/call, notifications без ответа; версия 2024-11-05. Нет HTTP/OAuth, ресурсов, prompts, subscriptions, cancellation. Запросы обрабатываются последовательно. Ожидание локального диалога блокирует обработку следующих запросов. Максимальная длина строки проверяется после чтения; сервер не предназначен для недоверенных клиентов.


## v0.2 changes (supersedes v0.1 always-confirm and screenshot-only descriptions)

UIA observer returns accessible controls and bounds without pixels. UiBridge consumes typed state and validates it before and after authorization; only observed element clicks, allowed keys and focused edit-field text are dispatched. Custom canvas regions may be inaccessible.

The native panel opens/arranges the actual slicer alongside. Human may explicitly grant current-session FULL TRUST to the selected PID, including print/upload/delete. Per-action confirmations remain default; there is no MCP trust-grant tool. Stop/close/reselect/resume revokes. An authorization epoch invalidates outstanding permits on trust changes. Already-dispatched input cannot be retracted. Native input still checks foreground ownership and held modifier keys.

Default server exposes three no-image tools. Optional legacy capture tools require --allow-screenshots. UIA provider calls may block; the panel runs on its own thread. Stop prevents later input but does not forcibly cancel an external UIA provider call.
