//! Joindre un serveur : quelles adresses essayer pour ce qui a été saisi, et
//! est-ce bien Allkin qui répond.

use std::io::ErrorKind;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Manager, Url, WebviewWindow};
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use ureq::ResponseExt;

use crate::store::Store;

/// Port de l'interface web d'Allkin (`webPort` de sa configuration).
pub const ALLKIN_PORT: u16 = 9191;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(6);

/// Pourquoi personne ne répond, de la cause la moins parlante à la plus
/// parlante : entre plusieurs adresses essayées, la plus parlante est donnée.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Reason {
    Other,
    Dns,
    Timeout,
    /// Aucune route : VPN coupé, ou, sur macOS, accès au réseau local refusé.
    NoRoute,
    Refused,
    Tls,
}

pub enum Probe {
    /// Un serveur web répond, à l'adresse `base` une fois les redirections suivies.
    Answer { base: Url, allkin: bool },
    Fail(Reason),
}

/// Interroge `base` : n'importe quelle réponse HTTP prouve qu'un serveur est
/// là ; le manifeste de l'application web (ou, à défaut, l'état de première
/// configuration) dit si c'est Allkin. Les deux sont publics.
pub fn probe(base: &Url, timeout: Duration) -> Probe {
    let tls = TlsConfig::builder()
        .provider(TlsProvider::NativeTls)
        .root_certs(RootCerts::PlatformVerifier)
        .build();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .user_agent(concat!("AllkinClient/", env!("CARGO_PKG_VERSION")))
        .tls_config(tls)
        .build()
        .into();

    let Ok(manifest) = base.join("manifest.webmanifest") else {
        return Probe::Fail(Reason::Other);
    };
    let mut response = match agent.get(manifest.as_str()).call() {
        Ok(response) => response,
        Err(error) => return Probe::Fail(reason(&error)),
    };
    // Après une redirection (http → https, nom canonique…), c'est l'adresse
    // finale qu'on retient.
    let base = Url::parse(&response.get_uri().to_string())
        .and_then(|url| url.join("./"))
        .unwrap_or_else(|_| base.clone());
    let named_allkin = response.status() == 200 && read_json(response.body_mut()).is_some_and(|json| {
        ["name", "short_name"].iter().any(|key| {
            json.get(key)
                .and_then(|name| name.as_str())
                .is_some_and(|name| name.starts_with("Allkin"))
        })
    });
    let allkin = named_allkin || answers_setup_status(&agent, &base);
    Probe::Answer { base, allkin }
}

fn answers_setup_status(agent: &ureq::Agent, base: &Url) -> bool {
    let Ok(url) = base.join("api/setup-status") else {
        return false;
    };
    agent
        .get(url.as_str())
        .call()
        .ok()
        .filter(|response| response.status() == 200)
        .and_then(|mut response| read_json(response.body_mut()))
        .is_some_and(|json| json.get("needsSetup").is_some())
}

fn read_json(body: &mut ureq::Body) -> Option<serde_json::Value> {
    let text = body.with_config().limit(256 * 1024).read_to_string().ok()?;
    serde_json::from_str(&text).ok()
}

fn reason(error: &ureq::Error) -> Reason {
    match error {
        ureq::Error::HostNotFound => Reason::Dns,
        ureq::Error::Timeout(_) => Reason::Timeout,
        ureq::Error::Tls(_) | ureq::Error::NativeTls(_) | ureq::Error::Der(_) => Reason::Tls,
        ureq::Error::Io(error) => match error.kind() {
            ErrorKind::ConnectionRefused => Reason::Refused,
            ErrorKind::TimedOut => Reason::Timeout,
            ErrorKind::HostUnreachable | ErrorKind::NetworkUnreachable => Reason::NoRoute,
            _ => Reason::Other,
        },
        _ => Reason::Other,
    }
}

/// Les adresses à essayer pour ce qui a été saisi, la préférée d'abord.
///
/// Sans schéma ni port, `hôte` peut être Allkin servi directement
/// (`http://hôte:9191`) ou derrière un proxy TLS (`https://hôte`, ce que fait
/// `tailscale serve`) : les deux sont essayés, HTTPS en premier quand les deux
/// répondent. Allkin se sert toujours à la racine (ses pages appellent
/// `/api/…`) : chemin, requête et fragment sont ignorés.
pub fn candidates(input: &str) -> Vec<Url> {
    let input = input.trim();
    if input.contains("://") {
        return Url::parse(input).ok().and_then(root).into_iter().collect();
    }
    let Some(plain) = Url::parse(&format!("http://{input}")).ok().and_then(root) else {
        return Vec::new();
    };
    let mut secure = plain.clone();
    let _ = secure.set_scheme("https");
    if plain.port().is_some() {
        return vec![plain, secure];
    }
    let mut allkin = plain.clone();
    let _ = allkin.set_port(Some(ALLKIN_PORT));
    vec![secure, allkin, plain]
}

