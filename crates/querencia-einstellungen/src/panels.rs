//! Seite „Panels“ (G2-AJ): zeigt den gespeicherten Aufbau aus
//! `~/.config/querencia/panel.ini` mit den Namen, die auch das Panel zeigt,
//! und erklärt, wie man ihn ändert – im Panel selbst (Rechtsklick), wie in
//! GNOME 2. Die Einstellungen schreiben panel.ini nicht: Das laufende Panel
//! hat keine Schnittstelle zum Neuladen, eine Änderung von außen würde erst
//! beim nächsten Start sichtbar und beim nächsten Speichern des Panels
//! verworfen. Die Anzeige folgt Änderungen an der Datei.
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gio, glib};

use crate::bereiche;
use crate::i18n::{gettext, gettext_f, gettext_noop};

/// Typen der Panel-Elemente und ihre Namen (wie `applets.rs` im Panel).
const NAMEN: &[(&str, &str)] = &[
    ("applications", gettext_noop("Applications")),
    ("places", gettext_noop("Places")),
    ("system", gettext_noop("System")),
    ("window-list", gettext_noop("Window List")),
    ("workspace-switcher", gettext_noop("Workspace Switcher")),
    ("clock", gettext_noop("Clock")),
    ("volume", gettext_noop("Volume")),
    ("network", gettext_noop("Network")),
];

const BEREICHE: &[(&str, &str)] = &[
    ("Start", gettext_noop("left")),
    ("Center", gettext_noop("center")),
    ("End", gettext_noop("right")),
];

pub fn pfad() -> PathBuf {
    glib::user_config_dir().join("querencia").join("panel.ini")
}

fn kante(k: &str) -> String {
    match k {
        "top" => gettext("Top Panel"),
        "bottom" => gettext("Bottom Panel"),
        "left" => gettext("Left Panel"),
        "right" => gettext("Right Panel"),
        _ => gettext_f("Panel ({edge})", &[("edge", k)]),
    }
}

/// Name eines Elements; `starter` liefert den Namen der Anwendung eines
/// Starters (Desktop-ID → Name).
fn element(kf: &glib::KeyFile, id: &str, starter: &dyn Fn(&str) -> Option<String>) -> String {
    let gruppe = format!("Applet {id}");
    let typ = kf
        .string(&gruppe, "Type")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| id.rsplit_once('-').map_or(id, |(t, _)| t).to_owned());
    if typ == "launcher" {
        let app = kf
            .string(&gruppe, "DesktopId")
            .ok()
            .and_then(|d| starter(&d).or_else(|| Some(d.to_string())));
        return gettext_f(
            "Launcher “{name}”",
            &[("name", &app.unwrap_or_else(|| id.to_owned()))],
        );
    }
    NAMEN
        .iter()
        .find(|(t, _)| *t == typ)
        .map_or(typ.clone(), |(_, n)| gettext(*n))
}

