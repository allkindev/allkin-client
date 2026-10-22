# Allkin Client

Application de bureau (macOS, Windows, Linux) qui affiche l'interface d'un serveur
[Allkin](https://allk.in) dans sa propre fenêtre.

- **Liste de serveurs** : on y ajoute des serveurs Allkin, trouvés sur le réseau ou saisis à la
  main, puis un clic ouvre leur page. Au lancement suivant, le dernier ouvert se rouvre
  directement.
- **Recherche sur le réseau** : trouve les serveurs Allkin de cet ordinateur, du réseau local,
  des VPN (WireGuard, OpenVPN, ZeroTier…) et du tailnet Tailscale, avec une barre de
  progression.
- **Langue** (français, anglais, espagnol, allemand) et **apparence** claire ou sombre, au
  choix dans la page.
- **Fenêtre** : taille, position et état agrandi sont retenus d'un lancement à l'autre.
- **Léger** : construit avec [Tauri 2](https://tauri.app), qui utilise la webview du système
  (WebKit sur macOS et Linux, WebView2 sur Windows) au lieu d'embarquer un navigateur. Pas de
  framework ni d'étape de compilation côté page.

## Installation

Les fichiers sont produits par GitHub Actions (voir [Publier une version](#publier-une-version)) :

| Système | Fichier | Remarque |
|---|---|---|
| macOS (Apple Silicon / Intel) | `Allkin Client_x.y.z_aarch64.dmg` / `_x64.dmg` | Application non signée : au premier lancement, clic droit › Ouvrir (ou `xattr -dr com.apple.quarantine "/Applications/Allkin Client.app"`). |
| Windows 10/11 | `Allkin Client_x.y.z_x64-setup.exe`, ou `Allkin Client_x64-portable.exe` sans installation | SmartScreen avertit tant que l'exécutable n'est pas signé (« Informations complémentaires › Exécuter quand même »). |
| Linux | `.deb` (Debian, Ubuntu), `.rpm` (Fedora…), `.AppImage` | `.deb`/`.rpm` s'appuient sur WebKitGTK du système ; l'AppImage l'embarque, d'où sa taille. |

## Utilisation

- **Ajouter un serveur** : **Lancer une recherche** ouvre la liste des serveurs Allkin trouvés,
  chacun avec un **+** pour l'ajouter ; **Manuel** demande une adresse. Seul un serveur Allkin
  entre dans la liste : une adresse qui répond sans être Allkin est refusée.
- **Adresse** : `192.168.1.20`, `mon-serveur:9191`, `https://allkin.example.com`… Sans port
  précisé, Allkin est cherché en HTTPS (proxy TLS, `tailscale serve`), puis sur son port
  **9191**, puis en HTTP. Seule la racine du site compte : chemin et paramètres sont ignorés.
- **Revenir à la liste** depuis Allkin : menu **Serveur › Changer de serveur…**
- **Langue et apparence** : en haut à gauche et à droite de la page. La langue vaut aussi pour
  les menus ; l'apparence vaut pour toute l'application, page d'Allkin comprise tant qu'on n'y
  a pas choisi la sienne.
- **Menus** : Recharger la page (Ctrl/Cmd + R), Ouvrir dans le navigateur, zoom
  (Ctrl/Cmd + `=` / `-` / `0`, retenu). Les raccourcis d'Allkin (Ctrl/Cmd + W, Tab, 1…9,
  Maj + T, J) restent à Allkin.
- **Liens** : un lien vers un autre site s'ouvre dans le navigateur du système ; une page du
  même serveur ouverte dans un nouvel onglet s'ouvre dans une fenêtre de l'application (même
  session).
- **Téléchargements** (sauvegardes, exports…) : enregistrés dans le dossier Téléchargements,
  puis montrés dans le Finder / l'Explorateur.
- **macOS** : fermer la fenêtre garde l'application ouverte ; un clic sur l'icône du Dock la
  rouvre telle quelle. Au premier accès au réseau local, macOS demande l'autorisation : si elle
  est refusée, la page l'indique (Réglages Système › Confidentialité et sécurité › Réseau local).

## Comment la recherche fonctionne

Un serveur Allkin est reconnu à son manifeste d'application web (`/manifest.webmanifest`, nom
« Allkin ») ou, à défaut, à `/api/setup-status` : deux adresses publiques, sans connexion.

| Réseau | Adresses vérifiées |
|---|---|
| Cet ordinateur | `127.0.0.1:9191` |
| Réseau local et VPN | toutes les adresses du sous-réseau de chaque interface active (jusqu'à un /22 ; au-delà, ou pour un lien point à point, le /24 de l'adresse), port 9191 |
| Tailscale | les machines en ligne du tailnet, d'après `tailscale status --json` : port 9191 en HTTP, et HTTPS sur leur nom MagicDNS (`tailscale serve`), préféré quand les deux répondent |

128 connexions à la fois, délai de 0,8 s par adresse sur le réseau local (1,5 s sur un VPN,
2,5 s sur Tailscale) : un /24 se parcourt en 1 à 2 secondes. La recherche peut être arrêtée à
tout moment.

## Données

Dans le dossier de configuration de l'application (`in.allk.client`) :

- macOS : `~/Library/Application Support/in.allk.client/`
- Windows : `%APPDATA%\in.allk.client\`
- Linux : `~/.config/in.allk.client/`

`servers.json` (serveurs connus, dernier ouvert, zoom, langue, apparence) et `.window-state.json`
(fenêtre).
Les cookies des serveurs (session Allkin) sont gardés par la webview, comme dans un navigateur.

## Développement

Prérequis : [Rust](https://rustup.rs) et Node.js (pour lancer la CLI Tauri avec `npx`, rien
n'est installé dans le projet). Sous Linux, les dépendances de WebKitGTK :
`sudo apt install libwebkit2gtk-4.1-dev libssl-dev libxdo-dev librsvg2-dev`.

```bash
cd src-tauri
npx @tauri-apps/cli@2 dev      # lance l'application (recompile au changement)
npx @tauri-apps/cli@2 build    # paquets du système courant, dans target.nosync/release/bundle/
cargo test                     # tests unitaires
cargo test -- --ignored --nocapture   # recherche réelle sur le réseau, résultats affichés
```

Le dossier de compilation est `src-tauri/target.nosync` (voir `src-tauri/.cargo/config.toml`) :
iCloud Drive ne synchronise pas un dossier en `.nosync`.

```
ui/                    la page locale : HTML, CSS, JS sans dépendance (textes en fr, en, es, de)
src-tauri/src/
  main.rs              fenêtre, liens externes, téléchargements, fermeture sur macOS
  net.rs               adresses à essayer, reconnaissance d'Allkin, commandes connect et add
  scan.rs              recherche sur le réseau (progression envoyée à la page)
  tailscale.rs         machines du tailnet (tailscale status --json)
  store.rs             serveurs connus et réglages (servers.json), langue, apparence
  menu.rs, i18n.rs     barre de menus et ses textes
src-tauri/Info.plist   autorisations macOS (réseau local, micro, HTTP dans la webview)
src-tauri/icons/       icônes de l'application, tirées de celles d'Allkin (generate.sh)
```

La marque et les icônes sont celles du projet Allkin (`web/` de `/home/tony/allkin`), jamais
redessinées : `ui/logo.webp` est une copie de son `web/logo.webp`. Après un changement de logo,
recopier ce fichier et régénérer les icônes depuis une copie locale de son dossier `web/` :

```bash
src-tauri/icons/generate.sh ~/chemin/vers/allkin/web
```

## Publier une version

1. Mettre à jour `version` dans `src-tauri/tauri.conf.json` et `src-tauri/Cargo.toml`.
2. Pousser un tag : `git tag v0.1.0 && git push origin v0.1.0`.
3. Le workflow `.github/workflows/build.yml` construit les quatre variantes ; si toutes
   réussissent, il publie la release du tag avec tous les fichiers et des notes générées
   depuis les commits. Si l'une échoue, rien n'est publié : corriger, puis supprimer et
   repousser le tag.
