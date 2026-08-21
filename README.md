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

### Microsoft Store (MSIX)

`main`, pull request, теги `v*` и ручной запуск workflow собирают x86/x64
`msixbundle` и `msixupload` для Partner Center. Идентификаторы пакета уже
привязаны к продукту Store `9NWHTNQH7ZJ4`.

Для локальной сборки нужен Windows 10/11 SDK с `MakeAppx.exe`:

```powershell
cargo build --release
cargo build --release --target i686-pc-windows-msvc
.\scripts\pack-msix.ps1
```

Результат: `msix-out\MouseMover_<version>_x86_x64.msixupload`. Это
неподписанный файл именно для отправки в Partner Center; Store подписывает
пакет для распространения. Для sideloading потребуется сертификат с Publisher
`CN=D68EAD28-BEC2-4B13-B878-F1F336C12B72`.

## Лицензия

[MIT](LICENSE)
