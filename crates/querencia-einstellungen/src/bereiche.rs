//! Die Bereiche des Kontrollzentrums. Jede Seite zeigt den tatsächlichen
//! Ist-Zustand aus ihrer Quelle und ändert ihn, wo es einen Schreibweg gibt;
//! Schaltflächen, die nichts tun, gibt es nicht. „Hintergrundbild“ (QE-01,
//! Galerie G2-AF/G2-AJ), „Bildschirm“ (G2-G), „Symbole der Arbeitsfläche“
//! (G2-X), „Standardanwendungen“ (G2-Z), „Fensterdesign“, „Tastatur“,
//! „Maus“, „Arbeitsflächen“ (G2-AG, Baustein `labwc_konfig`) sowie
//! „Erscheinungsbild“ und „Schriften“ (G2-AJ, gsettings und labwc). „Panels“
//! zeigt den Aufbau und erklärt die Bedienung im Panel selbst.
use gtk4::glib;
use gtk4::prelude::*;

use crate::logo;

pub struct Bereich {
    pub name: &'static str,
    pub seite: gtk4::Widget,
}

pub fn alle(app: &gtk4::Application) -> Vec<Bereich> {
    vec![
        Bereich {
            name: "Erscheinungsbild",
            seite: crate::labwc_seite::seite::<crate::erscheinungsbild::Erscheinungsbild>(),
        },
        Bereich {
            name: "Hintergrundbild",
            seite: crate::hintergrund_seite::seite(app),
        },
        Bereich {
            name: crate::arbeitsflaeche::NAME,
            seite: crate::arbeitsflaeche::seite(),
        },
        Bereich {
            name: "Fensterdesign",
            seite: crate::labwc_seite::seite::<crate::fensterdesign::Fensterdesign>(),
        },
        Bereich {
            name: "Schriften",
            seite: crate::labwc_seite::seite::<crate::schriften::Schriften>(),
        },
        Bereich {
            name: "Panels",
            seite: crate::panels::seite(),
        },
        Bereich {
            name: "Tastatur",
            seite: crate::labwc_seite::seite::<crate::tastatur::Tastatur>(),
        },
        Bereich {
            name: "Maus",
            seite: crate::labwc_seite::seite::<crate::maus::Maus>(),
        },
        Bereich {
            name: "Bildschirm",
            seite: crate::bildschirm::seite(false),
        },
        Bereich {
            name: "Arbeitsflächen",
            seite: crate::labwc_seite::seite::<crate::arbeitsflaechen::Arbeitsflaechen>(),
        },
        Bereich {
            name: crate::standardanwendungen::NAME,
            seite: crate::standardanwendungen::seite(),
        },
        Bereich {
            name: "Informationen über Querencia",
            seite: informationen(),
        },
    ]
}

// ------------------------------------------------------------------ Bausteine

pub(crate) fn seite(titel: &str) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    b.set_margin_top(12);
    b.set_margin_bottom(12);
    b.set_margin_start(14);
    b.set_margin_end(14);
    let ueberschrift = gtk4::Label::new(None);
    ueberschrift.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(titel)
    ));
    ueberschrift.set_xalign(0.0);
    b.append(&ueberschrift);
    b
}

/// Zweispaltige Ist-Werte (Beschriftung, Wert).
fn werte(zeilen: &[(&str, String)]) -> gtk4::Grid {
    let g = gtk4::Grid::new();
    g.set_row_spacing(4);
    g.set_column_spacing(12);
    for (i, (k, v)) in zeilen.iter().enumerate() {
        let kl = gtk4::Label::new(Some(k));
        kl.set_xalign(1.0);
        kl.add_css_class("dim-label");
        let vl = gtk4::Label::new(Some(v));
        vl.set_xalign(0.0);
        vl.set_selectable(true);
        vl.set_wrap(true);
        vl.set_hexpand(true);
        g.attach(&kl, 0, i as i32, 1, 1);
        g.attach(&vl, 1, i as i32, 1, 1);
    }
    g
}

pub(crate) fn quelle(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(&format!("Quelle: {text}")));
    l.set_xalign(0.0);
    l.set_wrap(true);
    l.add_css_class("dim-label");
    l
}

// ------------------------------------------------------------------ Seiten

fn informationen() -> gtk4::Widget {
    let s = seite("Informationen über Querencia");
    let kopf = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    kopf.append(&logo::bild(64));
    let text = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    let name = gtk4::Label::new(None);
    name.set_markup("<b><big>Querencia Desktop</big></b>");
    name.set_xalign(0.0);
    text.append(&name);
    for zeile in [
        format!("Querencia Einstellungen {}", env!("CARGO_PKG_VERSION")),
        "Ein klassischer Desktop im Stil von GNOME 2 für Wayland und labwc".to_string(),
        format!("Lizenz: {}", env!("CARGO_PKG_LICENSE")),
    ] {
        let l = gtk4::Label::new(Some(&zeile));
        l.set_xalign(0.0);
        text.append(&l);
    }
    kopf.append(&text);
    s.append(&kopf);
    let ubuntu = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.strip_prefix("PRETTY_NAME=")
                    .map(|v| v.trim_matches('"').to_string())
            })
        })
        .unwrap_or_default();
    s.append(&werte(&[
        ("System:", ubuntu),
        ("Compositor:", "labwc 0.9.8".into()),
        (
            "Desktop:",
            std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        ),
        (
            "GTK:",
            format!(
                "{}.{}.{}",
                gtk4::major_version(),
                gtk4::minor_version(),
                gtk4::micro_version()
            ),
        ),
    ]));
    s.append(&quelle("Versionen der Komponenten: ~/opt/querencia/protokoll.txt; der Panel-Dialog „Informationen über Querencia“ (System-Menü) zeigt dieselben Zeilen."));
    s.upcast()
}
