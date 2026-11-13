# Исследование — 13 ноября 2026

Первичные источники:

- Bambu Studio CLI: https://github.com/bambulab/BambuStudio/wiki/Command-Line-Usage — загрузка проектов, профилей и нарезка; не интерфейс управления открытым GUI.
- Bambu Studio исходники: https://github.com/bambulab/BambuStudio — наличие network plugin не доказывает существование публичного универсального plugin API для сторонних расширений UI. Такой контракт для этого проекта не подтверждён.
- Orca CLI: https://github.com/OrcaSlicer/OrcaSlicer/wiki/cli_mode — headless автоматизация, не замена UI-управлению.
- Orca Python plugins: https://github.com/OrcaSlicer/OrcaSlicer/wiki/plugins_getting_started — по текущей документации nightly или releases > 2.4.2.
- Orca plugin development: https://github.com/OrcaSlicer/OrcaSlicer/wiki/plugin_development — embedded CPython и отдельные capability contracts; это не гарантирует доступ ко всем GUI операциям и не обеспечивает общую поддержку Bambu Studio.
- Rust: https://rust-lang.org/tools/install/ — официальный rustup; выбран GNU toolchain + LLVM-MinGW linker (Zig проверен, но не используется в итоговой сборке), чтобы не ставить MSVC/Windows SDK на системный диск.

Решение: внешний Rust Windows UI-агент для обоих приложений. CLI остаётся возможным будущим вспомогательным backend, но не основным путём. Плагин Orca можно добавить позже как опциональную возможность после проверки конкретной установленной версии.


