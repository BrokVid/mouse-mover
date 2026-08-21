Перед правкой читай `.agents/memory/`.

Сборка:

```powershell
cargo test
cargo build --release
cargo build --release --target i686-pc-windows-msvc
```

Артефакты: `target/release/mouse-mover.exe` (x64) и `target/i686-pc-windows-msvc/release/mouse-mover.exe` (x86, идёт и на 64-бит через WoW64). CRT статически (`+crt-static`). Конфиг: `%APPDATA%\MouseMover\config.json`.
