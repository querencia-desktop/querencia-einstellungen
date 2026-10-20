//! Bereich „Fensterdesign“ (G2-AG, QD-046 ⑤): labwc-Thema, Eckenradius,
//! Schatten, Titelleistenknöpfe und – weil es nur rc.xml betrifft – die
//! Schrift der Titelleisten (`<theme><font place="ActiveWindow">` und
//! `InactiveWindow`). Alles wirkt mit `labwc --reconfigure` sofort.
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{glib, pango};

use crate::labwc_konfig::Stand;
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::{self as lx, Fehler};
use crate::tastatur::ja;

/// Titelleistenknöpfe (labwc `<theme><titlebar><layout>`), Standard zuerst.
pub const KNOEPFE: &[(&str, &str)] = &[
    (
        "icon:iconify,max,close",
        "Programmsymbol links; Minimieren, Maximieren, Schließen rechts (Standard)",
    ),
    (
        "menu:iconify,max,close",
        "Fenstermenü links; Minimieren, Maximieren, Schließen rechts",
    ),
    (
        "close,iconify,max:",
        "Schließen, Minimieren, Maximieren links",
    ),
    ("icon:close", "Nur Schließen"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    /// leer = labwcs eingebautes Thema
    pub thema: String,
    pub eckenradius: u32,
    pub schatten: bool,
    pub knoepfe: String,
    pub schrift: String,
    /// Punkte
    pub schriftgroesse: u32,
}

/// Installierte labwc-Themen (`themes/<name>/labwc/themerc`).
pub fn themen() -> Vec<String> {
    let mut ordner = vec![
        glib::user_data_dir().join("themes"),
        glib::home_dir().join(".themes"),
    ];
    ordner.extend(
        glib::system_data_dirs()
            .into_iter()
            .map(|d| d.join("themes")),
    );
    let mut liste = Vec::new();
    for d in ordner {
        if let Ok(e) = std::fs::read_dir(&d) {
            for t in e.flatten() {
                if t.path().join("labwc").join("themerc").is_file() {
                    liste.push(t.file_name().to_string_lossy().into_owned());
                }
            }
        }
    }
    liste.sort();
    liste.dedup();
    liste
}

pub struct Fensterdesign {
    themen_kasten: gtk4::Box,
    themen: RefCell<Vec<(String, gtk4::CheckButton)>>,
    eckenradius: gtk4::SpinButton,
    schatten: gtk4::CheckButton,
    knoepfe_kasten: gtk4::Box,
    knoepfe: RefCell<Vec<(String, gtk4::CheckButton)>>,
    schrift: gtk4::FontDialogButton,
    geaendert: Rc<dyn Fn()>,
}

impl Bereich for Fensterdesign {
    type Werte = Werte;
    const NAME: &'static str = "Fensterdesign";
    const QUELLE: &'static str = "~/.config/querencia/labwc/rc.xml <theme> (name, cornerRadius, dropShadows, titlebar/layout, font place=\"ActiveWindow\"/\"InactiveWindow\"); Themen in ~/.local/share/themes/*/labwc/themerc und /usr/share/themes.";

    fn lesen(st: &Stand) -> Werte {
        let t = |k: &str| lx::wert(&st.rc, lx::THEMA, k);
        let schrift = |k: &str| {
            lx::wert(&st.rc, lx::SCHRIFT_AKTIV, k).or_else(|| lx::wert(&st.rc, lx::SCHRIFT_ALLE, k))
        };
        Werte {
            thema: t("name").unwrap_or_default(),
            eckenradius: t("cornerRadius").and_then(|v| v.parse().ok()).unwrap_or(8),
            schatten: ja(t("dropShadows")),
            knoepfe: lx::wert(&st.rc, lx::TITELLEISTE, "layout")
                .unwrap_or_else(|| KNOEPFE[0].0.to_owned()),
            schrift: schrift("name").unwrap_or_else(|| "sans".into()),
            schriftgroesse: schrift("size").and_then(|v| v.parse().ok()).unwrap_or(10),
        }
    }

    fn rc_aendern(neu: &Werte, alt: &Werte, rc: &str) -> Result<String, Fehler> {
        let mut t = rc.to_owned();
        if neu.thema != alt.thema && !neu.thema.is_empty() {
            t = lx::setzen(&t, lx::THEMA, "name", &neu.thema)?;
        }
        if neu.eckenradius != alt.eckenradius {
            t = lx::setzen(&t, lx::THEMA, "cornerRadius", &neu.eckenradius.to_string())?;
        }
        if neu.schatten != alt.schatten {
            t = lx::setzen(
                &t,
                lx::THEMA,
                "dropShadows",
                if neu.schatten { "yes" } else { "no" },
            )?;
        }
        if neu.knoepfe != alt.knoepfe {
            t = lx::setzen(&t, lx::TITELLEISTE, "layout", &neu.knoepfe)?;
        }
        if (&neu.schrift, neu.schriftgroesse) != (&alt.schrift, alt.schriftgroesse) {
            for ort in [lx::SCHRIFT_AKTIV, lx::SCHRIFT_INAKTIV] {
                t = lx::setzen(&t, ort, "name", &neu.schrift)?;
                t = lx::setzen(&t, ort, "size", &neu.schriftgroesse.to_string())?;
            }
        }
        Ok(t)
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        let (f, b) = ls::rahmen("Thema der Fensterrahmen");
        let themen_kasten = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        b.append(&themen_kasten);
        inhalt.append(&f);

        let (f, b) = ls::rahmen("Form");
        let eckenradius = gtk4::SpinButton::with_range(0.0, 20.0, 1.0);
        b.append(&ls::zeile("Eckenradius oben (Pixel):", &eckenradius));
        let schatten = gtk4::CheckButton::with_label("Schatten hinter Fenstern");
        b.append(&schatten);
        inhalt.append(&f);

        let (f, b) = ls::rahmen("Knöpfe der Titelleiste");
        let knoepfe_kasten = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        b.append(&knoepfe_kasten);
        inhalt.append(&f);

        let (f, b) = ls::rahmen("Schrift der Titelleiste");
        let schrift = gtk4::FontDialogButton::new(Some(
            gtk4::FontDialog::builder()
                .title("Schrift der Titelleiste")
                .modal(true)
                .build(),
        ));
        schrift.set_halign(gtk4::Align::Start);
        b.append(&schrift);
        b.append(&ls::grau(
            "Für aktive und inaktive Fenster. Die übrigen Schriften stehen im Bereich „Schriften“.",
        ));
        inhalt.append(&f);

        let d = Fensterdesign {
            themen_kasten,
            themen: RefCell::new(Vec::new()),
            eckenradius,
            schatten,
            knoepfe_kasten,
            knoepfe: RefCell::new(Vec::new()),
            schrift,
            geaendert: geaendert.clone(),
        };
        let g = geaendert.clone();
        d.eckenradius.connect_value_changed(move |_| g());
        let g = geaendert.clone();
        d.schatten.connect_toggled(move |_| g());
        let g = geaendert.clone();
        d.schrift.connect_font_desc_notify(move |_| g());
        (d, inhalt)
    }

    fn zeigen(&self, w: &Werte) {
        let mut themen = themen();
        if !themen.contains(&w.thema) {
            themen.push(w.thema.clone());
        }
        let beschriftung = |t: &String| {
            if t.is_empty() {
                "Eingebautes labwc-Thema".to_owned()
            } else if t == &w.thema && !self::themen().contains(t) {
                format!("{t} (nicht installiert)")
            } else {
                t.clone()
            }
        };
        let liste: Vec<(String, String)> = themen
            .iter()
            .map(|t| (t.clone(), beschriftung(t)))
            .collect();
        ls::optionen(
            &self.themen_kasten,
            &self.themen,
            &liste,
            &w.thema,
            &self.geaendert,
        );
        self.eckenradius.set_value(w.eckenradius as f64);
        self.schatten.set_active(w.schatten);
        let mut knoepfe: Vec<(String, String)> = KNOEPFE
            .iter()
            .map(|(k, t)| ((*k).to_owned(), (*t).to_owned()))
            .collect();
        if !knoepfe.iter().any(|(k, _)| k == &w.knoepfe) {
            knoepfe.push((
                w.knoepfe.clone(),
                format!("Eigene Anordnung: {}", w.knoepfe),
            ));
        }
        ls::optionen(
            &self.knoepfe_kasten,
            &self.knoepfe,
            &knoepfe,
            &w.knoepfe,
            &self.geaendert,
        );
        let mut fd = pango::FontDescription::from_string(&w.schrift);
        fd.set_size(w.schriftgroesse as i32 * pango::SCALE);
        self.schrift.set_font_desc(&fd);
    }

    fn aktuell(&self) -> Werte {
        let gewaehlt = |l: &RefCell<Vec<(String, gtk4::CheckButton)>>| {
            l.borrow()
                .iter()
                .find(|(_, k)| k.is_active())
                .map(|(t, _)| t.clone())
                .unwrap_or_default()
        };
        let fd = self.schrift.font_desc();
        Werte {
            thema: gewaehlt(&self.themen),
            eckenradius: self.eckenradius.value_as_int().max(0) as u32,
            schatten: self.schatten.is_active(),
            knoepfe: gewaehlt(&self.knoepfe),
            schrift: fd
                .as_ref()
                .and_then(|f| f.family())
                .map_or_else(|| "sans".into(), |f| f.to_string()),
            schriftgroesse: fd
                .as_ref()
                .map(|f| f.size() / pango::SCALE)
                .filter(|s| *s > 0)
                .unwrap_or(10) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labwc_konfig::Quelle;
    use std::collections::BTreeMap;

    fn stand(rc: &str) -> Stand {
        Stand {
            rc: rc.into(),
            quelle: Quelle::Benutzer,
            rc_vorgabe: rc.into(),
            env: BTreeMap::new(),
            env_ohne_eigene: BTreeMap::new(),
            vorgabe_neu: false,
            fehler: None,
            gs: BTreeMap::new(),
        }
    }

    const VORGABE: &str = "<labwc_config>\n  <theme>\n    <name>Querencia-Klassisch</name>\n    <font place=\"OnScreenDisplay\">\n      <name>Adwaita Sans</name>\n      <size>11</size>\n    </font>\n  </theme>\n</labwc_config>\n";

    #[test]
    fn lesen_vorgabe() {
        let w = Fensterdesign::lesen(&stand(VORGABE));
        assert_eq!(
            w,
            Werte {
                thema: "Querencia-Klassisch".into(),
                eckenradius: 8,
                schatten: false,
                knoepfe: "icon:iconify,max,close".into(),
                schrift: "sans".into(),
                schriftgroesse: 10
            }
        );
    }

    #[test]
    fn schreiben_rundlauf() {
        let st = stand(VORGABE);
        let alt = Fensterdesign::lesen(&st);
        let neu = Werte {
            thema: "Andere".into(),
            eckenradius: 0,
            schatten: true,
            knoepfe: "icon:close".into(),
            schrift: "DejaVu Sans".into(),
            schriftgroesse: 12,
        };
        let rc = Fensterdesign::rc_aendern(&neu, &alt, &st.rc).unwrap();
        assert_eq!(Fensterdesign::lesen(&stand(&rc)), neu);
        assert_eq!(
            lx::wert(&rc, lx::SCHRIFT_INAKTIV, "size").as_deref(),
            Some("12")
        );
        // OSD-Schrift bleibt
        assert!(rc.contains("<font place=\"OnScreenDisplay\">\n      <name>Adwaita Sans</name>\n      <size>11</size>"));
        // leeres Thema wird nie geschrieben
        let rc2 = Fensterdesign::rc_aendern(
            &Werte {
                thema: String::new(),
                ..alt.clone()
            },
            &alt,
            &st.rc,
        )
        .unwrap();
        assert_eq!(rc2, st.rc);
    }
}
