//! Bereich „Schriften“ (G2-AJ, QD-046 ④): Schriften der Programme über
//! gsettings (`font-name`, `document-font-name`, `monospace-font-name`,
//! wirken sofort über das Settings-Portal) und die Schrift der Fenstertitel
//! in der labwc-Benutzerkopie (`<theme><font place="ActiveWindow">` und
//! `"InactiveWindow"`: Name, Größe, Fett; wirkt mit `labwc --reconfigure`).
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::pango;
use gtk4::prelude::*;

use crate::gsettings::{self as gs, DICKTENGLEICH, DOKUMENTSCHRIFT, SCHRIFT};
use crate::labwc_konfig::{Stand, Wirkung};
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::{self as lx, Fehler};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    /// Pango-Beschreibungen wie in gsettings, z. B. „DejaVu Sans 10“
    pub anwendungen: String,
    pub dokumente: String,
    pub dicktengleich: String,
    pub titel_schrift: String,
    /// Punkte
    pub titel_groesse: u32,
    pub titel_fett: bool,
}

impl Werte {
    fn gs_liste(&self) -> Vec<(&'static str, String)> {
        vec![
            (SCHRIFT, self.anwendungen.clone()),
            (DOKUMENTSCHRIFT, self.dokumente.clone()),
            (DICKTENGLEICH, self.dicktengleich.clone()),
        ]
    }
}

/// Ein Schriftknopf und der Text, mit dem er gefüllt wurde (bleibt
/// unverändert, solange dieselbe Schrift gewählt ist – Pango schreibt manche
/// Beschreibungen anders zurück).
struct Knopf {
    knopf: gtk4::FontDialogButton,
    geladen: RefCell<String>,
}

impl Knopf {
    fn neu(titel: &str) -> Knopf {
        let knopf = gtk4::FontDialogButton::new(Some(
            gtk4::FontDialog::builder().title(titel).modal(true).build(),
        ));
        knopf.set_use_font(true);
        knopf.set_halign(gtk4::Align::Start);
        Knopf {
            knopf,
            geladen: RefCell::new(String::new()),
        }
    }

    fn zeigen(&self, text: &str) {
        *self.geladen.borrow_mut() = text.to_owned();
        self.knopf
            .set_font_desc(&pango::FontDescription::from_string(text));
    }

    fn wert(&self) -> String {
        let geladen = self.geladen.borrow().clone();
        match self.knopf.font_desc() {
            Some(fd) if fd != pango::FontDescription::from_string(&geladen) => fd.to_string(),
            _ => geladen,
        }
    }
}

pub struct Schriften {
    anwendungen: Knopf,
    dokumente: Knopf,
    dicktengleich: Knopf,
    titel: gtk4::FontDialogButton,
}

fn ist_fett(wert: Option<String>) -> bool {
    wert.is_some_and(|w| w.trim().eq_ignore_ascii_case("bold"))
}

