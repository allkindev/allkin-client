//! Allkin Client : une fenêtre qui affiche l'interface web d'un serveur Allkin.
//!
//! La page locale (`ui/`) choisit le serveur — adresse saisie, serveur déjà
//! connu ou trouvé sur le réseau —, puis la fenêtre navigue vers lui : c'est
//! l'interface d'Allkin elle-même qui s'affiche, sans iframe, ses cookies de
//! session se comportent donc comme dans un navigateur.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;
mod menu;
mod net;
mod scan;
mod store;
mod tailscale;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::webview::{DownloadEvent, NewWindowResponse, PageLoadEvent};
use tauri::{
    AppHandle, Manager, RunEvent, Url, Webview, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent, Wry,
};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

/// Ce qui est retenu de la fenêtre d'un lancement à l'autre. Pas la
/// visibilité : sur macOS, fermer la fenêtre la masque, elle resterait cachée
/// au lancement suivant. Pas le plein écran non plus.
const WINDOW_STATE: StateFlags = StateFlags::SIZE
    .union(StateFlags::POSITION)
    .union(StateFlags::MAXIMIZED);

/// Adresse de la page affichée, tenue à jour à chaque navigation.
#[derive(Default)]
pub struct Shown(pub Mutex<Option<Url>>);

/// Destination de chaque téléchargement en cours : macOS ne la redonne pas à
/// la fin du téléchargement.
#[derive(Default)]
struct Downloads(Mutex<HashMap<String, PathBuf>>);

/// La page locale, `#servers` pour qu'elle affiche le choix du serveur au lieu
/// de rouvrir le dernier.
pub fn home(fragment: Option<&str>) -> Url {
    let base = if cfg!(windows) {
        "http://tauri.localhost/index.html"
    } else {
        "tauri://localhost/index.html"
    };
    let mut url = Url::parse(base).expect("adresse de la page locale");
    url.set_fragment(fragment);
    url
}

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(WINDOW_STATE)
                // Restaurée à la main, avant d'afficher la fenêtre : elle
                // n'apparaît jamais à la taille par défaut pour sauter ensuite.
                .skip_initial_state("main")
                .build(),
        )
        .manage(scan::Scanner::default())
        .manage(Shown::default())
        .manage(Downloads::default())
        .invoke_handler(tauri::generate_handler![
            store::state,
            store::forget,
            store::set_lang,
            store::set_theme,
            net::connect,
            net::navigate,
            net::add,
            scan::scan,
            scan::stop_scan,
        ])
        .on_menu_event(menu::handle)
        .setup(|app| {
            app.manage(store::Store::load(app.handle()));
            // Après le chargement des réglages : les menus suivent la langue choisie.
            app.set_menu(menu::build(app.handle())?)?;
            main_window(app.handle())?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("impossible de démarrer Allkin Client")
        .run(on_event);
}

fn main_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let handle = app.clone();
    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("Allkin")
        .inner_size(1180.0, 780.0)
        .min_inner_size(400.0, 480.0)
        // Au premier lancement ; ensuite, la position retenue l'emporte.
        .center()
        // L'apparence choisie dans la page (sinon celle du système).
        .theme(app.state::<store::Store>().theme())
        .visible(false)
        .on_page_load(|window, payload| match payload.event() {
            PageLoadEvent::Started => {
                *window.state::<Shown>().0.lock().unwrap() = Some(payload.url().clone());
            }
            // WebView2 retient un zoom par site : celui choisi doit valoir partout.
            PageLoadEvent::Finished => {
                let _ = window.set_zoom(window.state::<store::Store>().zoom());
            }
        })
        .on_new_window(move |url, _| new_window(&handle, url))
        .on_download(on_download)
        .build()?;
    let _ = window.restore_state(WINDOW_STATE);
    window.set_zoom(app.state::<store::Store>().zoom())?;
    window.show()?;
    Ok(window)
}

/// `target="_blank"` et `window.open` : une page du même serveur s'ouvre dans
/// une fenêtre de l'application (elle partage la session), un site extérieur
/// dans le navigateur du système.
fn new_window(app: &AppHandle, url: Url) -> NewWindowResponse<Wry> {
    let shown = app.state::<Shown>();
    let same_site = shown
        .0
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|page| page.origin() == url.origin());
    if same_site {
        return NewWindowResponse::Allow;
    }
    if matches!(url.scheme(), "http" | "https" | "mailto") {
        let _ = open::that_detached(url.as_str());
    }
    NewWindowResponse::Deny
}

/// Les téléchargements vont dans le dossier Téléchargements (destination
/// proposée par la webview, sans écraser un fichier existant), puis le
/// fichier est montré dans le gestionnaire de fichiers.
fn on_download(webview: Webview<Wry>, event: DownloadEvent<'_>) -> bool {
    let pending = webview.state::<Downloads>();
    match event {
        DownloadEvent::Requested { url, destination } => {
            pending.0.lock().unwrap().insert(url.to_string(), destination.clone());
        }
        DownloadEvent::Finished { url, path, success } => {
            let remembered = pending.0.lock().unwrap().remove(url.as_str());
            if let Some(path) = path.or(remembered).filter(|_| success) {
                reveal(&path);
            }
        }
        _ => {}
    }
    true
}

fn reveal(path: &Path) {
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("explorer")
            .raw_arg(format!("/select,\"{}\"", path.display()))
            .spawn();
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    if let Some(folder) = path.parent() {
        let _ = open::that_detached(folder);
    }
}

fn on_event(app: &AppHandle, event: RunEvent) {
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } if label == "main" => {
            // Sauvé dès la fermeture : un arrêt brutal de l'application ne
            // perd pas la taille et la position.
            let _ = app.save_window_state(WINDOW_STATE);
            // Sur macOS, l'application reste ouverte : la fenêtre revient
            // telle quelle (page comprise) au clic sur l'icône du Dock.
            #[cfg(target_os = "macos")]
            {
                api.prevent_close();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            #[cfg(not(target_os = "macos"))]
            let _ = api;
        }
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        _ => {}
    }
}
