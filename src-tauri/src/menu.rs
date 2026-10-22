//! Barre de menus. Aucun raccourci d'Allkin n'y est repris : Ctrl/Cmd + W,
//! Tab, 1…9, Maj+T et J restent à ses onglets et à ses missions.

use tauri::menu::{Menu, MenuBuilder, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Manager, WebviewWindow, Wry};

use crate::store::Store;
use crate::{home, i18n, Shown};

const ZOOMS: [f64; 13] = [0.5, 0.67, 0.75, 0.8, 0.9, 1.0, 1.1, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0];

/// Dans la langue choisie dans la page (voir `store::set_lang`).
pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let t = i18n::menu(app.state::<Store>().lang());
    let item = |id: &str, text: &str, shortcut: Option<&str>| {
        let item = MenuItemBuilder::with_id(id, text);
        match shortcut {
            Some(shortcut) => item.accelerator(shortcut),
            None => item,
        }
        .build(app)
    };

    let server = SubmenuBuilder::new(app, t.server)
        .item(&item("servers", t.servers, None)?)
        .separator()
        .item(&item("reload", t.reload, Some("CmdOrCtrl+R"))?)
        .item(&item("browser", t.browser, None)?);
    #[cfg(not(target_os = "macos"))]
    let server = server.separator().item(&item("quit", t.quit, Some("Ctrl+Q"))?);

    let view = SubmenuBuilder::new(app, t.view)
        .item(&item("zoom-in", t.zoom_in, Some("CmdOrCtrl+="))?)
        .item(&item("zoom-out", t.zoom_out, Some("CmdOrCtrl+-"))?)
        .item(&item("zoom-reset", t.actual_size, Some("CmdOrCtrl+0"))?)
        .separator();
    #[cfg(target_os = "macos")]
    let view = view.fullscreen_with_text(t.fullscreen);
    #[cfg(not(target_os = "macos"))]
    let view = view.item(&item("fullscreen", t.fullscreen, Some("F11"))?);

    #[cfg(target_os = "macos")]
    {
        let application = SubmenuBuilder::new(app, "Allkin Client")
            .about_with_text(t.about, None)
            .separator()
            .services_with_text(t.services)
            .separator()
            .hide_with_text(t.hide)
            .hide_others_with_text(t.hide_others)
            .show_all_with_text(t.show_all)
            .separator()
            .quit_with_text(t.quit)
            .build()?;
        // Sans menu Édition, macOS n'envoie ni copier ni coller à la page.
        let edit = SubmenuBuilder::new(app, t.edit)
            .undo_with_text(t.undo)
            .redo_with_text(t.redo)
            .separator()
            .cut_with_text(t.cut)
            .copy_with_text(t.copy)
            .paste_with_text(t.paste)
            .select_all_with_text(t.select_all)
            .build()?;
        let window = SubmenuBuilder::new(app, t.window)
            .minimize_with_text(t.minimize)
            .maximize_with_text(t.zoom)
            .separator()
            .bring_all_to_front_with_text(t.front)
            .build()?;
        MenuBuilder::new(app)
            .item(&application)
            .item(&server.build()?)
            .item(&edit)
            .item(&view.build()?)
            .item(&window)
            .build()
    }
    #[cfg(not(target_os = "macos"))]
    MenuBuilder::new(app)
        .item(&server.build()?)
        .item(&view.build()?)
        .build()
}

pub fn handle(app: &AppHandle, event: MenuEvent) {
    let Some(window) = app.get_webview_window("main") else { return };
    match event.id().as_ref() {
        "servers" => {
            let _ = window.navigate(home(Some("servers")));
            let _ = window.show();
        }
        "reload" => {
            let _ = window.reload();
        }
        "browser" => {
            // La page affichée si c'est celle d'un serveur, sinon le dernier serveur.
            let shown = app.state::<Shown>().0.lock().unwrap().clone();
            let page = shown
                .filter(|page| matches!(page.scheme(), "http" | "https") && page.host_str() != Some("tauri.localhost"))
                .map(|page| page.to_string())
                .or_else(|| app.state::<Store>().current());
            if let Some(page) = page {
                let _ = open::that_detached(page);
            }
        }
        "zoom-in" => zoom(app, &window, |zoom| ZOOMS.into_iter().find(|&level| level > zoom + 0.01)),
        "zoom-out" => zoom(app, &window, |zoom| ZOOMS.into_iter().rev().find(|&level| level < zoom - 0.01)),
        "zoom-reset" => zoom(app, &window, |_| Some(1.0)),
        "fullscreen" => {
            let fullscreen = window.is_fullscreen().unwrap_or(false);
            let _ = window.set_fullscreen(!fullscreen);
        }
        "quit" => app.exit(0),
        _ => {}
    }
}

fn zoom(app: &AppHandle, window: &WebviewWindow, next: impl Fn(f64) -> Option<f64>) {
    let store = app.state::<Store>();
    if let Some(zoom) = next(store.zoom()) {
        store.set_zoom(zoom);
        let _ = window.set_zoom(zoom);
    }
}