/// Zeilen (Beschriftung, Wert) des gespeicherten Aufbaus; `Err` mit Grund,
/// wenn die Datei fehlt oder unlesbar ist.
pub fn aufbau(
    text: &str,
    starter: &dyn Fn(&str) -> Option<String>,
) -> Result<Vec<(String, String)>, String> {
    let kf = glib::KeyFile::new();
    kf.load_from_data(text, glib::KeyFileFlags::NONE)
        .map_err(|e| e.to_string())?;
    let panels: Vec<String> = kf
        .string_list("Layout", "Panels")
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut zeilen = Vec::new();
    for p in panels {
        let gruppe = format!("Panel {p}");
        let k = kf
            .string(&gruppe, "Edge")
            .map(|s| s.to_string())
            .unwrap_or_else(|_| p.clone());
        let hoehe = kf.integer(&gruppe, "Height").unwrap_or(32);
        let mut teile = Vec::new();
        for (schluessel, wo) in BEREICHE {
            let ids: Vec<String> = kf
                .string_list(&gruppe, schluessel)
                .map(|l| {
                    l.iter()
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            if !ids.is_empty() {
                let namen: Vec<String> = ids.iter().map(|id| element(&kf, id, starter)).collect();
                teile.push(format!("{}: {}", gettext(*wo), namen.join(", ")));
            }
        }
        zeilen.push((
            gettext_f(
                "{panel} ({height} pixels):",
                &[("panel", &kante(&k)), ("height", &hoehe.to_string())],
            ),
            if teile.is_empty() {
                gettext("empty")
            } else {
                teile.join("\n")
            },
        ));
    }
    Ok(zeilen)
}

fn anwendungsname(desktop_id: &str) -> Option<String> {
    gio::AppInfo::all()
        .into_iter()
        .find(|a| a.id().as_deref() == Some(desktop_id))
        .map(|a| a.name().to_string())
}

fn fuellen(raster: &gtk4::Grid) {
    while let Some(c) = raster.first_child() {
        raster.remove(&c);
    }
    let zeilen = match std::fs::read_to_string(pfad()) {
        Ok(text) => aufbau(&text, &anwendungsname).unwrap_or_else(|e| {
            vec![(
                "panel.ini:".into(),
                gettext_f(
                    "not readable ({error}) – the panel then uses its default layout",
                    &[("error", &e)],
                ),
            )]
        }),
        Err(_) => vec![(
            "panel.ini:".into(),
            gettext(
                "not present yet – the panel shows its default layout and creates the file at the first change",
            ),
        )],
    };
    eprintln!(
        "querencia-einstellungen: Panels gelesen: {}",
        zeilen
            .iter()
            .map(|(k, v)| format!("{k} {}", v.replace('\n', " · ")))
            .collect::<Vec<_>>()
            .join(" | ")
    );
    for (i, (k, v)) in zeilen.iter().enumerate() {
        let kl = gtk4::Label::new(Some(k));
        kl.set_xalign(1.0);
        kl.set_yalign(0.0);
        kl.add_css_class("dim-label");
        let vl = gtk4::Label::new(Some(v));
        vl.set_xalign(0.0);
        vl.set_wrap(true);
        vl.set_hexpand(true);
        vl.set_selectable(true);
        raster.attach(&kl, 0, i as i32, 1, 1);
        raster.attach(&vl, 1, i as i32, 1, 1);
    }
}

pub fn seite() -> gtk4::Widget {
    let s = bereiche::seite(&gettext("Panels"));
    let einleitung = gtk4::Label::new(None);
    einleitung.set_markup(&format!(
        "<b>{}</b> {}",
        glib::markup_escape_text(&gettext(
            "You set up the panels directly on the panel, as in GNOME 2."
        )),
        glib::markup_escape_text(&gettext(
            "The panel saves every change immediately; this page shows the saved layout."
        ))
    ));
    einleitung.set_xalign(0.0);
    einleitung.set_wrap(true);
    einleitung.set_max_width_chars(70);
    s.append(&einleitung);

    let rahmen = gtk4::Frame::new(Some(&gettext("How It Works")));
    let liste = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    liste.set_margin_top(6);
    liste.set_margin_bottom(6);
    liste.set_margin_start(8);
    liste.set_margin_end(8);
    for zeile in [
        gettext(
            "Add: right-click a free spot on the panel → “Add to Panel …” (menus, clock, volume, network, application launcher). The item goes into the clicked part of the panel.",
        ),
        gettext(
            "Move: right-click an item → “Move Left”, “Move Right”, “Move to the Left Edge”, “Move to the Right Edge”; or drag it with the middle mouse button (or Alt and the left button).",
        ),
        gettext(
            "Remove: right-click a launcher, the clock or the volume control → “Remove From Panel”.",
        ),
    ] {
        let l = gtk4::Label::new(Some(&format!("•  {zeile}")));
        l.set_xalign(0.0);
        l.set_wrap(true);
        l.set_max_width_chars(70);
        liste.append(&l);
    }
    rahmen.set_child(Some(&liste));
    s.append(&rahmen);

    let rahmen = gtk4::Frame::new(Some(&gettext("Saved Layout")));
    let raster = gtk4::Grid::new();
    raster.set_row_spacing(6);
    raster.set_column_spacing(12);
    raster.set_margin_top(6);
    raster.set_margin_bottom(6);
    raster.set_margin_start(8);
    raster.set_margin_end(8);
    fuellen(&raster);
    rahmen.set_child(Some(&raster));
    s.append(&rahmen);

    let lage = gtk4::Label::new(Some(&gettext(
        "Position and height of the two panels are fixed (see above); a menu item for this comes with a later version of the panel.",
    )));
    lage.set_xalign(0.0);
    lage.set_wrap(true);
    lage.set_max_width_chars(70);
    lage.add_css_class("dim-label");
    s.append(&lage);
    s.append(&bereiche::quelle(&gettext_f(
        "{file} (format version 1), written by the panel",
        &[("file", &pfad().display().to_string())],
    )));

    // Änderungen im Panel sofort zeigen (das Panel ersetzt die Datei atomar)
    let monitor: Rc<RefCell<Option<gio::FileMonitor>>> = Rc::new(RefCell::new(None));
    if let Ok(m) = gio::File::for_path(pfad())
        .monitor_file(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE)
    {
        let r = raster.clone();
        m.connect_changed(move |_, _, _, ereignis| {
            if !matches!(
                ereignis,
                gio::FileMonitorEvent::Changed | gio::FileMonitorEvent::AttributeChanged
            ) {
                fuellen(&r);
            }
        });
        *monitor.borrow_mut() = Some(m);
    }
    // der Monitor lebt mit der Seite
    s.connect_destroy(move |_| {
        monitor.borrow_mut().take();
    });
    s.upcast()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE: &str = "[Layout]\nVersion=1\nPanels=top;bottom\n\n[Panel top]\nEdge=top\nHeight=32\nStart=applications-1;places-1;system-1;launcher-1\nCenter=\nEnd=volume-1;network-1;clock-1\n\n[Panel bottom]\nEdge=bottom\nHeight=32\nStart=window-list-1\nEnd=workspace-switcher-1\n\n[Applet launcher-1]\nType=launcher\nDesktopId=firefox_firefox.desktop\n\n[Applet volume-1]\nType=volume\n";

    #[test]
    fn aufbau_mit_namen() {
        let z = aufbau(LIVE, &|d| {
            (d == "firefox_firefox.desktop").then(|| "Firefox".to_owned())
        })
        .unwrap();
        assert_eq!(z.len(), 2);
        assert_eq!(z[0].0, "Top Panel (32 pixels):");
        assert_eq!(
            z[0].1,
            "left: Applications, Places, System, Launcher “Firefox”\nright: Volume, Network, Clock"
        );
        assert_eq!(z[1].0, "Bottom Panel (32 pixels):");
        assert_eq!(z[1].1, "left: Window List\nright: Workspace Switcher");
    }

    #[test]
    fn unbekannt_und_kaputt() {
        let z = aufbau(
            "[Layout]\nPanels=p\n[Panel p]\nEdge=left\nStart=neu-1\n",
            &|_| None,
        )
        .unwrap();
        assert_eq!(
            z[0],
            ("Left Panel (32 pixels):".to_owned(), "left: neu".to_owned())
        );
        assert!(aufbau("kein ini", &|_| None).is_err());
        assert!(aufbau("[Layout]\n", &|_| None).is_err());
    }
}
