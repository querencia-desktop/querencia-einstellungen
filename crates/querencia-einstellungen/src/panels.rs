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

/// Typen der Panel-Elemente und ihre Namen (wie `applets.rs` im Panel).
const NAMEN: &[(&str, &str)] = &[
    ("applications", "Anwendungen"),
    ("places", "Orte"),
    ("system", "System"),
    ("window-list", "Fensterliste"),
    ("workspace-switcher", "Arbeitsflächenumschalter"),
    ("clock", "Uhr"),
    ("volume", "Lautstärke"),
    ("network", "Netzwerk"),
];

const BEREICHE: &[(&str, &str)] = &[("Start", "links"), ("Center", "Mitte"), ("End", "rechts")];

pub fn pfad() -> PathBuf {
    glib::user_config_dir().join("querencia").join("panel.ini")
}

fn kante(k: &str) -> String {
    match k {
        "top" => "Oberes Panel".into(),
        "bottom" => "Unteres Panel".into(),
        "left" => "Linkes Panel".into(),
        "right" => "Rechtes Panel".into(),
        _ => format!("Panel ({k})"),
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
        return format!("Starter „{}“", app.unwrap_or_else(|| id.to_owned()));
    }
    NAMEN
        .iter()
        .find(|(t, _)| *t == typ)
        .map_or(typ.clone(), |(_, n)| (*n).to_owned())
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
                teile.push(format!("{wo}: {}", namen.join(", ")));
            }
        }
        zeilen.push((
            format!("{} ({hoehe} Pixel):", kante(&k)),
            if teile.is_empty() {
                "leer".into()
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
                format!("nicht lesbar ({e}) – das Panel nutzt dann seinen Standardaufbau"),
            )]
        }),
        Err(_) => vec![(
            "panel.ini:".into(),
            "noch nicht vorhanden – das Panel zeigt seinen Standardaufbau und legt die Datei bei der ersten Änderung an".into(),
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
    let s = bereiche::seite("Panels");
    let einleitung = gtk4::Label::new(None);
    einleitung.set_markup("<b>Die Panels richten Sie direkt am Panel ein, wie in GNOME 2.</b> Jede Änderung speichert das Panel sofort; diese Seite zeigt den gespeicherten Aufbau.");
    einleitung.set_xalign(0.0);
    einleitung.set_wrap(true);
    einleitung.set_max_width_chars(70);
    s.append(&einleitung);

    let rahmen = gtk4::Frame::new(Some("So geht es"));
    let liste = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    liste.set_margin_top(6);
    liste.set_margin_bottom(6);
    liste.set_margin_start(8);
    liste.set_margin_end(8);
    for zeile in [
        "Hinzufügen: Rechtsklick auf eine freie Stelle im Panel → „Zum Panel hinzufügen …“ (Menüs, Uhr, Lautstärke, Netzwerk, Anwendungsstarter). Das Element kommt in den angeklickten Teil des Panels.",
        "Verschieben: Rechtsklick auf ein Element → „Nach links“, „Nach rechts“, „An den linken Rand“, „An den rechten Rand“; oder mit der mittleren Maustaste (bzw. Alt und linker Taste) ziehen.",
        "Entfernen: Rechtsklick auf einen Starter, die Uhr oder die Lautstärke → „Aus dem Panel entfernen“.",
    ] {
        let l = gtk4::Label::new(Some(&format!("•  {zeile}")));
        l.set_xalign(0.0);
        l.set_wrap(true);
        l.set_max_width_chars(70);
        liste.append(&l);
    }
    rahmen.set_child(Some(&liste));
    s.append(&rahmen);

    let rahmen = gtk4::Frame::new(Some("Gespeicherter Aufbau"));
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

    let lage = gtk4::Label::new(Some(
        "Lage und Höhe der beiden Panels sind fest eingestellt (siehe oben); ein Menüpunkt dafür kommt mit einer späteren Fassung des Panels.",
    ));
    lage.set_xalign(0.0);
    lage.set_wrap(true);
    lage.set_max_width_chars(70);
    lage.add_css_class("dim-label");
    s.append(&lage);
    s.append(&bereiche::quelle(&format!(
        "{} (Formatversion 1), geschrieben vom Panel",
        pfad().display()
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
        assert_eq!(z[0].0, "Oberes Panel (32 Pixel):");
        assert_eq!(
            z[0].1,
            "links: Anwendungen, Orte, System, Starter „Firefox“\nrechts: Lautstärke, Netzwerk, Uhr"
        );
        assert_eq!(z[1].0, "Unteres Panel (32 Pixel):");
        assert_eq!(
            z[1].1,
            "links: Fensterliste\nrechts: Arbeitsflächenumschalter"
        );
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
            (
                "Linkes Panel (32 Pixel):".to_owned(),
                "links: neu".to_owned()
            )
        );
        assert!(aufbau("kein ini", &|_| None).is_err());
        assert!(aufbau("[Layout]\n", &|_| None).is_err());
    }
}
