//! Recherche des serveurs Allkin : cet ordinateur, les réseaux des interfaces
//! (Wi-Fi, Ethernet, VPN : WireGuard, OpenVPN, ZeroTier…) et les machines du
//! tailnet quand Tailscale est installé.

use std::collections::HashSet;
use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{State, Url};
use tokio::net::TcpStream;
use tokio::sync::{Notify, Semaphore};
use tokio::task::JoinSet;
use tokio::time::timeout;

use crate::net::{self, Probe, ALLKIN_PORT};
use crate::tailscale;

/// Connexions ouvertes en même temps : une application lancée depuis le
/// Finder n'a droit qu'à 256 descripteurs de fichiers.
const PARALLEL: usize = 128;

/// Une seule recherche à la fois : en lancer une arrête la précédente.
#[derive(Default)]
pub struct Scanner {
    stop: Arc<Notify>,
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum ScanEvent {
    /// Les adresses sont connues : `total` à vérifier, réparties en `networks`.
    Started { total: usize, networks: Vec<Network> },
    Progress { done: usize },
    Found { server: Found },
    /// `blocked` : aucune machine du réseau local n'a de route — sur macOS,
    /// l'accès au réseau local n'est pas (encore) autorisé à l'application.
    Finished { cancelled: bool, blocked: bool },
}

#[derive(Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Local,
    Lan,
    Vpn,
    Tailscale,
}

#[derive(Clone, Serialize)]
pub struct Network {
    kind: Kind,
    name: String,
    hosts: usize,
}

#[derive(Clone, Serialize)]
pub struct Found {
    url: String,
    name: Option<String>,
    kind: Kind,
}

struct Target {
    ip: Ipv4Addr,
    name: Option<String>,
    dns: Option<String>,
    kind: Kind,
    wait: Duration,
}

#[tauri::command]
pub async fn scan(scanner: State<'_, Scanner>, on_event: Channel<ScanEvent>) -> Result<(), ()> {
    scanner.stop.notify_waiters();
    let stopped = scanner.stop.notified();
    // Abandonner `search` abandonne ses tâches avec lui (JoinSet).
    let blocked = tokio::select! {
        blocked = search(on_event.clone()) => Some(blocked),
        _ = stopped => None,
    };
    let _ = on_event.send(ScanEvent::Finished {
        cancelled: blocked.is_none(),
        blocked: blocked.unwrap_or(false),
    });
    Ok(())
}

#[tauri::command]
pub fn stop_scan(scanner: State<'_, Scanner>) {
    scanner.stop.notify_waiters();
}

