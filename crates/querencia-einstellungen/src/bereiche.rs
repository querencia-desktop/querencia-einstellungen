//! Die Bereiche des Kontrollzentrums (Phase 2, Prototyp). Jede Seite zeigt den
//! tatsächlichen Ist-Zustand aus der jeweiligen Quelle, nur lesend. Was noch
//! kein Backend hat, steht als „Noch nicht verfügbar“ mit dem Grund – ohne
//! Schaltflächen, die nichts tun. Funktionen: „Hintergrund …“ (QE-01),
//! „Bildschirm“ (G2-G) und „Arbeitsfläche“ (G2-X).
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
            name: "Arbeitsfläche",
            seite: crate::arbeitsflaeche::seite(),
        },
        Bereich {
            name: "Fensterdesign",
            seite: fensterdesign(),
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
            seite: tastatur(),
        },
        Bereich {
            name: "Maus",
            seite: maus(),
        },
        Bereich {
            name: "Bildschirm",
            seite: crate::bildschirm::seite(false),
        },
        Bereich {
            name: "Arbeitsflächen",
            seite: arbeitsflaechen(),
        },
        Bereich {
            name: "Standardanwendungen",
            seite: standardanwendungen(),
        },
        Bereich {
            name: "Informationen über Querencia",
            seite: informationen(),
        },
    ]
}

// ------------------------------------------------------------------ Bausteine