impl Bereich for Schriften {
    type Werte = Werte;
    const NAME: &'static str = "Schriften";
    const QUELLE: &'static str = "gsettings org.gnome.desktop.interface (font-name, document-font-name, monospace-font-name); ~/.config/querencia/labwc/rc.xml <theme><font place=\"ActiveWindow\"> und \"InactiveWindow\" (name, size, weight).";
    const GSETTINGS: &'static [&'static str] = &[SCHRIFT, DOKUMENTSCHRIFT, DICKTENGLEICH];

    fn lesen(st: &Stand) -> Werte {
        let g = |k: &str| {
            st.gs
                .get(k)
                .cloned()
                .unwrap_or_else(|| gs::standard(&[k]).remove(k).unwrap_or_default())
        };
        let titel = |k: &str| {
            lx::wert(&st.rc, lx::SCHRIFT_AKTIV, k).or_else(|| lx::wert(&st.rc, lx::SCHRIFT_ALLE, k))
        };
        Werte {
            anwendungen: g(SCHRIFT),
            dokumente: g(DOKUMENTSCHRIFT),
            dicktengleich: g(DICKTENGLEICH),
            titel_schrift: titel("name").unwrap_or_else(|| "sans".into()),
            titel_groesse: titel("size").and_then(|v| v.parse().ok()).unwrap_or(10),
            titel_fett: ist_fett(titel("weight")),
        }
    }

    fn rc_aendern(neu: &Werte, alt: &Werte, rc: &str) -> Result<String, Fehler> {
        let mut t = rc.to_owned();
        for ort in [lx::SCHRIFT_AKTIV, lx::SCHRIFT_INAKTIV] {
            if (&neu.titel_schrift, neu.titel_groesse) != (&alt.titel_schrift, alt.titel_groesse) {
                t = lx::setzen(&t, ort, "name", &neu.titel_schrift)?;
                t = lx::setzen(&t, ort, "size", &neu.titel_groesse.to_string())?;
            }
            if neu.titel_fett != alt.titel_fett {
                t = lx::setzen(
                    &t,
                    ort,
                    "weight",
                    if neu.titel_fett { "bold" } else { "normal" },
                )?;
            }
        }
        Ok(t)
    }

    fn gs_aenderungen(neu: &Werte, alt: &Werte) -> Vec<(&'static str, String)> {
        gs::unterschiede(&neu.gs_liste(), &alt.gs_liste())
    }

    fn hinweis(w: &Wirkung, _st: &Stand) -> Option<String> {
        let titel = match w {
            Wirkung::Sofort { .. } => "ebenfalls sofort (labwc lädt seine Konfiguration neu)",
            Wirkung::NaechsteAnmeldung { .. } => "ab der nächsten Anmeldung",
        };
        Some(format!(
            "Die Schriften der Programme wirken nach „Übernehmen“ sofort in allen GTK-Programmen, im Panel und auf dem Desktop. Die Schrift der Fenstertitel wirkt {titel}."
        ))
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

        let (f, b) = ls::rahmen("Schriften der Programme");
        let raster = gtk4::Grid::new();
        raster.set_row_spacing(6);
        raster.set_column_spacing(12);
        let anwendungen = Knopf::neu("Schrift der Anwendungen");
        let dokumente = Knopf::neu("Schrift für Dokumente");
        let dicktengleich = Knopf::neu("Dicktengleiche Schrift");
        for (i, (text, k)) in [
            ("Anwendungen:", &anwendungen),
            ("Dokumente:", &dokumente),
            ("Dicktengleich:", &dicktengleich),
        ]
        .into_iter()
        .enumerate()
        {
            let l = gtk4::Label::new(Some(text));
            l.set_xalign(1.0);
            raster.attach(&l, 0, i as i32, 1, 1);
            raster.attach(&k.knopf, 1, i as i32, 1, 1);
        }
        b.append(&raster);
        b.append(&ls::grau(
            "„Anwendungen“ gilt für Menüs, Knöpfe und Beschriftungen, „Dokumente“ für Textinhalte, „Dicktengleich“ für Terminal und Quelltext. Der Schriftwähler zeigt alle installierten Schriften.",
        ));
        inhalt.append(&f);

        let (f, b) = ls::rahmen("Fenstertitel");
        let titel = gtk4::FontDialogButton::new(Some(
            gtk4::FontDialog::builder()
                .title("Schrift der Fenstertitel")
                .modal(true)
                .build(),
        ));
        titel.set_use_font(true);
        titel.set_halign(gtk4::Align::Start);
        b.append(&titel);
        b.append(&ls::grau(
            "Für aktive und inaktive Fenster; derselbe Wert steht im Bereich „Fensterdesign“. Fett wird übernommen, Kursiv nicht.",
        ));
        inhalt.append(&f);

        for k in [
            &anwendungen.knopf,
            &dokumente.knopf,
            &dicktengleich.knopf,
            &titel,
        ] {
            let g = geaendert.clone();
            k.connect_font_desc_notify(move |_| g());
        }
        (
            Schriften {
                anwendungen,
                dokumente,
                dicktengleich,
                titel,
            },
            inhalt,
        )
    }

    fn zeigen(&self, w: &Werte) {
        self.anwendungen.zeigen(&w.anwendungen);
        self.dokumente.zeigen(&w.dokumente);
        self.dicktengleich.zeigen(&w.dicktengleich);
        let mut fd = pango::FontDescription::new();
        fd.set_family(&w.titel_schrift);
        fd.set_size(w.titel_groesse as i32 * pango::SCALE);
        if w.titel_fett {
            fd.set_weight(pango::Weight::Bold);
        }
        self.titel.set_font_desc(&fd);
    }

    fn aktuell(&self) -> Werte {
        let fd = self.titel.font_desc();
        Werte {
            anwendungen: self.anwendungen.wert(),
            dokumente: self.dokumente.wert(),
            dicktengleich: self.dicktengleich.wert(),
            titel_schrift: fd
                .as_ref()
                .and_then(|f| f.family())
                .map_or_else(|| "sans".into(), |f| f.to_string()),
            titel_groesse: fd
                .as_ref()
                .map(|f| f.size() / pango::SCALE)
                .filter(|s| *s > 0)
                .unwrap_or(10) as u32,
            titel_fett: fd
                .as_ref()
                .is_some_and(|f| f.weight() >= pango::Weight::Bold),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labwc_konfig::Quelle;
    use std::collections::BTreeMap;

    const VORGABE: &str = "<labwc_config>\n  <theme>\n    <font place=\"ActiveWindow\">\n      <name>DejaVu Sans</name>\n      <size>10</size>\n      <weight>bold</weight>\n    </font>\n    <font place=\"InactiveWindow\">\n      <name>DejaVu Sans</name>\n      <size>10</size>\n      <weight>bold</weight>\n    </font>\n    <font place=\"MenuItem\">\n      <name>DejaVu Sans</name>\n      <size>10</size>\n    </font>\n  </theme>\n</labwc_config>\n";

    fn stand(rc: &str) -> Stand {
        Stand {
            rc: rc.into(),
            quelle: Quelle::Benutzer,
            rc_vorgabe: rc.into(),
            env: BTreeMap::new(),
            env_ohne_eigene: BTreeMap::new(),
            vorgabe_neu: false,
            fehler: None,
            gs: gs::standard(Schriften::GSETTINGS),
        }
    }

    #[test]
    fn lesen_vorgabe() {
        let w = Schriften::lesen(&stand(VORGABE));
        assert_eq!(
            w,
            Werte {
                anwendungen: "DejaVu Sans 10".into(),
                dokumente: "DejaVu Sans 10".into(),
                dicktengleich: "DejaVu Sans Mono 10".into(),
                titel_schrift: "DejaVu Sans".into(),
                titel_groesse: 10,
                titel_fett: true,
            }
        );
        // ohne Angabe: labwcs Standard
        let leer = Schriften::lesen(&stand("<labwc_config/>"));
        assert_eq!(
            (
                leer.titel_schrift.as_str(),
                leer.titel_groesse,
                leer.titel_fett
            ),
            ("sans", 10, false)
        );
    }

    #[test]
    fn titel_rundlauf_und_rest_bleibt() {
        let st = stand(VORGABE);
        let alt = Schriften::lesen(&st);
        let neu = Werte {
            titel_schrift: "DejaVu Serif".into(),
            titel_groesse: 12,
            titel_fett: false,
            ..alt.clone()
        };
        let rc = Schriften::rc_aendern(&neu, &alt, &st.rc).unwrap();
        assert_eq!(Schriften::lesen(&stand(&rc)), neu);
        assert_eq!(
            lx::wert(&rc, lx::SCHRIFT_INAKTIV, "weight").as_deref(),
            Some("normal")
        );
        assert!(rc.contains("<font place=\"MenuItem\">\n      <name>DejaVu Sans</name>\n      <size>10</size>\n    </font>"));
        // nur gsettings geändert: rc.xml unverändert
        let nur_gs = Werte {
            dokumente: "DejaVu Serif 11".into(),
            ..alt.clone()
        };
        assert_eq!(Schriften::rc_aendern(&nur_gs, &alt, &st.rc).unwrap(), st.rc);
        assert_eq!(
            Schriften::gs_aenderungen(&nur_gs, &alt),
            vec![(DOKUMENTSCHRIFT, "DejaVu Serif 11".to_owned())]
        );
    }

    #[test]
    fn fett_nur_bei_bold() {
        assert!(ist_fett(Some("Bold".into())));
        assert!(!ist_fett(Some("normal".into())));
        assert!(!ist_fett(None));
    }
}
