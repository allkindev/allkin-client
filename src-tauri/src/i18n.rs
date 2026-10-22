//! Langues (les quatre d'Allkin) et textes des menus. Ceux de la page locale
//! sont dans `ui/app.js`.

const LANGS: [&str; 4] = ["fr", "en", "es", "de"];

/// La langue choisie si elle est prise en charge, sinon celle du système,
/// sinon l'anglais.
pub fn resolve(chosen: Option<&str>) -> &'static str {
    let supported = |code: &str| -> Option<&'static str> { LANGS.into_iter().find(|lang| *lang == code) };
    let system = sys_locale::get_locale().unwrap_or_default().to_ascii_lowercase();
    chosen
        .and_then(supported)
        .or_else(|| supported(system.get(..2)?))
        .unwrap_or("en")
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub struct Menu {
    pub server: &'static str,
    pub servers: &'static str,
    pub reload: &'static str,
    pub browser: &'static str,
    pub view: &'static str,
    pub zoom_in: &'static str,
    pub zoom_out: &'static str,
    pub actual_size: &'static str,
    pub fullscreen: &'static str,
    pub quit: &'static str,
    // macOS seulement
    pub about: &'static str,
    pub services: &'static str,
    pub hide: &'static str,
    pub hide_others: &'static str,
    pub show_all: &'static str,
    pub edit: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
    pub cut: &'static str,
    pub copy: &'static str,
    pub paste: &'static str,
    pub select_all: &'static str,
    pub window: &'static str,
    pub minimize: &'static str,
    pub zoom: &'static str,
    pub front: &'static str,
}

const MAC: bool = cfg!(target_os = "macos");

pub fn menu(lang: &str) -> Menu {
    match lang {
        "fr" => Menu {
            server: "Serveur",
            servers: "Changer de serveur…",
            reload: "Recharger la page",
            browser: "Ouvrir dans le navigateur",
            view: if MAC { "Présentation" } else { "Affichage" },
            zoom_in: "Zoom avant",
            zoom_out: "Zoom arrière",
            actual_size: "Taille réelle",
            fullscreen: "Plein écran",
            quit: if MAC { "Quitter Allkin Client" } else { "Quitter" },
            about: "À propos d’Allkin Client",
            services: "Services",
            hide: "Masquer Allkin Client",
            hide_others: "Masquer les autres",
            show_all: "Tout afficher",
            edit: "Édition",
            undo: "Annuler",
            redo: "Rétablir",
            cut: "Couper",
            copy: "Copier",
            paste: "Coller",
            select_all: "Tout sélectionner",
            window: "Fenêtre",
            minimize: "Placer dans le Dock",
            zoom: "Réduire/agrandir",
            front: "Tout ramener au premier plan",
        },
        "es" => Menu {
            server: "Servidor",
            servers: "Cambiar de servidor…",
            reload: "Volver a cargar la página",
            browser: "Abrir en el navegador",
            view: if MAC { "Visualización" } else { "Ver" },
            zoom_in: "Ampliar",
            zoom_out: "Reducir",
            actual_size: "Tamaño real",
            fullscreen: "Pantalla completa",
            quit: if MAC { "Salir de Allkin Client" } else { "Salir" },
            about: "Acerca de Allkin Client",
            services: "Servicios",
            hide: "Ocultar Allkin Client",
            hide_others: "Ocultar otros",
            show_all: "Mostrar todo",
            edit: "Edición",
            undo: "Deshacer",
            redo: "Rehacer",
            cut: "Cortar",
            copy: "Copiar",
            paste: "Pegar",
            select_all: "Seleccionar todo",
            window: "Ventana",
            minimize: "Minimizar",
            zoom: "Zoom",
            front: "Traer todo al frente",
        },
        "de" => Menu {
            server: "Server",
            servers: "Server wechseln …",
            reload: "Seite neu laden",
            browser: "Im Browser öffnen",
            view: if MAC { "Darstellung" } else { "Ansicht" },
            zoom_in: "Vergrößern",
            zoom_out: "Verkleinern",
            actual_size: "Originalgröße",
            fullscreen: "Vollbild",
            quit: if MAC { "Allkin Client beenden" } else { "Beenden" },
            about: "Über Allkin Client",
            services: "Dienste",
            hide: "Allkin Client ausblenden",
            hide_others: "Andere ausblenden",
            show_all: "Alle einblenden",
            edit: "Bearbeiten",
            undo: "Widerrufen",
            redo: "Wiederholen",
            cut: "Ausschneiden",
            copy: "Kopieren",
            paste: "Einsetzen",
            select_all: "Alles auswählen",
            window: "Fenster",
            minimize: "Im Dock ablegen",
            zoom: "Zoomen",
            front: "Alle nach vorne bringen",
        },
        _ => Menu {
            server: "Server",
            servers: "Change Server…",
            reload: "Reload Page",
            browser: "Open in Browser",
            view: "View",
            zoom_in: "Zoom In",
            zoom_out: "Zoom Out",
            actual_size: "Actual Size",
            fullscreen: "Full Screen",
            quit: if MAC { "Quit Allkin Client" } else { "Quit" },
            about: "About Allkin Client",
            services: "Services",
            hide: "Hide Allkin Client",
            hide_others: "Hide Others",
            show_all: "Show All",
            edit: "Edit",
            undo: "Undo",
            redo: "Redo",
            cut: "Cut",
            copy: "Copy",
            paste: "Paste",
            select_all: "Select All",
            window: "Window",
            minimize: "Minimize",
            zoom: "Zoom",
            front: "Bring All to Front",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_supported_choice_wins() {
        assert_eq!(resolve(Some("de")), "de");
    }

    #[test]
    fn an_unsupported_choice_falls_back_to_a_supported_language() {
        assert!(LANGS.contains(&resolve(Some("it"))));
        assert!(LANGS.contains(&resolve(None)));
    }
}
