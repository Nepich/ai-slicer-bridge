# Фактическая проверка — 13 ноября 2026

- Windows x64: Rust 1.100.0 GNU, LLVM-MinGW 20261104 MSVCRT, инструменты в E:\codex\tools.
- `cargo test`: 8 успешных тестов политики, 0 ошибок.
- Release exe собран. Размер около 2 МБ.
- MCP smoke: initialize, tools/list, отклонение ввода без наблюдения, неизвестный метод, некорректный JSON, восстановление и ping. Скрипт `tests-mcp.ps1` не выполняет UI-ввод.
- Проверка существующего окна через доступный инструмент рабочего стола: Bambu Studio обнаружен, доступны снимок и частичное дерево элементов. Окно перекрыто другим приложением. Это проверка доступности окружения, **не** проверка Win32-backend созданного агента.
- Открытого OrcaSlicer не обнаружено. Установка OrcaSlicer не выполнялась.
- GUI импорт/поворот/PLA/поддержки/нарезка и печать созданным агентом не прогонялись. Перед практическим использованием требуется ручной acceptance для обеих установленных версий.
- Подключение exe к MCP-клиенту ещё не настроено; файл конфигурации является примером.
- `cargo fmt`, `cargo clippy -- -D warnings`: успешно; окончательные cargo test, release build и tests-mcp.ps1 повторены после последних исправлений.


## v0.2 — live workspace

- Windows UIA observation/action path contains no capture call; default MCP schema has three tools, and screenshot tools are disabled without explicit --allow-screenshots.
- Native side-by-side panel, per-action approval, explicit local full-session trust including print/upload/delete, STOP/revoke. Full trust is not persisted or exposed as an MCP grant tool.
- 14 tests passed, including five no-image UI-state tests and local trust scope/revocation. cargo fmt, clippy with denied warnings, release build and no-GUI MCP smoke passed.
- Native macOS/Linux backends remain unimplemented. CI tests shared core on those runners, not working native GUI agents.
- Live panel/real UIA backend acceptance results are recorded separately below; no model edits or printing are implied by automated results.
- Live Windows smoke: the compiled v0.2 agent read 124 UIA controls from an existing Bambu Studio window with truncated=false and no image payload. The panel opened, selected Bambu and arranged it alongside; its accessibility status reported Ready for MCP commands. Closing the panel ended the process with code 0. Full trust was not granted; no model edit, slicing or print was performed.
