//! Thin helpers around `tray-icon` + `tao`.
//!
//! Builds a tray with optional menu entries and Quit, then runs a blocking
//! event loop. The callback must be `'static` (use `Arc<Mutex<_>>` for shared
//! host state). Icon is raw RGBA (`width * height * 4` bytes).

use std::collections::HashMap;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use thiserror::Error;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};

/// Tray errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Icon decode / tray build failed.
    #[error("tray error: {0}")]
    Tray(String),
}

/// One extra menu entry (besides Quit).
#[derive(Debug, Clone)]
pub struct MenuEntry {
    /// Visible label.
    pub label: String,
    /// Stable id returned to the callback.
    pub id: String,
}

/// Events from the tray menu.
#[derive(Debug, Clone)]
pub enum TrayEvent {
    /// User picked a custom entry (`MenuEntry.id`).
    Menu(String),
    /// User picked Quit.
    Quit,
}

/// Build an icon from uncompressed RGBA pixels.
pub fn icon_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Icon, Error> {
    Icon::from_rgba(rgba, width, height).map_err(|e| Error::Tray(e.to_string()))
}

/// Run a blocking event loop with a tray icon.
///
/// Returns only if the event loop exits after [`TrayEvent::Quit`] when the
/// callback returns `true` (“please exit”). On some platforms `run` diverges;
/// treat this as “blocks until quit”.
pub fn run_loop<F>(
    tooltip: &str,
    icon: Icon,
    extras: &[MenuEntry],
    mut on_event: F,
) -> Result<(), Error>
where
    F: 'static + FnMut(TrayEvent) -> bool,
{
    let menu = Menu::new();
    let mut id_map: HashMap<String, String> = HashMap::new();
    for e in extras {
        let item = MenuItem::new(&e.label, true, None);
        id_map.insert(item.id().as_ref().to_string(), e.id.clone());
        menu.append(&item)
            .map_err(|e| Error::Tray(e.to_string()))?;
    }
    let quit = MenuItem::new("Quit", true, None);
    let quit_id = quit.id().as_ref().to_string();
    menu.append(&quit)
        .map_err(|e| Error::Tray(e.to_string()))?;

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip)
        .with_icon(icon)
        .build()
        .map_err(|e| Error::Tray(e.to_string()))?;

    let event_loop = EventLoopBuilder::new().build();
    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(event) = menu_channel.try_recv() {
            let eid = event.id.as_ref();
            if eid == quit_id {
                if on_event(TrayEvent::Quit) {
                    *control_flow = ControlFlow::Exit;
                }
            } else if let Some(app_id) = id_map.get(eid) {
                let _ = on_event(TrayEvent::Menu(app_id.clone()));
            }
        }

        let _ = tray_channel.try_recv();
    });
}
