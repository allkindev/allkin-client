//! Les serveurs connus et les réglages, dans `servers.json` du dossier de
//! configuration de l'application.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, Theme};

use crate::{i18n, menu};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub url: String,
    pub name: String,
    #[serde(default)]
    pub last_used: u64,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Data {
    /// Le plus récemment utilisé d'abord, puis les ajoutés jamais ouverts.
    servers: Vec<Server>,
    /// Le serveur rouvert au lancement.
    current: Option<String>,
    zoom: Option<f64>,
    /// Choisie dans la page ; sinon celle du système.
    lang: Option<String>,
    /// « light » ou « dark », choisi dans la page ; sinon celui du système.
    theme: Option<String>,
}

pub struct Store {
    path: PathBuf,
    data: Mutex<Data>,
}

impl Store {
    pub fn load(app: &AppHandle) -> Self {
        let path = app
            .path()
            .app_config_dir()
            .unwrap_or_default()
            .join("servers.json");
        let data = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self { path, data: Mutex::new(data) }
    }

    fn save(&self, data: &Data) {
        if let Some(folder) = self.path.parent() {
            let _ = fs::create_dir_all(folder);
        }
        // Écrit à côté puis renomme : un arrêt brutal ne laisse jamais un
        // fichier à moitié écrit.
        let temporary = self.path.with_extension("json.tmp");
        if let Ok(json) = serde_json::to_vec_pretty(data) {
            if fs::write(&temporary, json).is_ok() {
                let _ = fs::rename(&temporary, &self.path);
            }
        }
    }

    /// Ajoute un serveur en fin de liste, s'il n'y est pas déjà ; renvoie le
    /// nom sous lequel il y figure.
    pub fn add(&self, url: &str, name: String) -> String {
        let mut data = self.data.lock().unwrap();
        if let Some(known) = data.servers.iter().find(|server| server.url == url) {
            return known.name.clone();
        }
        data.servers.push(Server { url: url.into(), name: name.clone(), last_used: 0 });
        self.save(&data);
        name
    }

    /// Retient le serveur auquel on vient de se connecter : il passe en tête
    /// et sera rouvert au prochain lancement. Sans nom donné, il garde celui
    /// qu'il avait déjà.
    pub fn remember(&self, url: &str, name: Option<String>, fallback: String) -> String {
        let mut data = self.data.lock().unwrap();
        let known = data.servers.iter().position(|server| server.url == url);
        let previous = known.map(|index| data.servers.remove(index).name);
        let name = name.or(previous).unwrap_or(fallback);
        let last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        data.servers.insert(0, Server { url: url.into(), name: name.clone(), last_used });
        data.current = Some(url.into());
        self.save(&data);
        name
    }

    pub fn forget(&self, url: &str) {
        let mut data = self.data.lock().unwrap();
        data.servers.retain(|server| server.url != url);
        if data.current.as_deref() == Some(url) {
            data.current = None;
        }
        self.save(&data);
    }

    pub fn current(&self) -> Option<String> {
        self.data.lock().unwrap().current.clone()
    }

    pub fn zoom(&self) -> f64 {
        self.data.lock().unwrap().zoom.unwrap_or(1.0)
    }

    pub fn set_zoom(&self, zoom: f64) {
        let mut data = self.data.lock().unwrap();
        data.zoom = Some(zoom);
        self.save(&data);
    }

    pub fn lang(&self) -> &'static str {
        i18n::resolve(self.data.lock().unwrap().lang.as_deref())
    }

    /// `None` : celui du système.
    pub fn theme(&self) -> Option<Theme> {
        match self.data.lock().unwrap().theme.as_deref() {
            Some("dark") => Some(Theme::Dark),
            Some("light") => Some(Theme::Light),
            _ => None,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    lang: &'static str,
    os: &'static str,
    theme: Option<String>,
    servers: Vec<Server>,
    current: Option<String>,
}

/// Ce que la page locale affiche au démarrage.
#[tauri::command]
pub fn state(store: State<Store>) -> View {
    let lang = store.lang();
    let data = store.data.lock().unwrap();
    View {
        lang,
        os: std::env::consts::OS,
        theme: data.theme.clone(),
        servers: data.servers.clone(),
        current: data.current.clone(),
    }
}

#[tauri::command]
pub fn forget(store: State<Store>, url: String) {
    store.forget(&url);
}

/// Change la langue de la page et des menus.
#[tauri::command]
pub fn set_lang(app: AppHandle, lang: String) -> Result<(), String> {
    let store = app.state::<Store>();
    {
        let mut data = store.data.lock().unwrap();
        data.lang = Some(i18n::resolve(Some(&lang)).to_string());
        store.save(&data);
    }
    let menu = menu::build(&app).map_err(|error| error.to_string())?;
    app.set_menu(menu).map_err(|error| error.to_string())?;
    Ok(())
}

/// Apparence claire ou sombre, pour toute l'application : la page d'Allkin
/// la suit aussi tant qu'on n'y a pas choisi la sienne.
#[tauri::command]
pub fn set_theme(app: AppHandle, theme: String) {
    let dark = theme == "dark";
    let store = app.state::<Store>();
    {
        let mut data = store.data.lock().unwrap();
        data.theme = Some(if dark { "dark" } else { "light" }.into());
        store.save(&data);
    }
    app.set_theme(Some(if dark { Theme::Dark } else { Theme::Light }));
}
