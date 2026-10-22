"use strict";

/* Page locale d'Allkin Client : la liste des serveurs, leur recherche sur le
   réseau ou leur saisie, puis la fenêtre cédée au serveur choisi. Les
   commandes Rust sont dans src-tauri/src (state, connect, add, forget, scan,
   stop_scan, set_lang, set_theme). */

const STRINGS = {
  fr: {
    language: "Langue",
    toDark: "Passer en apparence sombre",
    toLight: "Passer en apparence claire",
    close: "Fermer",
    emptyTitle: "Aucun serveur enregistré",
    emptyText: "Lancez une recherche pour trouver les serveurs Allkin de votre réseau, ou saisissez l’adresse d’un serveur.",
    scan: "Lancer une recherche",
    manual: "Manuel",
    savedTitle: "Serveurs",
    lastUsed: "dernier utilisé",
    forget: "Oublier ce serveur",
    backHint: "Pour revenir ici depuis Allkin : menu Serveur › Changer de serveur…",
    scanTitle: "Recherche de serveurs",
    scanHint: "Sur cet ordinateur, le réseau local et les VPN (Tailscale, WireGuard, ZeroTier…).",
    preparing: "Préparation de la recherche…",
    scanning: "Analyse de {total} adresses ({networks}) : {done} / {total}",
    foundOne: "1 serveur Allkin trouvé.",
    foundMany: "{n} serveurs Allkin trouvés.",
    foundNone: "Aucun serveur Allkin trouvé.",
    stopped: "Recherche arrêtée.",
    scanBlocked: "macOS bloque l’accès au réseau local : autorisez « Allkin Client » (dans la demande affichée, ou Réglages Système › Confidentialité et sécurité › Réseau local), puis relancez la recherche.",
    stop: "Arrêter",
    rescan: "Relancer",
    addToList: "Ajouter à la liste",
    added: "Dans la liste",
    kinds: { local: "cet ordinateur", lan: "réseau local", vpn: "VPN", tailscale: "Tailscale" },
    manualTitle: "Ajouter un serveur",
    addressLabel: "Adresse du serveur",
    addressPlaceholder: "192.168.1.20, mon-serveur:9191 ou https://…",
    addressHint: "Sans port précisé, Allkin est cherché en HTTPS puis sur le port 9191.",
    cancel: "Annuler",
    add: "Ajouter",
    checking: "Vérification…",
    connectingTo: "Connexion à {name}…",
    failureTitle: "Impossible d’ouvrir {name}",
    retry: "Réessayer",
    choose: "Changer de serveur",
    invalid: "Cette adresse n’est pas valide.",
    notAllkin: "{address} répond, mais ce n’est pas un serveur Allkin.",
    reasons: {
      noRoute: "Aucune route vers le serveur : vérifiez le réseau ou le VPN.",
      noRouteMac: "Aucune route vers le serveur. S’il est sur le réseau local, autorisez « Allkin Client » à y accéder (demande de macOS, ou Réglages Système › Confidentialité et sécurité › Réseau local), puis réessayez.",
      dns: "Adresse introuvable : vérifiez le nom du serveur.",
      timeout: "Le serveur ne répond pas.",
      refused: "Le serveur refuse la connexion : Allkin est-il démarré ?",
      tls: "Connexion sécurisée impossible : le certificat du serveur n’est pas reconnu.",
      other: "Impossible de joindre le serveur.",
    },
  },
  en: {
    language: "Language",
    toDark: "Switch to dark appearance",
    toLight: "Switch to light appearance",
    close: "Close",
    emptyTitle: "No saved server",
    emptyText: "Search for the Allkin servers on your network, or enter a server’s address.",
    scan: "Start a search",
    manual: "Manual",
    savedTitle: "Servers",
    lastUsed: "last used",
    forget: "Forget this server",
    backHint: "To come back here from Allkin: Server menu › Change Server…",
    scanTitle: "Server search",
    scanHint: "On this computer, the local network and VPNs (Tailscale, WireGuard, ZeroTier…).",
    preparing: "Preparing the search…",
    scanning: "Checking {total} addresses ({networks}): {done} / {total}",
    foundOne: "1 Allkin server found.",
    foundMany: "{n} Allkin servers found.",
    foundNone: "No Allkin server found.",
    stopped: "Search stopped.",
    scanBlocked: "macOS is blocking local network access: allow “Allkin Client” (in the prompt, or System Settings › Privacy & Security › Local Network), then search again.",
    stop: "Stop",
    rescan: "Search again",
    addToList: "Add to the list",
    added: "In the list",
    kinds: { local: "this computer", lan: "local network", vpn: "VPN", tailscale: "Tailscale" },
    manualTitle: "Add a server",
    addressLabel: "Server address",
    addressPlaceholder: "192.168.1.20, my-server:9191 or https://…",
    addressHint: "Without a port, Allkin is looked for over HTTPS, then on port 9191.",
    cancel: "Cancel",
    add: "Add",
    checking: "Checking…",
    connectingTo: "Connecting to {name}…",
    failureTitle: "Can’t open {name}",
    retry: "Try again",
    choose: "Change server",
    invalid: "This address is not valid.",
    notAllkin: "{address} answers, but it isn’t an Allkin server.",
    reasons: {
      noRoute: "No route to the server: check the network or the VPN.",
      noRouteMac: "No route to the server. If it’s on the local network, allow “Allkin Client” to access it (macOS prompt, or System Settings › Privacy & Security › Local Network), then try again.",
      dns: "Address not found: check the server name.",
      timeout: "The server doesn’t answer.",
      refused: "The server refuses the connection: is Allkin running?",
      tls: "No secure connection: the server’s certificate isn’t trusted.",
      other: "Can’t reach the server.",
    },
  },
  es: {
    language: "Idioma",
    toDark: "Cambiar a apariencia oscura",
    toLight: "Cambiar a apariencia clara",
    close: "Cerrar",
    emptyTitle: "Ningún servidor guardado",
    emptyText: "Busque los servidores Allkin de su red o introduzca la dirección de un servidor.",
    scan: "Iniciar una búsqueda",
    manual: "Manual",
    savedTitle: "Servidores",
    lastUsed: "último usado",
    forget: "Olvidar este servidor",
    backHint: "Para volver aquí desde Allkin: menú Servidor › Cambiar de servidor…",
    scanTitle: "Búsqueda de servidores",
    scanHint: "En este ordenador, la red local y las VPN (Tailscale, WireGuard, ZeroTier…).",
    preparing: "Preparando la búsqueda…",
    scanning: "Analizando {total} direcciones ({networks}): {done} / {total}",
    foundOne: "1 servidor Allkin encontrado.",
    foundMany: "{n} servidores Allkin encontrados.",
    foundNone: "No se ha encontrado ningún servidor Allkin.",
    stopped: "Búsqueda detenida.",
    scanBlocked: "macOS bloquea el acceso a la red local: permita «Allkin Client» (en el aviso, o en Ajustes del Sistema › Privacidad y seguridad › Red local) y vuelva a buscar.",
    stop: "Detener",
    rescan: "Buscar de nuevo",
    addToList: "Añadir a la lista",
    added: "En la lista",
    kinds: { local: "este ordenador", lan: "red local", vpn: "VPN", tailscale: "Tailscale" },
    manualTitle: "Añadir un servidor",
    addressLabel: "Dirección del servidor",
    addressPlaceholder: "192.168.1.20, mi-servidor:9191 o https://…",
    addressHint: "Sin puerto, Allkin se busca por HTTPS y luego en el puerto 9191.",
    cancel: "Cancelar",
    add: "Añadir",
    checking: "Comprobando…",
    connectingTo: "Conectando con {name}…",
    failureTitle: "No se puede abrir {name}",
    retry: "Reintentar",
    choose: "Cambiar de servidor",
    invalid: "Esta dirección no es válida.",
    notAllkin: "{address} responde, pero no es un servidor Allkin.",
    reasons: {
      noRoute: "No hay ruta hacia el servidor: compruebe la red o la VPN.",
      noRouteMac: "No hay ruta hacia el servidor. Si está en la red local, permita que «Allkin Client» acceda a ella (aviso de macOS, o Ajustes del Sistema › Privacidad y seguridad › Red local) y vuelva a intentarlo.",
      dns: "Dirección no encontrada: compruebe el nombre del servidor.",
      timeout: "El servidor no responde.",
      refused: "El servidor rechaza la conexión: ¿está Allkin en marcha?",
      tls: "Conexión segura imposible: el certificado del servidor no es de confianza.",
      other: "No se puede contactar con el servidor.",
    },
  },
  de: {
    language: "Sprache",
    toDark: "Zur dunklen Darstellung wechseln",
    toLight: "Zur hellen Darstellung wechseln",
    close: "Schließen",
    emptyTitle: "Kein gespeicherter Server",
    emptyText: "Suchen Sie die Allkin-Server in Ihrem Netzwerk oder geben Sie die Adresse eines Servers ein.",
    scan: "Suche starten",
    manual: "Manuell",
    savedTitle: "Server",
    lastUsed: "zuletzt verwendet",
    forget: "Diesen Server vergessen",
    backHint: "Zurück hierher aus Allkin: Menü Server › Server wechseln …",
    scanTitle: "Serversuche",
    scanHint: "Auf diesem Computer, im lokalen Netzwerk und in VPNs (Tailscale, WireGuard, ZeroTier …).",
    preparing: "Suche wird vorbereitet …",
    scanning: "{total} Adressen werden geprüft ({networks}): {done} / {total}",
    foundOne: "1 Allkin-Server gefunden.",
    foundMany: "{n} Allkin-Server gefunden.",
    foundNone: "Kein Allkin-Server gefunden.",
    stopped: "Suche gestoppt.",
    scanBlocked: "macOS blockiert den Zugriff auf das lokale Netzwerk: Erlauben Sie „Allkin Client“ (in der Abfrage oder unter Systemeinstellungen › Datenschutz & Sicherheit › Lokales Netzwerk) und suchen Sie erneut.",
    stop: "Stoppen",
    rescan: "Erneut suchen",
    addToList: "Zur Liste hinzufügen",
    added: "In der Liste",
    kinds: { local: "dieser Computer", lan: "lokales Netzwerk", vpn: "VPN", tailscale: "Tailscale" },
    manualTitle: "Server hinzufügen",
    addressLabel: "Serveradresse",
    addressPlaceholder: "192.168.1.20, mein-server:9191 oder https://…",
    addressHint: "Ohne Port wird Allkin per HTTPS und dann auf Port 9191 gesucht.",
    cancel: "Abbrechen",
    add: "Hinzufügen",
    checking: "Wird geprüft …",
    connectingTo: "Verbindung mit {name} …",
    failureTitle: "{name} lässt sich nicht öffnen",
    retry: "Erneut versuchen",
    choose: "Server wechseln",
    invalid: "Diese Adresse ist ungültig.",
    notAllkin: "{address} antwortet, ist aber kein Allkin-Server.",
    reasons: {
      noRoute: "Keine Route zum Server: Prüfen Sie das Netzwerk oder das VPN.",
      noRouteMac: "Keine Route zum Server. Wenn er im lokalen Netzwerk ist, erlauben Sie „Allkin Client“ den Zugriff (Abfrage von macOS oder Systemeinstellungen › Datenschutz & Sicherheit › Lokales Netzwerk) und versuchen Sie es erneut.",
      dns: "Adresse nicht gefunden: Prüfen Sie den Servernamen.",
      timeout: "Der Server antwortet nicht.",
      refused: "Der Server verweigert die Verbindung: Läuft Allkin?",
      tls: "Keine sichere Verbindung: Dem Zertifikat des Servers wird nicht vertraut.",
      other: "Der Server ist nicht erreichbar.",
    },
  },
};

