| Путь | Зачем |
|---|---|
| `src/main.rs` | Win32 окно, трей (ЛКМ открыть, ПКМ меню), джиггл |
| `src/config.rs` | настройки и джиттер таймера |
| `assets/icon.ico` | иконка приложения |
| `build.rs` | вшивка иконки и версии |
| `.cargo/config.toml` | `crt-static` для x64 и i686 |
| `Cargo.toml` | зависимости и release LTO/opt-z |
