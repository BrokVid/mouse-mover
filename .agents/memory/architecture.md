# Mouse Mover

Лёгкая портативная Win32-утилита: не даёт Windows уйти в idle/lock. Один exe, без .NET runtime.

Модули:

- `src/main.rs` — окно «Mouse Mover», трей, таймер, SendInput, SetThreadExecutionState
- `src/config.rs` — JSON в `%APPDATA%\MouseMover\config.json`
- `assets/icon.ico` — иконка exe / заголовка / трея
- `build.rs` + winresource — VERSIONINFO и ICON
- `msix/AppxManifest.xml` — desktop full-trust package для Microsoft Store
- `scripts/pack-msix.ps1` — MakeAppx x86/x64 bundle и `.msixupload`

Нельзя:

- packer / UPX / обфускация / прятание процесса
- хуки, инжект, драйверы, чужие процессы
- подмена имени окна под системное
