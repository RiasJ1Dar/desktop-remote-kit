**Українська** · [English](README.en.md)

# desktop-remote-kit

Rust-набір для самодостатніх десктопних програм: трей, автозапуск Windows,
опційний Telegram-пульт і перевірка релізу на GitHub. Хост-застосунок лишає
свою бізнес-логіку; спільне «обв’язування» бере кіт.

Це **бібліотека**, не окремий продукт для кінцевого користувача.

## Можливості (MVP 0.1)

| Модуль | Feature | Що робить |
|---|---|---|
| `autostart` | завжди | Windows: `HKCU\...\Run`. Linux/macOS — `Unsupported` (налаштовуй у хості). |
| `update` | `update` (default) | `releases/latest` на GitHub; **лише перевірка**, без підміни бінарника. |
| `telegram` | `telegram` | Long-poll `getUpdates` + `sendMessage`; хост реалізує `CommandHandler`. |
| `tray` | `tray` | Обгортка `tray-icon` + `tao`: меню, Quit, іконка RGBA. |

Feature `full` = update + telegram + tray.

## Швидкий старт

```toml
[dependencies]
desktop-remote-kit = { git = "https://github.com/RiasJ1Dar/desktop-remote-kit", features = ["full"] }
```

```rust
use desktop_remote_kit::{autostart, update, AppId};
use desktop_remote_kit::update::GithubRepo;
use std::env;
use std::path::PathBuf;

let app = AppId::new("MyApp", "My App", "1.0.0");
let exe = env::current_exe().unwrap();
let _ = autostart::enable(&app, &exe, "");

let repo = GithubRepo::new("You", "MyApp");
if let Ok(Some(newer)) = update::check_latest(&app, &repo) {
    eprintln!("є оновлення: {} {}", newer.tag, newer.html_url);
}
```

Демо без трею:

```bash
cargo run --example demo --features update
```

## Чого навмисне немає

- Підміна `.exe` / інсталятор оновлень — у хості або в майбутньому `ota-sign`.
- Автозапуск Linux/macOS — у документації хоста (як у TwitchDropFarm).
- Готовий набір Telegram-команд — лише колбек.

## Ліцензія

MIT