const ICONS = {
  plus: '<path d="M12 5v14M5 12h14"/>',
  check: '<path d="m5 12 5 5 9-10"/>',
  spinner: '<path d="M21 12a9 9 0 1 1-6.2-8.6"/>',
  x: '<path d="M6 6l12 12M18 6 6 18"/>',
};

const { invoke, Channel } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);
const fill = (text, values) => text.replace(/\{(\w+)\}/g, (_, key) => values[key] ?? "");
const short = (url) => url.trim().replace(/^https?:\/\//, "").replace(/\/$/, "");
const capitalize = (text) => text.charAt(0).toUpperCase() + text.slice(1);
const systemDark = matchMedia("(prefers-color-scheme: dark)");

let t = STRINGS.en;
let view = { lang: "en", os: "", theme: null, servers: [], current: null };
/** L'écran affiché et ce qu'il montre, pour le redessiner dans une autre langue. */
let screen = { name: "home" };
/** Chaque connexion lancée périme la précédente (Annuler, Réessayer). */
let attempt = 0;
/** La dernière recherche : progression, serveurs trouvés, ajouts en cours. */
let scan = null;
/** Le serveur qui vient d'entrer dans la liste, mis en évidence un instant. */
let fresh = null;

async function init() {
  try {
    view = await invoke("state");
  } catch {
    // Sans état, la page reste utilisable : rien n'est rouvert, c'est tout.
  }
  t = STRINGS[view.lang] ?? STRINGS.en;
  applyTheme(view.theme ?? (systemDark.matches ? "dark" : "light"));

  systemDark.addEventListener("change", () => {
    if (view.theme) return;
    applyTheme(systemDark.matches ? "dark" : "light");
    render();
  });
  $("lang").addEventListener("change", () => {
    view.lang = $("lang").value;
    t = STRINGS[view.lang];
    render();
    invoke("set_lang", { lang: view.lang });
  });
  $("theme").addEventListener("click", () => {
    view.theme = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
    applyTheme(view.theme);
    render();
    invoke("set_theme", { theme: view.theme });
  });
  document.addEventListener("click", (event) => {
    const action = event.target.closest("[data-action]")?.dataset.action;
    if (action === "scan") openScan();
    if (action === "manual") openManual();
    if (event.target.closest("[data-close]")) event.target.closest("dialog").close();
  });
  for (const dialog of document.querySelectorAll("dialog")) {
    // Un clic sur le fond sombre, autour de la fenêtre, la ferme.
    dialog.addEventListener("click", (event) => {
      if (event.target === dialog) dialog.close();
    });
  }
  $("scan-dialog").addEventListener("close", () => {
    if (scan?.running) invoke("stop_scan");
  });
  $("scan-toggle").addEventListener("click", () => (scan?.running ? invoke("stop_scan") : startScan()));
  $("manual-form").addEventListener("submit", addManual);
  $("connecting-cancel").addEventListener("click", home);
  $("retry").addEventListener("click", () => connect(screen.server));
  $("choose").addEventListener("click", home);
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && screen.name === "connecting") home();
  });

  // Au lancement, le dernier serveur se rouvre ; « #servers » (menu Serveur ›
  // Changer de serveur) montre la liste à la place.
  const last = view.servers.find((server) => server.url === view.current);
  if (last && location.hash !== "#servers") connect(last);
  else home();
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
}

