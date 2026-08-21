## 2026-08-21 — стек Rust + Win32, два PE

Контекст: Mouse Jiggler 137 МБ из-за self-contained .NET 10. Нужен один лёгкий exe на Win10/11, в том числе старые сборки, 32 и 64 бит.

Решение: Rust 1.97 / edition 2024, crate `windows` 0.62.2, GUI на чистом Win32, `+crt-static`. Два бинарника: x64 и i686. Fat-binary у Windows нет; i686 работает на 32-битном Win10 и на 64-битном через WoW64.

Почему не Go: размер, GC, ложные срабатывания AV на unsigned Go. Почему не .NET: исходная проблема.

Случайный интервал по умолчанию (50–100% периода + 0–999 мс) — как у Mouse Jiggler, не стелс от EDR. Имя окна честное, иконка в трее видна.

## 2026-08-21 — MSIX для Microsoft Store

Пакет связан с записью Partner Center через `SFNVX.MouseMoverr` и Publisher
`CN=D68EAD28-BEC2-4B13-B878-F1F336C12B72`; зарезервированное отображаемое имя —
`Mouse Moverr`. CI собирает x86 и x64 MSIX, bundle и `.msixupload` с
PDB-символами. В репозитории не хранится сертификат: артефакт предназначен для
Partner Center, который подписывает Store-распространение.
