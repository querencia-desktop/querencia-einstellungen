//! Bereich „Arbeitsflächen“ (G2-AG, QD-046 ⑤): Anzahl (1 bis 12), eigene
//! Namen und die Anzeige beim Wechsel (`rc.xml <desktops number=""
//! popupTime=""><names><name>`). labwc legt beim Neuladen fehlende
//! Arbeitsflächen an, benennt um und entfernt überzählige (die Fenster darauf
//! kommen auf die erste, labwc 0.9.8 `workspaces_reconfigure`). Der
//! Umschalter im Panel folgt über ext-workspace.
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n::{gettext, gettext_noop};
use crate::labwc_konfig::Stand;
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::{self as lx, Fehler};

pub const HOECHSTENS: u32 = 12;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    pub anzahl: u32,
    /// genau `anzahl` Einträge; leer = Standardname „<Vorsilbe> <n>“
    pub namen: Vec<String>,
    pub anzeige: bool,
    /// Millisekunden, nur mit `anzeige`
    pub dauer: u32,
    /// `<desktops prefix="">` (nur gelesen, für die Standardnamen)
    pub vorsilbe: String,
}

fn standardname(vorsilbe: &str, i: usize) -> String {
    if vorsilbe.is_empty() {
        format!("{}", i + 1)
    } else {
        format!("{vorsilbe} {}", i + 1)
    }
}

impl Werte {
    /// Die Namen, wie sie in rc.xml stehen sollen: leere Einträge am Ende
    /// fallen weg (labwc ergänzt sie), leere davor bekommen den Standardnamen.
    fn namen_fuer_datei(&self) -> Vec<String> {
        let letzte = self.namen.iter().rposition(|n| !n.trim().is_empty());
        match letzte {
            None => Vec::new(),
            Some(l) => self.namen[..=l]
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    if n.trim().is_empty() {
                        standardname(&self.vorsilbe, i)
                    } else {
                        n.trim().to_owned()
                    }
                })
                .collect(),
        }
    }
}

pub struct Arbeitsflaechen {
    anzahl: gtk4::SpinButton,
    namen: Vec<(gtk4::Box, gtk4::Entry)>,
    anzeige: gtk4::CheckButton,
    dauer: gtk4::SpinButton,
    vorsilbe: std::cell::RefCell<String>,
}

impl Bereich for Arbeitsflaechen {
    type Werte = Werte;
    const NAME: &'static str = gettext_noop("Workspaces");
    const QUELLE: &'static str = gettext_noop(
        "~/.config/querencia/labwc/rc.xml <desktops> (number, popupTime, names); the switcher in the panel follows ext-workspace.",
    );

    fn lesen(st: &Stand) -> Werte {
        let a = |k: &str| lx::wert(&st.rc, lx::ARBEITSFLAECHEN, k);
        let vorsilbe = a("prefix").unwrap_or_else(|| "Workspace".into());
        let datei = lx::namen(&st.rc);
        let zahl = a("number")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1)
            .max(1);
        let anzahl = zahl.max(datei.len() as u32).min(HOECHSTENS);
        let namen = (0..anzahl as usize)
            .map(|i| match datei.get(i) {
                Some(n) if *n != standardname(&vorsilbe, i) => n.clone(),
                _ => String::new(),
            })
            .collect();
        let popup = a("popupTime")
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(1000);
        Werte {
            anzahl,
            namen,
            anzeige: popup > 0,
            dauer: if popup > 0 {
                popup.min(10_000) as u32
            } else {
                1000
            },
            vorsilbe,
        }
    }

    fn rc_aendern(neu: &Werte, alt: &Werte, rc: &str) -> Result<String, Fehler> {
        let mut t = rc.to_owned();
        if neu.anzahl != alt.anzahl {
            t = lx::setzen(&t, lx::ARBEITSFLAECHEN, "number", &neu.anzahl.to_string())?;
        }
        let namen = neu.namen_fuer_datei();
        if namen != alt.namen_fuer_datei() || lx::namen(&t).len() > neu.anzahl as usize {
            t = lx::namen_setzen(&t, &namen)?;
        }
        if (neu.anzeige, neu.dauer) != (alt.anzeige, alt.dauer) {
            let p = if neu.anzeige { neu.dauer } else { 0 };
            t = lx::setzen(&t, lx::ARBEITSFLAECHEN, "popupTime", &p.to_string())?;
        }
        Ok(t)
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        let anzahl = gtk4::SpinButton::with_range(1.0, HOECHSTENS as f64, 1.0);
        inhalt.append(&ls::zeile(&gettext("Number of workspaces:"), &anzahl));
        inhalt.append(&ls::grau(&gettext(
            "Windows on removed workspaces move to the first one.",
        )));

        let (f, b) = ls::rahmen(&gettext("Names"));
        let mut namen = Vec::new();
        for i in 0..HOECHSTENS as usize {
            let e = gtk4::Entry::new();
            e.set_hexpand(true);
            let z = ls::zeile(&format!("{}:", i + 1), &e);
            b.append(&z);
            namen.push((z, e));
        }
        b.append(&ls::grau(&gettext("Leave empty for the default name.")));
        inhalt.append(&f);

        let (f, b) = ls::rahmen(&gettext("When Switching"));
        let anzeige = gtk4::CheckButton::with_label(&gettext("Briefly show the workspace name"));
        b.append(&anzeige);
        let dauer = gtk4::SpinButton::with_range(100.0, 5000.0, 100.0);
        b.append(&ls::zeile(&gettext("Duration (milliseconds):"), &dauer));
        b.append(&ls::grau(&gettext(
            "The switcher in the panel always shows the switch; the overlay comes from labwc.",
        )));
        inhalt.append(&f);

        let a = Arbeitsflaechen {
            anzahl,
            namen,
            anzeige,
            dauer,
            vorsilbe: std::cell::RefCell::new(String::new()),
        };
        let (g, zeilen, sp) = (
            geaendert.clone(),
            a.namen.iter().map(|(z, _)| z.clone()).collect::<Vec<_>>(),
            a.anzahl.clone(),
        );
        a.anzahl.connect_value_changed(move |_| {
            sichtbar(&zeilen, sp.value_as_int());
            g();
        });
        for (_, e) in &a.namen {
            let g = geaendert.clone();
            e.connect_changed(move |_| g());
        }
        let (g, d) = (geaendert.clone(), a.dauer.clone());
        a.anzeige.connect_toggled(move |k| {
            d.set_sensitive(k.is_active());
            g();
        });
        let g = geaendert.clone();
        a.dauer.connect_value_changed(move |_| g());
        (a, inhalt)
    }

    fn zeigen(&self, w: &Werte) {
        *self.vorsilbe.borrow_mut() = w.vorsilbe.clone();
        self.anzahl.set_value(w.anzahl as f64);
        for (i, (_, e)) in self.namen.iter().enumerate() {
            e.set_text(w.namen.get(i).map_or("", String::as_str));
            e.set_placeholder_text(Some(&standardname(&w.vorsilbe, i)));
        }
        let zeilen: Vec<gtk4::Box> = self.namen.iter().map(|(z, _)| z.clone()).collect();
        sichtbar(&zeilen, w.anzahl as i32);
        self.anzeige.set_active(w.anzeige);
        self.dauer.set_value(w.dauer as f64);
        self.dauer.set_sensitive(w.anzeige);
    }

    fn aktuell(&self) -> Werte {
        let anzahl = self.anzahl.value_as_int().clamp(1, HOECHSTENS as i32) as u32;
        Werte {
            anzahl,
            namen: self.namen[..anzahl as usize]
                .iter()
                .map(|(_, e)| e.text().trim().to_owned())
                .collect(),
            anzeige: self.anzeige.is_active(),
            dauer: self.dauer.value_as_int().max(100) as u32,
            vorsilbe: self.vorsilbe.borrow().clone(),
        }
    }
}