function show(name, details = {}) {
  screen = { name, ...details };
  for (const id of ["connecting", "failure", "home"]) $(id).hidden = id !== name;
  render();
}

function home() {
  attempt++;
  show("home");
}

/** Tous les textes de la page, dans la langue choisie. */
function render() {
  document.documentElement.lang = view.lang;
  for (const element of document.querySelectorAll("[data-t]")) element.textContent = t[element.dataset.t];
  for (const element of document.querySelectorAll("[data-t-label]")) label(element, t[element.dataset.tLabel]);
  $("lang").value = view.lang;
  label($("theme"), document.documentElement.dataset.theme === "dark" ? t.toLight : t.toDark);
  $("address").placeholder = t.addressPlaceholder;
  if (screen.name === "connecting") {
    $("connecting-text").textContent = fill(t.connectingTo, { name: screen.server.name });
  }
  if (screen.name === "failure") {
    $("failure-title").textContent = fill(t.failureTitle, { name: screen.server.name });
    $("failure-detail").textContent = explain(screen.outcome, short(screen.server.url));
  }
  renderServers();
  renderProgress();
  renderFound();
}

function label(element, text) {
  element.setAttribute("aria-label", text);
  element.title = text;
}

function explain(outcome, address) {
  if (outcome.status === "invalid") return t.invalid;
  if (outcome.status === "notAllkin") return fill(t.notAllkin, { address });
  if (outcome.reason === "noRoute" && view.os === "macos") return t.reasons.noRouteMac;
  return t.reasons[outcome.reason] ?? t.reasons.other;
}

