# Mouse Mover

Портативный джигглер мыши для Windows 10/11. Один exe, без .NET, ~330 КБ.

Не даёт экрану блокироваться по простою.

## Скачать

- [mouse-mover-x64.exe](https://github.com/BrokVid/mouse-mover/releases/latest/download/mouse-mover-x64.exe) — 64-бит
- [mouse-mover-x86.exe](https://github.com/BrokVid/mouse-mover/releases/latest/download/mouse-mover-x86.exe) — 32-бит (на 64-бит тоже работает)

Настройки: `%APPDATA%\MouseMover\config.json`

Закрытие окна сворачивает в трей. Выход — из меню иконки.

## Сборка

Rust (MSVC) + Windows SDK:

```powershell
cargo build --release
cargo build --release --target i686-pc-windows-msvc
```

## Лицензия

[MIT](LICENSE)
