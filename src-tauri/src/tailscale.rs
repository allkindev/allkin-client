//! Les machines du tailnet, d'après `tailscale status --json`. Leurs adresses
//! sont dispersées dans 100.64.0.0/10 : seule la liste de Tailscale les donne.

use std::io::Read;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

pub struct Peer {
    pub ip: Ipv4Addr,
    pub name: String,
    /// Nom MagicDNS (`machine.tailnet.ts.net`) : celui du certificat HTTPS
    /// quand Allkin est publié par `tailscale serve`.
    pub dns: Option<String>,
}

pub struct Tailnet {
    pub name: String,
    /// Les adresses de cet ordinateur sur le tailnet.
    pub own: Vec<Ipv4Addr>,
    /// Les autres machines, en ligne.
    pub peers: Vec<Peer>,
}

pub fn status() -> Option<Tailnet> {
    let output = run(cli()?)?;
    let status: Value = serde_json::from_slice(&output).ok()?;
    if status["BackendState"] != "Running" {
        return None;
    }
    let peers = status["Peer"]
        .as_object()
        .map(|peers| peers.values().filter_map(peer).collect())
        .unwrap_or_default();
    let name = status["MagicDNSSuffix"]
        .as_str()
        .or(status["CurrentTailnet"]["Name"].as_str())
        .unwrap_or_default()
        .to_string();
    Some(Tailnet { name, own: ipv4(&status["Self"]), peers })
}

fn peer(peer: &Value) -> Option<Peer> {
    if peer["Online"] == false {
        return None;
    }
    let ip = *ipv4(peer).first()?;
    let dns = peer["DNSName"]
        .as_str()
        .map(|name| name.trim_end_matches('.'))
        .filter(|name| !name.is_empty())
        .map(String::from);
    // Le nom MagicDNS plutôt que HostName : un iPhone s'appelle « localhost ».
    let name = dns
        .as_deref()
        .and_then(|dns| dns.split('.').next())
        .or(peer["HostName"].as_str())
        .unwrap_or_default()
        .to_string();
    Some(Peer { ip, name, dns })
}

fn ipv4(node: &Value) -> Vec<Ipv4Addr> {
    node["TailscaleIPs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|ip| ip.as_str()?.parse().ok())
        .collect()
}

/// La commande `tailscale`. Une application lancée depuis le Finder ou le
/// menu Démarrer n'hérite pas du PATH du terminal : les emplacements
/// d'installation habituels sont essayés aussi.
fn cli() -> Option<PathBuf> {
    let file = if cfg!(windows) { "tailscale.exe" } else { "tailscale" };
    let in_path: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).map(|dir| dir.join(file)).collect())
        .unwrap_or_default();
    #[cfg(target_os = "macos")]
    let known = [
        "/usr/local/bin/tailscale",
        "/opt/homebrew/bin/tailscale",
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
    ]
    .map(PathBuf::from);
    #[cfg(windows)]
    let known = [std::env::var_os("ProgramFiles")
        .map_or_else(|| PathBuf::from(r"C:\Program Files"), PathBuf::from)
        .join(r"Tailscale\tailscale.exe")];
    #[cfg(not(any(target_os = "macos", windows)))]
    let known = [
        "/usr/bin/tailscale",
        "/usr/sbin/tailscale",
        "/usr/local/bin/tailscale",
        "/snap/bin/tailscale",
    ]
    .map(PathBuf::from);
    in_path.into_iter().chain(known).find(|path| path.is_file())
}

/// Lance `tailscale status --json`, abandonné au bout de 5 s (démon bloqué).
fn run(cli: PathBuf) -> Option<Vec<u8>> {
    let mut command = Command::new(cli);
    command
        .args(["status", "--json"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().ok()?;
    let mut stdout = child.stdout.take()?;
    let reader = thread::spawn(move || {
        let mut output = Vec::new();
        let _ = stdout.read_to_end(&mut output);
        output
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return reader.join().ok(),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}
