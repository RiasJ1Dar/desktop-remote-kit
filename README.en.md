[Українська](README.md) · **English**

# desktop-remote-kit

A Rust kit for self-contained desktop applications: Windows autostart, GitHub
Release checks, an optional Telegram remote, and a system-tray helper. The host
application keeps its business logic, commands, and update policy; the kit
provides reusable low-level building blocks.

This is a library, not an end-user application. Current version: 0.1.0.

## Cargo features

| Module | Feature | Purpose |
|---|---|---|
| `autostart` | always available | Windows `HKCU\...\Run`; returns `Unsupported` on Linux/macOS |
| `update` | `update`, enabled by default | Check GitHub `releases/latest` |
| `telegram` | `telegram` | `getUpdates`, `sendMessage`, and a `CommandHandler` callback |
| `tray` | `tray` | `tray-icon` + `tao` wrapper with a blocking event loop |
| all optional modules | `full` | `update + telegram + tray` |

Enable the complete kit:

```toml
[dependencies]
desktop-remote-kit = { git = "https://github.com/RiasJ1Dar/desktop-remote-kit", features = ["full"] }
```

Use only core autostart support, without HTTP or GUI dependencies:

```toml
[dependencies]
desktop-remote-kit = { git = "https://github.com/RiasJ1Dar/desktop-remote-kit", default-features = false }
```

## Application identity

```rust
use desktop_remote_kit::AppId;

let app = AppId::new(
    "MyApp",
    "My App",
    env!("CARGO_PKG_VERSION"),
);
```

- `id` becomes the Windows Run value name;
- `name` is used in labels and the User-Agent;
- `version` is compared with the GitHub Release tag.

## Windows autostart

```rust
use desktop_remote_kit::{autostart, AppId};

let app = AppId::new("MyApp", "My App", "1.0.0");
let exe = std::env::current_exe()?;

autostart::enable(&app, &exe, "--background")?;
assert!(autostart::is_enabled(&app)?);

// when autostart is no longer needed:
// autostart::disable(&app)?;
```

`enable` requires an absolute executable path. Registration is per-user under
`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

## GitHub Release check

```rust
use desktop_remote_kit::update::{self, GithubRepo};

let repo = GithubRepo::new("You", "MyApp");
if let Some(release) = update::check_latest(&app, &repo)? {
    println!("new version: {} — {}", release.tag, release.html_url);
}
```

`NewerRelease` holds `tag`, `html_url` and `body` (release notes, if any). Drafts are ignored.

The module only reports a newer release. It does not download or replace the
binary. Version comparison is numeric by component (`1.2`, `1.2.0`,
`v1.3.0`); non-numeric suffixes do not receive full semver treatment.

For signed file delivery, see
[ota-sign](https://github.com/RiasJ1Dar/ota-sign).

## Telegram remote

The host controls allowed chat IDs and command handling:

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

To send a message on your own (for example a notification with no incoming command):

```rust
desktop_remote_kit::telegram::send_text(&cfg, 123456789, "done")?;
```

An empty `allow_chats` accepts commands from every chat. The host stores the
token and owns the loop that repeatedly calls `poll_once`.

## Tray

The `tray` module accepts an RGBA icon, extra `MenuEntry` values, and a
callback for `TrayEvent::Menu` / `TrayEvent::Quit`. `run_loop` blocks until
exit, so the host decides where the event loop runs and how shared state is
passed into it.
A "Quit" item is always added; the callback returns `true` to leave the loop.

```rust
use desktop_remote_kit::tray::{icon_rgba, run_loop, MenuEntry, TrayEvent};

let icon = icon_rgba(vec![0x40; 16 * 16 * 4], 16, 16)?;   // RGBA, width, height
let extras = [MenuEntry { label: "Open".into(), id: "open".into() }];
run_loop("MyApp", icon, &extras, |ev| match ev {
    TrayEvent::Menu(id) => { println!("{id}"); false }
    TrayEvent::Quit => true,                               // true = exit the loop
})?;
```

## Demo

```bash
cargo run --example demo --features update
```

The demo constructs an `AppId`, reads autostart status, and checks this
repository's latest release. It does not change autostart.

## Scope

- Binary replacement, installers, and rollback belong to the host or `ota-sign`.
- Linux/macOS autostart belongs in host-specific desktop-entry or LaunchAgent setup.
- The kit does not define a fixed Telegram command set or store the bot token.
- UI, logging, and background workers remain in the host application.

See [docs/roadmap.md](docs/roadmap.md).

## Development

```bash
cargo fmt --check
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT