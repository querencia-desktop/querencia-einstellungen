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

use crate::i18n::{gettext, gettext_f};
use crate::logo;

pub struct Bereich {
    /// Fester Name der Stapelseite (unabhängig von der Sprache).
    pub id: &'static str,
    /// Beschriftung in der Seitenleiste (übersetzt).
    pub name: String,
    pub seite: gtk4::Widget,
}

pub fn alle(app: &gtk4::Application) -> Vec<Bereich> {
    vec![
        Bereich {
            id: "erscheinungsbild",
            name: gettext("Appearance"),
            seite: crate::labwc_seite::seite::<crate::erscheinungsbild::Erscheinungsbild>(),
        },
        Bereich {
            id: "hintergrundbild",
            name: gettext("Background Image"),
            seite: crate::hintergrund_seite::seite(app),
        },
        Bereich {
            id: "symbole",
            name: gettext(crate::arbeitsflaeche::NAME),
            seite: crate::arbeitsflaeche::seite(),
        },
        Bereich {
            id: "fensterdesign",
            name: gettext("Window Design"),
            seite: crate::labwc_seite::seite::<crate::fensterdesign::Fensterdesign>(),
        },
        Bereich {
            id: "schriften",
            name: gettext("Fonts"),
            seite: crate::labwc_seite::seite::<crate::schriften::Schriften>(),
        },
        Bereich {
            id: "panels",
            name: gettext("Panels"),
            seite: crate::panels::seite(),
        },
        Bereich {
            id: "tastatur",
            name: gettext("Keyboard"),
            seite: crate::labwc_seite::seite::<crate::tastatur::Tastatur>(),
        },
        Bereich {
            id: "maus",
            name: gettext("Mouse"),
            seite: crate::labwc_seite::seite::<crate::maus::Maus>(),
        },
        Bereich {
            id: "bildschirm",
            name: gettext("Screen"),
            seite: crate::bildschirm::seite(false),
        },
        Bereich {
            id: "arbeitsflaechen",
            name: gettext("Workspaces"),
            seite: crate::labwc_seite::seite::<crate::arbeitsflaechen::Arbeitsflaechen>(),
        },
        Bereich {
            id: "standardanwendungen",
            name: gettext(crate::standardanwendungen::NAME),
            seite: crate::standardanwendungen::seite(),
        },
        Bereich {
            id: "sprache",
            name: gettext(<crate::sprache::Sprache as crate::labwc_seite::Bereich>::NAME),
            seite: crate::labwc_seite::seite::<crate::sprache::Sprache>(),
        },
        Bereich {
            id: "informationen",
            name: gettext("About Querencia"),
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
fn werte(zeilen: &[(String, String)]) -> gtk4::Grid {
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
    let l = gtk4::Label::new(Some(&gettext_f("Source: {source}", &[("source", text)])));
    l.set_xalign(0.0);
    l.set_wrap(true);
    l.add_css_class("dim-label");
    l
}

// ------------------------------------------------------------------ Seiten

fn informationen() -> gtk4::Widget {
    let s = seite(&gettext("About Querencia"));
    let kopf = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    kopf.append(&logo::bild(64));
    let text = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    let name = gtk4::Label::new(None);
    name.set_markup("<b><big>Querencia Desktop</big></b>");
    name.set_xalign(0.0);
    text.append(&name);
    for zeile in [
        gettext_f(
            "Querencia Settings {version}",
            &[("version", env!("CARGO_PKG_VERSION"))],
        ),
        gettext("A classic desktop in the style of GNOME 2 for Wayland and labwc"),
        gettext_f(
            "License: {license}",
            &[("license", env!("CARGO_PKG_LICENSE"))],
        ),
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
        (gettext("System:"), ubuntu),
        (gettext("Compositor:"), "labwc 0.9.8".into()),
        (
            gettext("Desktop:"),
            std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        ),
        (
            "GTK:".into(),
            format!(
                "{}.{}.{}",
                gtk4::major_version(),
                gtk4::minor_version(),
                gtk4::micro_version()
            ),
        ),
    ]));
    s.append(&quelle(&gettext("Component versions: ~/opt/querencia/protokoll.txt; the panel dialog “About Querencia” (System menu) shows the same lines.")));
    s.upcast()
}