function cell(className, text) {
  const span = document.createElement("span");
  span.className = className;
  span.textContent = text ?? "";
  return span;
}

function iconButton(name, className, text) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = `icon-button ${className}`;
  button.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true">${ICONS[name]}</svg>`;
  label(button, text);
  return button;
}

/* ---- Serveurs enregistrés ---- */

function renderServers() {
  const empty = view.servers.length === 0;
  $("empty").hidden = !empty;
  $("servers").hidden = empty;
  $("saved-list").replaceChildren(
    ...view.servers.map((server) => {
      const item = document.createElement("li");
      if (server.url === fresh) item.className = "fresh";
      const open = document.createElement("button");
      open.type = "button";
      open.className = "server";
      open.append(
        cell("name", server.name),
        cell("address", short(server.url)),
        cell("badge", server.url === view.current ? t.lastUsed : ""),
      );
      open.addEventListener("click", () => connect(server));
      const forget = iconButton("x", "forget", `${t.forget} (${server.name})`);
      forget.addEventListener("click", async () => {
        await invoke("forget", { url: server.url });
        view.servers = view.servers.filter((other) => other.url !== server.url);
        if (view.current === server.url) view.current = null;
        renderServers();
        renderFound();
      });
      item.append(open, forget);
      return item;
    }),
  );
}

