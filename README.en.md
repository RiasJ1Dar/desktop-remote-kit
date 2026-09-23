[Українська](README.md) · **English**

# desktop-remote-kit

Rust kit for self-contained desktop apps: tray, Windows autostart, optional
Telegram remote, and GitHub release check. The host keeps business logic; the
kit owns the shared shell.

This is a **library**, not an end-user product.

## Features (MVP 0.1)

| Module | Cargo feature | Role |
|---|---|---|
| `autostart` | always | Windows `HKCU\...\Run`. Linux/macOS → `Unsupported`. |
| `update` | `update` (default) | GitHub `releases/latest` **check only**. |
| `telegram` | `telegram` | Long-poll + `sendMessage`; host implements `CommandHandler`. |
| `tray` | `tray` | `tray-icon` + `tao` helper loop. |

`full` = update + telegram + tray.

## Quick start

```toml
[dependencies]
desktop-remote-kit = { git = "https://github.com/RiasJ1Dar/desktop-remote-kit", features = ["full"] }
```

```bash
cargo run --example demo --features update
```

## Out of scope (on purpose)

- Binary replace / installer — host app or future `ota-sign`.
- Linux/macOS autostart — host docs.
- Built-in Telegram command set — callback only.

## License

MIT
