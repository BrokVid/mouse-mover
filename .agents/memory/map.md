| Путь | Зачем |
|---|---|
| `src/main.rs` | Win32 окно, трей (ЛКМ открыть, ПКМ меню), джиггл |
| `src/config.rs` | настройки и джиттер таймера |
| `assets/icon.ico` | иконка приложения |
| `build.rs` | вшивка иконки и версии |
| `msix/AppxManifest.xml` | шаблон Store-идентичности и Win32 MSIX-манифеста |
| `scripts/pack-msix.ps1` | сборка x86/x64 MSIX bundle и Partner Center upload |
| `.github/workflows/build.yml` | CI: PE и Store MSIX артефакты |
| `.cargo/config.toml` | `crt-static` для x64 и i686 |
| `Cargo.toml` | зависимости и release LTO/opt-z |