/** Un serveur vient d'être ajouté : il apparaît dans la liste, mis en évidence. */
function added({ url, name }) {
  if (!view.servers.some((server) => server.url === url)) view.servers.push({ url, name, lastUsed: 0 });
  fresh = url;
  renderServers();
  fresh = null;
}

/**
 * Ouvrir un serveur : s'il répond et que c'est toujours Allkin, la fenêtre
 * passe à sa page.
 */
async function connect(server) {
  show("connecting", { server });
  const mine = ++attempt;
  const outcome = await invoke("connect", { address: server.url, name: server.name });
  if (mine !== attempt) return;
  if (outcome.status === "ready") {
    // Pas de location.replace : macOS enverrait le serveur à sa webapp Safari
    // s'il en a une (voir `navigate`, src-tauri/src/net.rs).
    invoke("navigate", { url: outcome.url });
    return;
  }
  show("failure", { server, outcome });
  $("retry").focus();
}

/* ---- Saisie manuelle ---- */

function openManual() {
  $("address").value = "";
  manualError(null);
  $("manual-dialog").showModal();
  $("address").focus();
}

function manualError(text) {
  $("manual-error").hidden = !text;
  $("manual-error").textContent = text ?? "";
}

async function addManual(event) {
  event.preventDefault();
  const address = $("address").value.trim();
  if (!address) return;
  const submit = $("manual-submit");
  manualError(null);
  submit.disabled = true;
  submit.textContent = t.checking;
  const outcome = await invoke("add", { address, name: null });
  submit.disabled = false;
  submit.textContent = t.add;
  // Une adresse qui répond sans être Allkin n'entre pas dans la liste.
  if (outcome.status !== "ready") {
    manualError(explain(outcome, short(address)));
    $("address").focus();
    return;
  }
  $("manual-dialog").close();
  added(outcome);
}

