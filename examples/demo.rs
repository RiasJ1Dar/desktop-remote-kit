//! Smoke demo: AppId + update check against a public repo (no tray/Telegram).

use desktop_remote_kit::update::{self, GithubRepo};
use desktop_remote_kit::{autostart, AppId};

fn main() {
    let app = AppId::new("DesktopRemoteKitDemo", "Desktop Remote Kit Demo", "0.1.0");
    println!("app: {} {}", app.name, app.version);

    match autostart::is_enabled(&app) {
        Ok(v) => println!("autostart enabled: {v}"),
        Err(e) => println!("autostart: {e}"),
    }

    let repo = GithubRepo::new("RiasJ1Dar", "desktop-remote-kit");
    match update::check_latest(&app, &repo) {
        Ok(Some(n)) => println!("newer release: {} — {}", n.tag, n.html_url),
        Ok(None) => println!("update: up to date (or no newer release)"),
        Err(e) => println!("update check: {e}"),
    }
}