fn root(mut url: Url) -> Option<Url> {
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none_or(str::is_empty) {
        return None;
    }
    url.set_path("/");
    url.set_query(None);
    url.set_fragment(None);
    let _ = url.set_username("");
    let _ = url.set_password(None);
    Some(url)
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Outcome {
    /// Allkin répond à `url` ; le serveur est dans la liste, sous `name`.
    Ready { url: String, name: String },
    /// Un serveur répond, mais ce n'est pas Allkin : il n'est ni ajouté ni ouvert.
    NotAllkin,
    Unreachable { reason: Reason },
    Invalid,
}

/// L'adresse d'Allkin pour ce qui a été saisi (ou enregistré), ou pourquoi
/// il n'y en a pas.
async fn find(address: &str) -> Result<Url, Outcome> {
    let candidates = candidates(address);
    if candidates.is_empty() {
        return Err(Outcome::Invalid);
    }

    // Toutes les variantes à la fois : une adresse muette ne fait pas
    // attendre les autres.
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    for (index, url) in candidates.iter().cloned().enumerate() {
        let sender = sender.clone();
        tokio::task::spawn_blocking(move || {
            let _ = sender.send((index, probe(&url, CONNECT_TIMEOUT)));
        });
    }
    drop(sender);

    let mut results: Vec<Option<Probe>> = candidates.iter().map(|_| None).collect();
    while let Some((index, probe)) = receiver.recv().await {
        results[index] = Some(probe);
        // Allkin l'emporte dès que les adresses préférées ont toutes répondu.
        let first_allkin = results.iter().position(|result| {
            matches!(result, Some(Probe::Answer { allkin: true, .. }))
        });
        if let Some(index) = first_allkin {
            if results[..index].iter().all(Option::is_some) {
                if let Some(Probe::Answer { base, .. }) = results[index].take() {
                    return Ok(base);
                }
            }
        }
    }

    if results.iter().any(|result| matches!(result, Some(Probe::Answer { .. }))) {
        return Err(Outcome::NotAllkin);
    }
    let reason = results
        .iter()
        .filter_map(|result| match result {
            Some(Probe::Fail(reason)) => Some(*reason),
            _ => None,
        })
        .max()
        .unwrap_or(Reason::Other);
    Err(Outcome::Unreachable { reason })
}

fn host_of(url: &Url) -> String {
    url.host_str().unwrap_or(url.as_str()).to_string()
}

/// Ouvrir un serveur : s'il répond et que c'est Allkin, il passe en tête de
/// liste et sera rouvert au prochain lancement.
#[tauri::command]
pub async fn connect(app: AppHandle, address: String, name: Option<String>) -> Outcome {
    match find(&address).await {
        Ok(url) => {
            let name = name.filter(|name| !name.trim().is_empty());
            let name = app.state::<Store>().remember(url.as_str(), name, host_of(&url));
            Outcome::Ready { url: url.to_string(), name }
        }
        Err(outcome) => outcome,
    }
}

/// Montrer `url` dans la fenêtre. La page ne s'y rend pas elle-même : macOS
/// confie à une webapp Safari (« Ajouter au Dock ») toute navigation lancée
/// par une page vers son site, alors qu'un chargement demandé par
/// l'application reste dans la fenêtre.
#[tauri::command]
pub fn navigate(window: WebviewWindow, url: Url) -> Result<(), String> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err("invalid".into());
    }
    window.navigate(url).map_err(|error| error.to_string())
}

/// Ajouter un serveur à la liste, s'il répond et que c'est Allkin.
#[tauri::command]
pub async fn add(app: AppHandle, address: String, name: Option<String>) -> Outcome {
    match find(&address).await {
        Ok(url) => {
            let name = name.filter(|name| !name.trim().is_empty()).unwrap_or_else(|| host_of(&url));
            let name = app.state::<Store>().add(url.as_str(), name);
            Outcome::Ready { url: url.to_string(), name }
        }
        Err(outcome) => outcome,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(input: &str) -> Vec<String> {
        candidates(input).iter().map(Url::to_string).collect()
    }

    #[test]
    fn bare_host_tries_https_then_allkin_port_then_http() {
        assert_eq!(
            strings("cerebro.local"),
            ["https://cerebro.local/", "http://cerebro.local:9191/", "http://cerebro.local/"]
        );
    }

    #[test]
    fn explicit_port_tries_http_then_https() {
        assert_eq!(strings(" 192.168.1.20:9191 "), ["http://192.168.1.20:9191/", "https://192.168.1.20:9191/"]);
    }

    #[test]
    fn full_url_is_kept_at_its_root() {
        assert_eq!(strings("https://allkin.example.com/agents?x=1#chat"), ["https://allkin.example.com/"]);
    }

    #[test]
    fn nonsense_is_rejected() {
        assert!(strings("").is_empty());
        assert!(strings("ftp://example.com").is_empty());
        assert!(strings("not an address").is_empty());
    }
}