/* ---- Recherche sur le réseau ---- */

function openScan() {
  $("scan-dialog").showModal();
  startScan();
}

function startScan() {
  const current = (scan = {
    running: true,
    total: 0,
    done: 0,
    networks: [],
    found: [],
    adding: new Set(),
    cancelled: false,
    blocked: false,
    error: null,
  });
  const channel = new Channel();
  channel.onmessage = (message) => {
    if (scan === current) onScan(message);
  };
  renderProgress();
  renderFound();
  invoke("scan", { onEvent: channel });
}

function onScan(message) {
  switch (message.event) {
    case "started":
      scan.total = message.total;
      scan.networks = message.networks;
      break;
    case "progress":
      scan.done = message.done;
      break;
    case "found":
      if (scan.found.some((server) => server.url === message.server.url)) return;
      scan.found.push(message.server);
      renderFound();
      break;
    case "finished":
      scan.running = false;
      scan.cancelled = message.cancelled;
      scan.blocked = message.blocked;
      break;
  }
  renderProgress();
}

function networkName(network) {
  const kind = t.kinds[network.kind];
  return network.name && network.kind !== "local" ? `${kind} ${network.name}` : kind;
}

function renderProgress() {
  if (!scan) return;
  const bar = $("scan-bar");
  if (scan.running && !scan.total) bar.removeAttribute("value");
  else bar.value = scan.total ? scan.done / scan.total : 1;

  let status;
  if (scan.running) {
    status = scan.total
      ? fill(t.scanning, { total: scan.total, done: scan.done, networks: scan.networks.map(networkName).join(", ") })
      : t.preparing;
  } else {
    const count = scan.found.length;
    status = scan.cancelled
      ? t.stopped
      : count === 0
        ? t.foundNone
        : count === 1
          ? t.foundOne
          : fill(t.foundMany, { n: count });
    if (scan.blocked && view.os === "macos") status += ` ${t.scanBlocked}`;
  }
  if (scan.error) status += ` ${scan.error}`;
  $("scan-status").textContent = status;
  $("scan-toggle").textContent = scan.running ? t.stop : t.rescan;
}

/** Les serveurs trouvés, chacun avec son « + » pour entrer dans la liste. */
function renderFound() {
  if (!scan) return;
  $("found-list").replaceChildren(
    ...scan.found.map((server) => {
      const local = server.kind === "local";
      const name = server.name || (local ? capitalize(t.kinds.local) : null);
      const saved = view.servers.some((other) => other.url === server.url);
      const busy = scan.adding.has(server.url);
      const item = document.createElement("li");
      const row = document.createElement("div");
      row.className = "row";
      const add = iconButton(saved ? "check" : busy ? "spinner" : "plus", busy ? "add busy" : "add", saved ? t.added : t.addToList);
      add.disabled = saved || busy;
      add.addEventListener("click", () => addFound(server, name));
      row.append(
        cell("name", name || short(server.url)),
        cell("address", name ? short(server.url) : ""),
        cell("badge", local ? "" : t.kinds[server.kind]),
        add,
      );
      item.append(row);
      return item;
    }),
  );
}

async function addFound(server, name) {
  const current = scan;
  current.adding.add(server.url);
  current.error = null;
  renderFound();
  const outcome = await invoke("add", { address: server.url, name });
  current.adding.delete(server.url);
  if (outcome.status === "ready") added(outcome);
  else current.error = explain(outcome, short(server.url));
  if (scan !== current) return;
  renderFound();
  renderProgress();
}

init();