fn sichtbar(zeilen: &[gtk4::Box], anzahl: i32) {
    for (i, z) in zeilen.iter().enumerate() {
        z.set_visible((i as i32) < anzahl);
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

    const VORGABE: &str = "<labwc_config>\n  <desktops number=\"4\" popupTime=\"0\" prefix=\"Arbeitsfläche\" />\n</labwc_config>\n";

    #[test]
    fn lesen_vorgabe_und_labwc_standard() {
        let w = Arbeitsflaechen::lesen(&stand(VORGABE));
        assert_eq!(
            (w.anzahl, w.anzeige, w.dauer, w.vorsilbe.as_str()),
            (4, false, 1000, "Arbeitsfläche")
        );
        assert_eq!(w.namen, vec![String::new(); 4]);
        let w = Arbeitsflaechen::lesen(&stand("<labwc_config/>"));
        assert_eq!(
            (w.anzahl, w.anzeige, w.dauer, w.vorsilbe.as_str()),
            (1, true, 1000, "Workspace")
        );
        // mehr Namen als number: Namen zählen (labwc legt sie an)
        let w = Arbeitsflaechen::lesen(&stand(
            "<labwc_config><desktops number=\"2\"><names><name>A</name><name>B</name><name>C</name></names></desktops></labwc_config>",
        ));
        assert_eq!(
            (w.anzahl, w.namen.clone()),
            (3, vec!["A".to_string(), "B".into(), "C".into()])
        );
    }

    #[test]
    fn schreiben_rundlauf() {
        let st = stand(VORGABE);
        let alt = Arbeitsflaechen::lesen(&st);
        let neu = Werte {
            anzahl: 6,
            namen: vec![
                "Büro".into(),
                String::new(),
                "Web".into(),
                String::new(),
                String::new(),
                String::new(),
            ],
            anzeige: true,
            dauer: 800,
            vorsilbe: alt.vorsilbe.clone(),
        };
        let rc = Arbeitsflaechen::rc_aendern(&neu, &alt, &st.rc).unwrap();
        assert_eq!(lx::namen(&rc), vec!["Büro", "Arbeitsfläche 2", "Web"]);
        assert_eq!(Arbeitsflaechen::lesen(&stand(&rc)), neu);
        assert!(rc.contains("number=\"6\" popupTime=\"800\" prefix=\"Arbeitsfläche\""));
        // weniger: Namen werden mit gekürzt
        let weniger = Werte {
            anzahl: 2,
            namen: vec!["Büro".into(), String::new()],
            ..neu.clone()
        };
        let rc2 = Arbeitsflaechen::rc_aendern(&weniger, &neu, &rc).unwrap();
        assert_eq!(lx::namen(&rc2), vec!["Büro"]);
        assert_eq!(Arbeitsflaechen::lesen(&stand(&rc2)).anzahl, 2);
        // Anzeige aus
        let aus = Werte {
            anzeige: false,
            ..weniger.clone()
        };
        let rc3 = Arbeitsflaechen::rc_aendern(&aus, &weniger, &rc2).unwrap();
        assert_eq!(
            lx::wert(&rc3, lx::ARBEITSFLAECHEN, "popupTime").as_deref(),
            Some("0")
        );
    }
}
