//! Die Bereiche des Kontrollzentrums (Phase 2, Prototyp). Jede Seite zeigt den
//! tatsächlichen Ist-Zustand aus der jeweiligen Quelle, nur lesend. Was noch
//! kein Backend hat, steht als „Noch nicht verfügbar“ mit dem Grund – ohne
//! Schaltflächen, die nichts tun. Funktionen: „Hintergrund …“ (QE-01),
//! „Bildschirm“ (G2-G), „Symbole der Arbeitsfläche“ (G2-X),
//! „Standardanwendungen“ (G2-Z) sowie „Fensterdesign“, „Tastatur“, „Maus“
//! und „Arbeitsflächen“ (G2-AG, über den Baustein `labwc_konfig`).
use gtk4::prelude::*;
use gtk4::{gio, glib};

use crate::{hintergrund, logo, modell};

pub struct Bereich {
    pub name: &'static str,
    pub seite: gtk4::Widget,
}

pub fn alle(app: &gtk4::Application) -> Vec<Bereich> {
    vec![
        Bereich {
            name: "Erscheinungsbild",
            seite: erscheinungsbild(app),
        },
        Bereich {
            name: "Hintergrundbild",
            seite: hintergrundbild(app),
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
            seite: schriften(),
        },
        Bereich {
            name: "Panels",
            seite: panels(),
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

fn noch_nicht(grund: &str) -> gtk4::Frame {
    let f = gtk4::Frame::new(Some("Noch nicht verfügbar"));
    let l = gtk4::Label::new(Some(grund));
    l.set_xalign(0.0);
    l.set_wrap(true);
    l.set_margin_top(6);
    l.set_margin_bottom(6);
    l.set_margin_start(8);
    l.set_margin_end(8);
    f.set_child(Some(&l));
    f
}

fn gsetting(schema: &str, key: &str) -> String {
    let quelle = gio::SettingsSchemaSource::default();
    let Some(schema_obj) = quelle.and_then(|s| s.lookup(schema, true)) else {
        return "(Schema fehlt)".into();
    };
    if !schema_obj.has_key(key) {
        return "(Schlüssel fehlt)".into();
    }
    let wert = gio::Settings::new(schema).value(key);
    wert.str()
        .map_or_else(|| wert.print(false).to_string(), str::to_string)
}

fn oder(v: Option<String>, standard: &str) -> String {
    v.unwrap_or_else(|| format!("{standard} (labwc-Standard, nicht gesetzt)"))
}

// ------------------------------------------------------------------ Seiten

fn hintergrund_knopf(app: &gtk4::Application) -> gtk4::Button {
    let knopf = gtk4::Button::with_label("Hintergrund …");
    knopf.set_halign(gtk4::Align::Start);
    let app = app.clone();
    knopf.connect_clicked(move |_| {
        hintergrund::oeffnen(&app).present();
    });
    knopf
}

fn hintergrund_ist() -> gtk4::Grid {
    let text = std::fs::read_to_string(hintergrund::konfig_pfad()).unwrap_or_default();
    let (h, _) = modell::Hintergrund::lesen(&text);
    werte(&[
        (
            "Typ:",
            match h.typ {
                modell::Typ::Bild => "Hintergrundbild".into(),
                modell::Typ::Farbe => "Einfarbig".into(),
            },
        ),
        (
            "Bild:",
            h.datei.as_ref().map_or_else(
                || "eingebaut (Querencia Blau)".into(),
                |p| p.display().to_string(),
            ),
        ),
        ("Darstellung:", h.darstellung.beschriftung().into()),
        ("Farbe:", h.farbe.hex()),
    ])
}

fn erscheinungsbild(app: &gtk4::Application) -> gtk4::Widget {
    let s = seite("Erscheinungsbild");
    let f = gtk4::Frame::new(Some("Hintergrund"));
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    b.set_margin_top(6);
    b.set_margin_bottom(6);
    b.set_margin_start(8);
    b.set_margin_end(8);
    b.append(&hintergrund_ist());
    b.append(&hintergrund_knopf(app));
    f.set_child(Some(&b));
    s.append(&f);
    s.append(&werte(&[
        (
            "Symbolthema (GTK):",
            gsetting("org.gnome.desktop.interface", "icon-theme"),
        ),
        (
            "Symbolthema Panel/Desktop:",
            "gnome-human (im Programm festgelegt, HMP-05)".into(),
        ),
        (
            "GTK-Thema:",
            gsetting("org.gnome.desktop.interface", "gtk-theme"),
        ),
        (
            "Farbschema:",
            gsetting("org.gnome.desktop.interface", "color-scheme"),
        ),
        (
            "Zeigerthema / -größe:",
            format!(
                "{} / {}",
                gsetting("org.gnome.desktop.interface", "cursor-theme"),
                gsetting("org.gnome.desktop.interface", "cursor-size")
            ),
        ),
    ]));
    s.append(&quelle("gsettings org.gnome.desktop.interface (dconf) über das GTK-Portal; Panel und Desktop setzen ihr Symbolthema selbst."));
    s.append(&noch_nicht("Symbol-, GTK- und Zeigerthema: Das Schreiben über gsettings ist möglich, aber Panel und Desktop überschreiben das Symbolthema beim Start; außerdem sind nur die GTK-Themen „Default“ und „Emacs“ und nur der Zeiger „Adwaita“ installiert. Vorschlag siehe Bericht."));
    s.upcast()
}

fn hintergrundbild(app: &gtk4::Application) -> gtk4::Widget {
    let s = seite("Hintergrundbild");
    s.append(&hintergrund_ist());
    s.append(&hintergrund_knopf(app));
    s.append(&quelle("~/.config/querencia/desktop.ini [Background]; Querencia Desktop zeichnet den Hintergrund und lädt die Datei bei Änderung sofort neu."));
    s.upcast()
}

fn schriften() -> gtk4::Widget {
    let s = seite("Schriften");
    // Titelleiste wie labwc: place="ActiveWindow", sonst <font> ohne place
    let rc = crate::labwc_konfig::lesen(&crate::labwc_konfig::Orte::standard()).rc;
    let schrift = |k: &str| {
        crate::labwc_xml::wert(&rc, crate::labwc_xml::SCHRIFT_AKTIV, k)
            .or_else(|| crate::labwc_xml::wert(&rc, crate::labwc_xml::SCHRIFT_ALLE, k))
    };
    let titel = (schrift("name").is_some() || schrift("size").is_some()).then(|| {
        format!(
            "{} {}",
            schrift("name").unwrap_or_else(|| "sans".into()),
            schrift("size").unwrap_or_else(|| "10".into())
        )
    });
    s.append(&werte(&[
        (
            "Anwendungen (GTK):",
            gsetting("org.gnome.desktop.interface", "font-name"),
        ),
        (
            "Dokumente:",
            gsetting("org.gnome.desktop.interface", "document-font-name"),
        ),
        (
            "Dicktengleich:",
            gsetting("org.gnome.desktop.interface", "monospace-font-name"),
        ),
        ("Fenstertitel (labwc):", oder(titel, "sans 10")),
    ]));
    s.append(&quelle("gsettings org.gnome.desktop.interface; rc.xml <theme><font place=\"ActiveWindow\">. Installiert sind nur DejaVu Sans/Serif/Mono und Noto Color Emoji."));
    s.append(&noch_nicht("GTK-Schriften über gsettings wären sofort wirksam. Die Schrift der Titelleiste lässt sich unter „Fensterdesign“ ändern."));
    s.upcast()
}

fn panels() -> gtk4::Widget {
    let s = seite("Panels");
    let pfad = glib::user_config_dir().join("querencia").join("panel.ini");
    let kf = glib::KeyFile::new();
    let mut zeilen: Vec<(&str, String)> = Vec::new();
    match kf.load_from_file(&pfad, glib::KeyFileFlags::NONE) {
        Ok(()) => {
            let panels = kf
                .string_list("Layout", "Panels")
                .map(|l| l.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                .unwrap_or_default();
            zeilen.push(("Panels:", panels.join(", ")));
            for p in &panels {
                let gruppe = format!("Panel {p}");
                let kante = kf
                    .string(&gruppe, "Edge")
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                let hoehe = kf.integer(&gruppe, "Height").unwrap_or(0);
                let mut applets = Vec::new();
                for bereich in ["Start", "Center", "End"] {
                    if let Ok(l) = kf.string_list(&gruppe, bereich) {
                        let ids: Vec<String> = l
                            .iter()
                            .map(|s| s.to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !ids.is_empty() {
                            applets.push(format!("{bereich}: {}", ids.join(", ")));
                        }
                    }
                }
                zeilen.push((
                    "",
                    format!("{p}: Kante {kante}, Höhe {hoehe}; {}", applets.join(" · ")),
                ));
            }
        }
        Err(err) => zeilen.push((
            "panel.ini:",
            format!("nicht lesbar ({err}) – Panel nutzt das Standardlayout"),
        )),
    }
    s.append(&werte(&zeilen));
    s.append(&quelle(&format!(
        "{} (Formatversion 1; Editor im Panel: Rechtsklick → „Zum Panel hinzufügen …“)",
        pfad.display()
    )));
    s.append(&noch_nicht("Das Panel liest panel.ini nur beim Start und bietet keine Schnittstelle für Änderungen von außen (D-Bus: nur „quit“). Vorschlag: Panel-Layoutcode als gemeinsames Crate und eine „Neu laden“-Aktion im Panel."));
    s.upcast()
}

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
        format!(
            "Querencia Einstellungen {} (Prototyp)",
            env!("CARGO_PKG_VERSION")
        ),
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