fn seite(titel: &str) -> gtk4::Box {
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

fn quelle(text: &str) -> gtk4::Label {
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

/// Der Konfigurationsordner des laufenden labwc (`-C`), aus /proc gelesen;
/// sonst `~/.config/labwc`.
fn labwc_konfigordner() -> std::path::PathBuf {
    if let Ok(eintraege) = std::fs::read_dir("/proc") {
        for e in eintraege.flatten() {
            let Ok(cmd) = std::fs::read(e.path().join("cmdline")) else {
                continue;
            };
            let teile: Vec<&[u8]> = cmd.split(|b| *b == 0).collect();
            if teile
                .first()
                .is_some_and(|p| p.ends_with(b"/labwc") || *p == b"labwc")
                && let Some(i) = teile.iter().position(|t| *t == b"-C")
                && let Some(dir) = teile.get(i + 1)
            {
                return std::path::PathBuf::from(String::from_utf8_lossy(dir).into_owned());
            }
        }
    }
    glib::user_config_dir().join("labwc")
}

struct RcXml {
    pfad: std::path::PathBuf,
    text: String,
}

fn rc_xml() -> RcXml {
    let pfad = labwc_konfigordner().join("rc.xml");
    let text = std::fs::read_to_string(&pfad).unwrap_or_default();
    RcXml { pfad, text }
}

impl RcXml {
    fn dokument(&self) -> Option<roxmltree::Document<'_>> {
        roxmltree::Document::parse(&self.text).ok()
    }
    /// Text des ersten Elements mit diesem Pfad unter der Wurzel, z. B. ["theme","name"].
    fn text(&self, pfad: &[&str]) -> Option<String> {
        let doc = self.dokument()?;
        let mut knoten = doc.root_element();
        for name in pfad {
            knoten = knoten.children().find(|c| c.has_tag_name(*name))?;
        }
        knoten.text().map(|t| t.trim().to_string())
    }
    fn attribut(&self, pfad: &[&str], attr: &str) -> Option<String> {
        let doc = self.dokument()?;
        let mut knoten = doc.root_element();
        for name in pfad {
            knoten = knoten.children().find(|c| c.has_tag_name(*name))?;
        }
        knoten.attribute(attr).map(str::to_string)
    }
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

fn fensterdesign() -> gtk4::Widget {
    let s = seite("Fensterdesign");
    let rc = rc_xml();
    let mut themen = Vec::new();
    for dir in [glib::user_data_dir().join("themes")].into_iter().chain(
        glib::system_data_dirs()
            .into_iter()
            .map(|d| d.join("themes")),
    ) {
        if let Ok(e) = std::fs::read_dir(&dir) {
            for t in e.flatten() {
                if t.path().join("labwc").join("themerc").is_file() {
                    themen.push(t.file_name().to_string_lossy().into_owned());
                }
            }
        }
    }
    themen.sort();
    themen.dedup();
    s.append(&werte(&[
        (
            "Aktives Thema:",
            oder(rc.text(&["theme", "name"]), "eingebaut"),
        ),
        (
            "Installierte labwc-Themen:",
            if themen.is_empty() {
                "keine".into()
            } else {
                themen.join(", ")
            },
        ),
        (
            "Eckenradius:",
            oder(rc.text(&["theme", "cornerRadius"]), "8"),
        ),
        ("Schatten:", oder(rc.text(&["theme", "dropShadows"]), "no")),
        (
            "Titelleistenknöpfe:",
            oder(
                rc.text(&["theme", "titlebar", "layout"]),
                "icon:iconify,max,close",
            ),
        ),
    ]));
    s.append(&quelle(&format!(
        "{} <theme>; Themen in ~/.local/share/themes/*/labwc/themerc",
        rc.pfad.display()
    )));
    s.append(&noch_nicht("Eine Änderung schreibt rc.xml im Runtime-Ordner (geschütztes Repository) und wird erst mit „labwc --reconfigure“ wirksam – beides braucht nach den Projektregeln eine Freigabe."));
    s.upcast()
}

fn schriften() -> gtk4::Widget {
    let s = seite("Schriften");
    let rc = rc_xml();
    let titel = rc.dokument().and_then(|d| {
        d.root_element()
            .children()
            .find(|c| c.has_tag_name("theme"))?
            .children()
            .find(|c| {
                c.has_tag_name("font") && c.attribute("place").is_some_and(|p| p == "ActiveWindow")
            })
            .map(|f| {
                format!(
                    "{} {}",
                    f.attribute("name")
                        .or_else(|| f
                            .children()
                            .find(|c| c.has_tag_name("name"))
                            .and_then(|c| c.text()))
                        .unwrap_or("sans"),
                    f.attribute("size")
                        .or_else(|| f
                            .children()
                            .find(|c| c.has_tag_name("size"))
                            .and_then(|c| c.text()))
                        .unwrap_or("10")
                )
            })
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
    s.append(&noch_nicht("GTK-Schriften über gsettings wären sofort wirksam, der Fenstertitel braucht rc.xml und „labwc --reconfigure“."));
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

fn tastatur() -> gtk4::Widget {
    let s = seite("Tastatur");
    let rc = rc_xml();
    let belegung =
        std::env::var("XKB_DEFAULT_LAYOUT").unwrap_or_else(|_| "(nicht gesetzt: us)".into());
    let variante = std::env::var("XKB_DEFAULT_VARIANT").unwrap_or_default();
    s.append(&werte(&[
        (
            "Belegung:",
            if variante.is_empty() {
                belegung
            } else {
                format!("{belegung} ({variante})")
            },
        ),
        (
            "Wiederholrate:",
            oder(rc.text(&["keyboard", "repeatRate"]), "25"),
        ),
        (
            "Verzögerung (ms):",
            oder(rc.text(&["keyboard", "repeatDelay"]), "600"),
        ),
        ("Num-Taste:", oder(rc.text(&["keyboard", "numlock"]), "off")),
        (
            "Arbeitsflächen wechseln:",
            "Strg+Alt+←/→, mit Umschalt Fenster mitnehmen (rc.xml)".into(),
        ),
    ]));
    s.append(&quelle("Belegung: Runtime-Datei labwc/environment (XKB_DEFAULT_LAYOUT, nur beim Start von labwc gelesen); Rate/Verzögerung: rc.xml <keyboard>."));
    s.append(&noch_nicht("Belegung wirkt erst nach neuer Anmeldung; Rate/Verzögerung brauchen rc.xml und „labwc --reconfigure“."));
    s.upcast()
}

fn maus() -> gtk4::Widget {
    let s = seite("Maus");
    let rc = rc_xml();
    let geraet = |k: &str| oder(rc.text(&["libinput", "device", k]), "libinput-Standard");
    s.append(&werte(&[
        (
            "Doppelklickzeit (ms):",
            oder(rc.text(&["mouse", "doubleClickTime"]), "500"),
        ),
        ("Zeigergeschwindigkeit:", geraet("pointerSpeed")),
        ("Beschleunigung:", geraet("accelProfile")),
        ("Linkshändig:", geraet("leftHanded")),
        ("Natürliches Rollen:", geraet("naturalScroll")),
        (
            "Zeigerthema / -größe (labwc):",
            format!(
                "{} / {}",
                std::env::var("XCURSOR_THEME").unwrap_or_else(|_| "(Standard)".into()),
                std::env::var("XCURSOR_SIZE").unwrap_or_else(|_| "24".into())
            ),
        ),
    ]));
    s.append(&quelle("rc.xml <mouse><doubleClickTime>, <libinput><device>; Zeiger: Umgebungsvariablen XCURSOR_THEME/XCURSOR_SIZE (labwc/environment)."));
    s.append(&noch_nicht(
        "Änderungen brauchen rc.xml und „labwc --reconfigure“ (Zeigerthema: neue Anmeldung).",
    ));
    s.upcast()
}

fn arbeitsflaechen() -> gtk4::Widget {
    let s = seite("Arbeitsflächen");
    let rc = rc_xml();
    let namen = rc
        .dokument()
        .map(|d| {
            d.root_element()
                .children()
                .find(|c| c.has_tag_name("desktops"))
                .map(|ds| {
                    ds.children()
                        .find(|c| c.has_tag_name("names"))
                        .map(|n| {
                            n.children()
                                .filter(|c| c.has_tag_name("name"))
                                .filter_map(|c| c.text())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();
    s.append(&werte(&[
        ("Anzahl:", oder(rc.attribut(&["desktops"], "number"), "1")),
        (
            "Namensvorsilbe:",
            oder(rc.attribut(&["desktops"], "prefix"), "Workspace"),
        ),
        (
            "Eigene Namen:",
            if namen.is_empty() {
                "keine".into()
            } else {
                namen
            },
        ),
        (
            "Anzeige beim Wechsel (ms):",
            oder(rc.attribut(&["desktops"], "popupTime"), "1000"),
        ),
    ]));
    s.append(&quelle(
        "rc.xml <desktops>; der Umschalter im Panel folgt ext_workspace_v1.",
    ));
    s.append(&noch_nicht(
        "Anzahl und Namen ändern: rc.xml und „labwc --reconfigure“.",
    ));
    s.upcast()
}

fn standardanwendungen() -> gtk4::Widget {
    let s = seite("Standardanwendungen");
    let app_name = |a: Option<gio::AppInfo>| {
        a.map(|a| a.name().to_string())
            .unwrap_or_else(|| "(keine)".into())
    };
    s.append(&werte(&[
        (
            "Webbrowser:",
            app_name(gio::AppInfo::default_for_uri_scheme("http")),
        ),
        (
            "E-Mail:",
            app_name(gio::AppInfo::default_for_uri_scheme("mailto")),
        ),
        (
            "Ordner:",
            app_name(gio::AppInfo::default_for_type("inode/directory", false)),
        ),
        (
            "Textdateien:",
            app_name(gio::AppInfo::default_for_type("text/plain", false)),
        ),
        (
            "Bilder (PNG):",
            app_name(gio::AppInfo::default_for_type("image/png", false)),
        ),
        (
            "PDF:",
            app_name(gio::AppInfo::default_for_type("application/pdf", false)),
        ),
        (
            "Terminal:",
            "Querencia Terminal (Panel-Menü; kein MIME-Standard)".into(),
        ),
    ]));
    s.append(&quelle("GIO (mimeapps.list in ~/.config, sonst Systemstandard). ~/.config/mimeapps.list existiert noch nicht."));
    s.append(&noch_nicht("Ändern über gio::AppInfo::set_as_default_for_type ist technisch bereit (schreibt ~/.config/mimeapps.list) – wartet auf Freigabe als eigener Meilenstein."));
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
