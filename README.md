# Mouse Mover

Портативный джигглер мыши для Windows 10/11. Один exe, без .NET, ~330 КБ. Не даёт экрану уйти в блокировку по простою.

## Скачать

| Файл | Для кого |
|---|---|
| [mouse-mover-x64.exe](https://github.com/BrokVid/mouse-mover/releases/latest/download/mouse-mover-x64.exe) | 64-битные Windows |
| [mouse-mover-x86.exe](https://github.com/BrokVid/mouse-mover/releases/latest/download/mouse-mover-x86.exe) | 32-битные Windows и 64-битные через WoW64 |

Распаковывать не нужно. Настройки: `%APPDATA%\MouseMover\config.json`.

## Возможности

- скрытый режим — система видит движение, курсор на месте
- видимый — курсор рисует квадрат 64×64 px
- интервал в секундах, применяется сразу
- случайный разброс ±1–90% от интервала
- трей: ЛКМ открывает окно, ПКМ — открыть / выход
- сворачивание при запуске + уведомление (клик по нему открывает окно)

Закрытие окна сворачивает в трей. Выход только из меню иконки.

## Сборка

Нужны Rust (MSVC) и Windows SDK.

```powershell
cargo test
cargo build --release
cargo build --release --target i686-pc-windows-msvc
```

CRT статически (`+crt-static` в `.cargo/config.toml`). VC++ Redistributable не нужен.

## Подпись

Сборки пока **без Authenticode**. Для OSS можно запросить бесплатную подпись в [SignPath Foundation](https://signpath.org/apply) — это не Let's Encrypt: заявка, публичный репозиторий и OSI-лицензия. Одобрение не мгновенное и не гарантировано (у них пункт «не PUP»).

## Лицензия

[MIT](LICENSE)