/// Vérifie toutes les adresses ; vrai si aucune du réseau local n'avait de route.
async fn search(on_event: Channel<ScanEvent>) -> bool {
    let (networks, targets) = tokio::task::spawn_blocking(targets).await.unwrap_or_default();
    let _ = on_event.send(ScanEvent::Started { total: targets.len(), networks });

    let lan = targets.iter().filter(|target| target.kind == Kind::Lan).count();
    let slots = Arc::new(Semaphore::new(PARALLEL));
    let done = Arc::new(AtomicUsize::new(0));
    let no_route = Arc::new(AtomicUsize::new(0));
    let mut tasks = JoinSet::new();
    for target in targets {
        let Ok(slot) = slots.clone().acquire_owned().await else { break };
        let (on_event, done, no_route) = (on_event.clone(), done.clone(), no_route.clone());
        tasks.spawn(async move {
            let kind = target.kind;
            match check(target).await {
                Checked::Found(server) => {
                    let _ = on_event.send(ScanEvent::Found { server });
                }
                Checked::NoRoute if kind == Kind::Lan => {
                    no_route.fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
            drop(slot);
            let done = done.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = on_event.send(ScanEvent::Progress { done });
        });
    }
    while tasks.join_next().await.is_some() {}
    lan > 0 && no_route.load(Ordering::Relaxed) == lan
}

/// Les adresses à vérifier, regroupées par réseau, sans doublon ni les
/// adresses de cet ordinateur (déjà couvert par 127.0.0.1).
fn targets() -> (Vec<Network>, Vec<Target>) {
    let mut networks = Vec::new();
    let mut targets = Vec::new();
    let interfaces = if_addrs::get_if_addrs().unwrap_or_default();
    let mut seen: HashSet<Ipv4Addr> = interfaces
        .iter()
        .filter_map(|interface| match interface.ip() {
            IpAddr::V4(ip) => Some(ip),
            IpAddr::V6(_) => None,
        })
        .collect();

    targets.push(Target {
        ip: Ipv4Addr::LOCALHOST,
        name: None,
        dns: None,
        kind: Kind::Local,
        wait: Duration::from_millis(400),
    });
    networks.push(Network { kind: Kind::Local, name: String::new(), hosts: 1 });

    let tailnet = tailscale::status();
    if let Some(tailnet) = &tailnet {
        let before = targets.len();
        for peer in &tailnet.peers {
            if seen.insert(peer.ip) {
                targets.push(Target {
                    ip: peer.ip,
                    name: Some(peer.name.clone()),
                    dns: peer.dns.clone(),
                    kind: Kind::Tailscale,
                    // Une machine jointe par un relais DERP met du temps à répondre.
                    wait: Duration::from_millis(2500),
                });
            }
        }
        let hosts = targets.len() - before;
        networks.push(Network { kind: Kind::Tailscale, name: tailnet.name.clone(), hosts });
    }

    for interface in &interfaces {
        let if_addrs::IfAddr::V4(address) = &interface.addr else { continue };
        let on_tailnet = tailnet.as_ref().is_some_and(|tailnet| tailnet.own.contains(&address.ip));
        if address.ip.is_loopback() || address.ip.is_link_local() || !interface.is_oper_up() || on_tailnet {
            continue;
        }
        let (kind, wait) = if interface.is_p2p() || looks_like_vpn(&interface.name) {
            (Kind::Vpn, Duration::from_millis(1500))
        } else {
            (Kind::Lan, Duration::from_millis(800))
        };
        let before = targets.len();
        for ip in subnet(address.ip, address.prefixlen) {
            if seen.insert(ip) {
                targets.push(Target { ip, name: None, dns: None, kind, wait });
            }
        }
        let hosts = targets.len() - before;
        if hosts > 0 {
            networks.push(Network { kind, name: interface.name.clone(), hosts });
        }
    }
    (networks, targets)
}

/// Les adresses du sous-réseau. Au-delà d'un /22 (1 022 machines), ou pour un
/// lien point à point (/31, /32, fréquent sur les VPN), seul le /24 de
/// l'adresse est parcouru.
fn subnet(ip: Ipv4Addr, prefix: u8) -> impl Iterator<Item = Ipv4Addr> {
    let prefix = if (22..=30).contains(&prefix) { prefix } else { 24 };
    let mask = u32::MAX << (32 - prefix);
    let network = u32::from(ip) & mask;
    let broadcast = network | !mask;
    (network + 1..broadcast).map(Ipv4Addr::from)
}

fn looks_like_vpn(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    ["utun", "tun", "tap", "wg", "ppp", "ipsec", "zt", "feth"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
        || ["vpn", "wireguard", "tailscale", "zerotier", "nordlynx", "netbird", "mullvad"]
            .iter()
            .any(|word| name.contains(word))
}

enum Checked {
    Found(Found),
    Nothing,
    /// Aucune route vers la machine, refusé sur-le-champ par le système.
    NoRoute,
}

/// Allkin répond-il sur cette machine ? D'abord sur son port, en HTTP ; pour
/// une machine du tailnet, aussi en HTTPS sur son nom MagicDNS, où le publie
/// `tailscale serve` — adresse préférée quand les deux répondent.
async fn check(target: Target) -> Checked {
    let (direct, proxied) = tokio::join!(
        reachable(target.ip, ALLKIN_PORT, target.wait),
        async {
            match target.dns {
                Some(_) => reachable(target.ip, 443, target.wait).await,
                None => Some(false),
            }
        }
    );
    let Some(direct) = direct else { return Checked::NoRoute };
    let mut url = None;
    if let (Some(true), Some(dns)) = (proxied, &target.dns) {
        url = allkin_at(format!("https://{dns}/")).await;
    }
    if url.is_none() && direct {
        url = allkin_at(format!("http://{}:{ALLKIN_PORT}/", target.ip)).await;
        if let (None, Some(dns)) = (&url, &target.dns) {
            url = allkin_at(format!("https://{dns}:{ALLKIN_PORT}/")).await;
        }
    }
    let Some(url) = url else { return Checked::Nothing };
    let name = match target.kind {
        Kind::Local => None,
        _ => match target.name {
            Some(name) => Some(name),
            None => reverse_lookup(target.ip).await,
        },
    };
    Checked::Found(Found { url: url.to_string(), name, kind: target.kind })
}

/// `Some(true)` si le port accepte la connexion, `None` s'il n'y a aucune
/// route vers la machine : c'est ce que répond macOS, sans délai, tant que
/// l'accès au réseau local n'est pas autorisé.
async fn reachable(ip: Ipv4Addr, port: u16, wait: Duration) -> Option<bool> {
    match timeout(wait, TcpStream::connect((ip, port))).await {
        Ok(Ok(_)) => Some(true),
        Ok(Err(error)) if error.kind() == ErrorKind::HostUnreachable => None,
        _ => Some(false),
    }
}

async fn allkin_at(address: String) -> Option<Url> {
    let url = Url::parse(&address).ok()?;
    let probe = tokio::task::spawn_blocking(move || net::probe(&url, Duration::from_secs(4)));
    match probe.await {
        Ok(Probe::Answer { base, allkin: true }) => Some(base),
        _ => None,
    }
}

/// Le nom de la machine (DNS inverse, mDNS sur macOS), s'il vient vite.
async fn reverse_lookup(ip: Ipv4Addr) -> Option<String> {
    let lookup = tokio::task::spawn_blocking(move || dns_lookup::lookup_addr(&IpAddr::V4(ip)).ok());
    let name = timeout(Duration::from_secs(2), lookup).await.ok()?.ok()??;
    let name = name.trim_end_matches('.');
    (!name.is_empty() && name.parse::<IpAddr>().is_err()).then(|| name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subnet_of_a_home_network() {
        let hosts: Vec<_> = subnet(Ipv4Addr::new(192, 168, 1, 48), 24).collect();
        assert_eq!(hosts.len(), 254);
        assert_eq!(hosts[0], Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(hosts[253], Ipv4Addr::new(192, 168, 1, 254));
    }

    #[test]
    fn large_or_point_to_point_networks_fall_back_to_the_24() {
        assert_eq!(subnet(Ipv4Addr::new(10, 8, 3, 7), 16).count(), 254);
        assert_eq!(subnet(Ipv4Addr::new(10, 8, 3, 7), 32).count(), 254);
        assert_eq!(subnet(Ipv4Addr::new(10, 8, 3, 7), 22).count(), 1022);
    }

    #[test]
    fn vpn_interfaces_are_recognised() {
        assert!(looks_like_vpn("utun3"));
        assert!(looks_like_vpn("wg0"));
        assert!(looks_like_vpn("ZeroTier One [8056c2e21c000001]"));
        assert!(!looks_like_vpn("en0"));
        assert!(!looks_like_vpn("Wi-Fi"));
    }

    /// Recherche réelle, événements affichés : `cargo test -- --ignored --nocapture`.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn real_scan() {
        let channel = Channel::new(|body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                if !json.contains(r#""event":"progress""#) {
                    println!("{json}");
                }
            }
            Ok(())
        });
        search(channel).await;
    }
}
