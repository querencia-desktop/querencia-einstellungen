//! Die Bereiche des Kontrollzentrums. Jede Seite zeigt den tatsächlichen
//! Ist-Zustand aus ihrer Quelle und ändert ihn, wo es einen Schreibweg gibt;
//! Schaltflächen, die nichts tun, gibt es nicht. „Hintergrundbild“ (QE-01,
//! Galerie G2-AF/G2-AJ), „Bildschirm“ (G2-G), „Symbole der Arbeitsfläche“
//! (G2-X), „Standardanwendungen“ (G2-Z), „Fensterdesign“, „Tastatur“,
//! „Maus“, „Arbeitsflächen“ (G2-AG, Baustein `labwc_konfig`) sowie
//! „Erscheinungsbild“ und „Schriften“ (G2-AJ, gsettings und labwc). „Panels“
//! zeigt den Aufbau und erklärt die Bedienung im Panel selbst, „Netzwerk“
//! verweist auf das Netzwerksymbol im Panel.
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
            id: "netzwerk",
            name: gettext("Network"),
            seite: netzwerk(),
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

/// Fußzeile einer Seite: wo die angezeigten Werte gespeichert sind. Klein
/// und grau, immer die letzte Zeile des Seiteninhalts (über der Knopfleiste).
pub(crate) fn quelle(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(None);
    l.set_markup(&format!(
        "<small>{}</small>",
        glib::markup_escape_text(&gettext_f("Source: {source}", &[("source", text)]))
    ));
    l.set_xalign(0.0);
    l.set_wrap(true);
    l.set_max_width_chars(70);
    l.set_margin_top(6);
    l.add_css_class("dim-label");
    l
}

/// Öffnet `adresse` im Standardbrowser (GtkUriLauncher, über das Portal oder
/// GIO); schlägt das fehl, steht der Grund in `meldung`.
fn adresse_oeffnen(neben: &impl IsA<gtk4::Widget>, adresse: &str, meldung: &gtk4::Label) {
    let fenster = neben.root().and_downcast::<gtk4::Window>();
    let meldung = meldung.clone();
    let a = adresse.to_string();
    gtk4::UriLauncher::new(adresse).launch(
        fenster.as_ref(),
        gtk4::gio::Cancellable::NONE,
        move |ergebnis| {
            if let Err(e) = ergebnis {
                meldung.set_text(&gettext_f(
                    "Could not open {address}: {error}",
                    &[("address", &a), ("error", &e.to_string())],
                ));
                meldung.set_visible(true);
            }
        },
    );
}

// ------------------------------------------------------------------ Seiten

fn informationen() -> gtk4::Widget {
    let produkt = crate::produkt::Produkt::lesen();
    let s = seite(&gettext("About Querencia"));
    let kopf = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    kopf.append(&logo::bild(64));
    let text = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    let name = gtk4::Label::new(None);
    // Produktversion nur, wenn das Metapaket sie liefert
    let titel = match &produkt.version {
        Some(v) => format!("{} {v}", produkt.name()),
        None => produkt.name().to_string(),
    };
    name.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(&titel)
    ));
    name.set_xalign(0.0);
    name.set_selectable(true);
    text.append(&name);
    for zeile in [
        gettext("A classic desktop in the style of GNOME 2 for Wayland and labwc"),
        gettext_f(
            "Querencia Settings {version}",
            &[("version", env!("CARGO_PKG_VERSION"))],
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
    let zeilen = [
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
        (gettext("License:"), env!("CARGO_PKG_LICENSE").into()),
    ];
    let raster = werte(&zeilen);
    let meldung = gtk4::Label::new(None);
    meldung.set_xalign(0.0);
    meldung.set_wrap(true);
    meldung.set_max_width_chars(70);
    meldung.set_selectable(true);
    meldung.set_visible(false);

    let kl = gtk4::Label::new(Some(&gettext("Website:")));
    kl.set_xalign(1.0);
    kl.add_css_class("dim-label");
    let website = produkt.website().to_string();
    let link = gtk4::Label::new(None);
    let w = glib::markup_escape_text(&website);
    link.set_markup(&format!("<a href=\"{w}\">{w}</a>"));
    link.set_xalign(0.0);
    let m = meldung.clone();
    link.connect_activate_link(move |l, adresse| {
        adresse_oeffnen(l, adresse, &m);
        glib::Propagation::Stop
    });
    raster.attach(&kl, 0, zeilen.len() as i32, 1, 1);
    raster.attach(&link, 1, zeilen.len() as i32, 1, 1);
    s.append(&raster);

    let knoepfe = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    knoepfe.set_margin_top(6);
    for (beschriftung, adresse) in [
        (gettext("Online Help"), produkt.hilfe()),
        (
            gettext("Report a Problem"),
            produkt.fehler_melden().to_string(),
        ),
    ] {
        let k = gtk4::Button::with_label(&beschriftung);
        k.set_tooltip_text(Some(&adresse));
        let m = meldung.clone();
        k.connect_clicked(move |k| adresse_oeffnen(k, &adresse, &m));
        knoepfe.append(&k);
    }
    s.append(&knoepfe);
    s.append(&meldung);
    s.upcast()
}

/// Ziel von „Netzwerkeinstellungen …“ im Netzwerksymbol des Panels
/// (Aktion `netzwerk` der Desktop-Datei): erklärt, wo Verbindungen
/// eingerichtet werden. Eigene Netzwerkeinstellungen hat das Programm nicht.
fn netzwerk() -> gtk4::Widget {
    let s = seite(&gettext("Network"));
    // Menüpunkte mit geschütztem Leerzeichen vor „…“ (kein Umbruch davor)
    for text in [
        gettext(
            "Network connections are set up in the panel: a click on the network icon shows the wired connections and the wireless networks in range. Choosing a network there connects to it; for protected networks the panel asks for the password.",
        ),
        gettext(
            "The same menu switches wireless networking and airplane mode on and off. “Connection Information\u{a0}…” shows the address and other details of the active connection.",
        ),
        gettext(
            "If the network icon is missing: right-click a free spot in the panel → “Add to Panel\u{a0}…” → Network.",
        ),
    ] {
        let l = gtk4::Label::new(Some(&text));
        l.set_xalign(0.0);
        l.set_wrap(true);
        l.set_max_width_chars(70);
        s.append(&l);
    }
    let spaeter = gtk4::Label::new(Some(&gettext(
        "Without NetworkManager the panel only shows the state of the network. Settings for addresses, VPN and proxy are not part of Querencia yet.",
    )));
    spaeter.set_xalign(0.0);
    spaeter.set_wrap(true);
    spaeter.set_max_width_chars(70);
    spaeter.add_css_class("dim-label");
    s.append(&spaeter);
    s.upcast()
}
