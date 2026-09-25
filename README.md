**Українська** · [English](README.en.md)

# desktop-remote-kit

Rust-набір для самодостатніх десктопних програм: автозапуск Windows, перевірка
GitHub Releases, опційний Telegram-пульт і helper для системного трея.
Хост-застосунок зберігає власну бізнес-логіку, команди й політику оновлення;
kit дає спільні низькорівневі блоки.

Це бібліотека, а не окремий продукт для кінцевого користувача. Поточна версія:
0.1.0.

## Cargo features

| Модуль | Feature | Що робить |
|---|---|---|
| `autostart` | завжди | Windows `HKCU\...\Run`; на Linux/macOS повертає `Unsupported` |
| `update` | `update`, увімкнено за замовчуванням | Перевіряє `releases/latest` на GitHub |
| `telegram` | `telegram` | `getUpdates`, `sendMessage` і callback `CommandHandler` |
| `tray` | `tray` | Обгортка `tray-icon` + `tao` з меню та блокувальним event loop |
| усі опційні модулі | `full` | `update + telegram + tray` |

Підключити весь набір:

```toml
[dependencies]
desktop-remote-kit = { git = "https://github.com/RiasJ1Dar/desktop-remote-kit", features = ["full"] }
```

Лише базовий `autostart` без HTTP і GUI-залежностей:

```toml
[dependencies]
desktop-remote-kit = { git = "https://github.com/RiasJ1Dar/desktop-remote-kit", default-features = false }
```

## Ідентифікатор застосунку

```rust
use desktop_remote_kit::AppId;

let app = AppId::new(
    "MyApp",
    "My App",
    env!("CARGO_PKG_VERSION"),
);
```

- `id` використовується як ім'я Windows Run value;
- `name` входить у підписи й User-Agent;
- `version` порівнюється з тегом GitHub Release.

## Автозапуск Windows

```rust
use desktop_remote_kit::{autostart, AppId};

let app = AppId::new("MyApp", "My App", "1.0.0");
let exe = std::env::current_exe()?;

autostart::enable(&app, &exe, "--background")?;
assert!(autostart::is_enabled(&app)?);

// коли автозапуск більше не потрібен:
// autostart::disable(&app)?;
```

`enable` приймає лише абсолютний шлях. Реєстрація діє для поточного
користувача в
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

## Перевірка GitHub Release

```rust
use desktop_remote_kit::update::{self, GithubRepo};

let repo = GithubRepo::new("You", "MyApp");
if let Some(release) = update::check_latest(&app, &repo)? {
    println!("нова версія: {} — {}", release.tag, release.html_url);
}
```

`NewerRelease` містить `tag`, `html_url` і `body` (текст релізу, якщо є). Чернетки не враховуються.

Модуль лише перевіряє наявність новішого релізу. Він не завантажує й не
підміняє бінарник. Порівняння версій числове за компонентами
(`1.2`, `1.2.0`, `v1.3.0`); нечислові хвости не мають повної semver-семантики.

Для підписаної доставки файлів див.
[ota-sign](https://github.com/RiasJ1Dar/ota-sign).

## Telegram-пульт

Хост визначає дозволені chat ID і обробку команд:

```rust
use desktop_remote_kit::telegram::{
    poll_once, BotConfig, CommandHandler, Incoming,
};

struct Handler;

impl CommandHandler for Handler {
    fn handle(&mut self, msg: &Incoming) -> Option<String> {
        match msg.text.as_str() {
            "/status" => Some("running".into()),
            _ => Some("unknown command".into()),
        }
    }
}

let cfg = BotConfig {
    token: std::env::var("TELEGRAM_BOT_TOKEN")?,
    allow_chats: vec![123456789],
};

let mut offset = 0;
let mut handler = Handler;
offset = poll_once(&cfg, offset, 30, &mut handler)?;
```

Надіслати повідомлення самостійно (наприклад, сповіщення без вхідної команди):

```rust
desktop_remote_kit::telegram::send_text(&cfg, 123456789, "готово")?;
```

Порожній `allow_chats` дозволяє команди з будь-якого чату. Токен і цикл
повторного виклику `poll_once` зберігає та контролює хост.

## Трей

Модуль `tray` приймає RGBA-іконку, додаткові `MenuEntry` і callback для
`TrayEvent::Menu` / `TrayEvent::Quit`. `run_loop` блокує потік до виходу,
тому хост має сам вирішити, де запускати event loop і як передавати спільний стан.
Пункт «Quit» додається завжди; callback повертає `true`, щоб вийти з циклу.

```rust
use desktop_remote_kit::tray::{icon_rgba, run_loop, MenuEntry, TrayEvent};

let icon = icon_rgba(vec![0x40; 16 * 16 * 4], 16, 16)?;   // RGBA, width, height
let extras = [MenuEntry { label: "Відкрити".into(), id: "open".into() }];
run_loop("MyApp", icon, &extras, |ev| match ev {
    TrayEvent::Menu(id) => { println!("{id}"); false }
    TrayEvent::Quit => true,                               // true = exit the loop
})?;
```

## Демо

```bash
cargo run --example demo --features update
```

Демо створює `AppId`, читає стан автозапуску й перевіряє останній реліз цього
репозиторію. Воно не змінює автозапуск.

## Межі відповідальності

- Підміна `.exe`, інсталятор і rollback належать хосту або `ota-sign`.
- Автозапуск Linux/macOS налаштовує хост через desktop entry або LaunchAgent.
- Набір не визначає готові Telegram-команди й не зберігає bot token.
- UI, журналювання та фонові worker-и залишаються в хост-застосунку.

План розвитку: [docs/roadmap.md](docs/roadmap.md).

## Розробка

```bash
cargo fmt --check
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

## Ліцензія

MIT