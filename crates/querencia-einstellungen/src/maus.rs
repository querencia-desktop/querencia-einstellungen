//! Bereich „Maus“ (G2-AG, QD-046 ③): Doppelklickzeit (`rc.xml <mouse>`),
//! Zeigergeschwindigkeit, Beschleunigung, Linkshändig, natürliches Rollen
//! (`<libinput><device category="default">`, gilt für alle Zeigegeräte ohne
//! eigenes Profil) und Zeigerthema/-größe (Umgebung `XCURSOR_THEME`,
//! `XCURSOR_SIZE` in `environment.d`). G2-AJ: ein geänderter Zeiger geht
//! zusätzlich nach gsettings (`cursor-theme`, `cursor-size`), wie im Bereich
//! „Erscheinungsbild“, damit GTK-Programme denselben Zeiger zeigen.
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::gsettings as gs;
use crate::i18n::{gettext, gettext_noop, pgettext};
use crate::labwc_konfig::Stand;
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::{self as lx, Fehler};
use crate::tastatur::ja;

pub const GROESSEN: &[(u32, &str)] = &[
    (24, gettext_noop("Normal (24)")),
    (32, gettext_noop("Large (32)")),
    (48, gettext_noop("Larger (48)")),
    (64, gettext_noop("Very large (64)")),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    /// Millisekunden (labwc-Standard 500)
    pub doppelklick: u32,
    /// Hundertstel von -1,00 bis 1,00 (libinput-Standard 0)
    pub geschwindigkeit: i32,
    /// `adaptive` (Standard) statt `flat`
    pub beschleunigung: bool,
    pub linkshaendig: bool,
    pub natuerlich: bool,
    /// leer = Standard (kein XCURSOR_THEME)
    pub zeigerthema: String,
    pub zeigergroesse: u32,
}

/// Installierte Zeigerthemen (Ordner mit `cursors/`), ohne „default“.
pub fn zeigerthemen() -> Vec<String> {
    let mut ordner: Vec<PathBuf> = vec![
        gtk4::glib::user_data_dir().join("icons"),
        gtk4::glib::home_dir().join(".icons"),
    ];
    ordner.extend(
        gtk4::glib::system_data_dirs()
            .into_iter()
            .map(|d| d.join("icons")),
    );
    let mut liste = Vec::new();
    for d in ordner {
        if let Ok(e) = std::fs::read_dir(&d) {
            for t in e.flatten() {
                let n = t.file_name().to_string_lossy().into_owned();
                if n != "default" && t.path().join("cursors").is_dir() {
                    liste.push(n);
                }
            }
        }
    }
    liste.sort();
    liste.dedup();
    liste
}

pub struct Maus {
    doppelklick: gtk4::SpinButton,
    geschwindigkeit: gtk4::Scale,
    beschleunigung: gtk4::CheckButton,
    linkshaendig: gtk4::CheckButton,
    natuerlich: gtk4::CheckButton,
    themen_kasten: gtk4::Box,
    themen: std::cell::RefCell<Vec<(String, gtk4::CheckButton)>>,
    groessen: Vec<gtk4::CheckButton>,
    /// Größe aus der Datei, die keine der vier Stufen ist (bleibt, solange
    /// keine Stufe gewählt wird)
    eigene_groesse: std::cell::Cell<u32>,
    geaendert: Rc<dyn Fn()>,
}

impl Bereich for Maus {
    type Werte = Werte;
    const NAME: &'static str = gettext_noop("Mouse");
    const PROTOKOLL: &'static str = "Maus";
    const QUELLE: &'static str = gettext_noop(
        "~/.config/querencia/labwc: rc.xml <mouse><doubleClickTime>, <libinput><device category=\"default\">; environment.d/zz-querencia-einstellungen.env (XCURSOR_THEME, XCURSOR_SIZE).",
    );

    fn lesen(st: &Stand) -> Werte {
        let g = |k: &str| lx::wert(&st.rc, lx::GERAET, k);
        Werte {
            doppelklick: lx::wert(&st.rc, lx::MAUS, "doubleClickTime")
                .and_then(|v| v.parse().ok())
                .filter(|v| *v > 0)
                .unwrap_or(500),
            geschwindigkeit: g("pointerSpeed")
                .and_then(|v| v.parse::<f64>().ok())
                .map_or(0, |v| (v.clamp(-1.0, 1.0) * 100.0).round() as i32),
            beschleunigung: g("accelProfile").is_none_or(|v| !v.eq_ignore_ascii_case("flat")),
            linkshaendig: ja(g("leftHanded")),
            natuerlich: ja(g("naturalScroll")),
            zeigerthema: st.env.get("XCURSOR_THEME").cloned().unwrap_or_default(),
            zeigergroesse: st
                .env
                .get("XCURSOR_SIZE")
                .and_then(|v| v.parse().ok())
                .unwrap_or(24),
        }
    }

    fn rc_aendern(neu: &Werte, alt: &Werte, rc: &str) -> Result<String, Fehler> {
        let mut t = rc.to_owned();
        let jn = |b: bool| if b { "yes" } else { "no" };
        if neu.doppelklick != alt.doppelklick {
            t = lx::setzen(
                &t,
                lx::MAUS,
                "doubleClickTime",
                &neu.doppelklick.to_string(),
            )?;
        }
        if neu.geschwindigkeit != alt.geschwindigkeit {
            t = lx::setzen(
                &t,
                lx::GERAET,
                "pointerSpeed",
                &format!("{:.2}", neu.geschwindigkeit as f64 / 100.0),
            )?;
        }
        if neu.beschleunigung != alt.beschleunigung {
            t = lx::setzen(
                &t,
                lx::GERAET,
                "accelProfile",
                if neu.beschleunigung {
                    "adaptive"
                } else {
                    "flat"
                },
            )?;
        }
        if neu.linkshaendig != alt.linkshaendig {
            t = lx::setzen(&t, lx::GERAET, "leftHanded", jn(neu.linkshaendig))?;
        }
        if neu.natuerlich != alt.natuerlich {
            t = lx::setzen(&t, lx::GERAET, "naturalScroll", jn(neu.natuerlich))?;
        }
        Ok(t)
    }

    fn env_aenderungen(
        neu: &Werte,
        alt: &Werte,
        st: &Stand,
    ) -> Vec<(&'static str, Option<String>)> {
        let mut v = Vec::new();
        if neu.zeigerthema != alt.zeigerthema {
            v.push(ls::env_wert(st, "XCURSOR_THEME", &neu.zeigerthema));
        }
        if neu.zeigergroesse != alt.zeigergroesse {
            let vorgabe = st
                .env_ohne_eigene
                .get("XCURSOR_SIZE")
                .and_then(|s| s.parse().ok())
                .unwrap_or(24);
            v.push((
                "XCURSOR_SIZE",
                (neu.zeigergroesse != vorgabe).then(|| neu.zeigergroesse.to_string()),
            ));
        }
        v
    }

    fn gs_aenderungen(neu: &Werte, alt: &Werte) -> Vec<(&'static str, String)> {
        let mut v = Vec::new();
        // „Standard“ (kein XCURSOR_THEME) heißt für GTK: Querencias Zeiger
        if neu.zeigerthema != alt.zeigerthema {
            let thema = if neu.zeigerthema.is_empty() {
                gs::standard(&[gs::ZEIGERTHEMA])
                    .remove(gs::ZEIGERTHEMA)
                    .unwrap_or_default()
            } else {
                neu.zeigerthema.clone()
            };
            v.push((gs::ZEIGERTHEMA, thema));
        }
        if neu.zeigergroesse != alt.zeigergroesse {
            v.push((gs::ZEIGERGROESSE, neu.zeigergroesse.to_string()));
        }
        v
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

        let (f, b) = ls::rahmen(&gettext("Pointer Motion"));
        let geschwindigkeit =
            gtk4::Scale::with_range(gtk4::Orientation::Horizontal, -1.0, 1.0, 0.05);
        geschwindigkeit.set_hexpand(true);
        geschwindigkeit.set_size_request(260, -1);
        geschwindigkeit.add_mark(-1.0, gtk4::PositionType::Bottom, Some(&gettext("Slow")));
        geschwindigkeit.add_mark(0.0, gtk4::PositionType::Bottom, None);
        geschwindigkeit.add_mark(1.0, gtk4::PositionType::Bottom, Some(&gettext("Fast")));
        geschwindigkeit.set_digits(2);
        geschwindigkeit.set_draw_value(true);
        geschwindigkeit.set_margin_start(16);
        geschwindigkeit.set_margin_end(28);
        geschwindigkeit.set_format_value_func(|_, v| {
            // TRANSLATORS: decimal separator of the pointer speed (+0.25)
            format!("{v:+.2}").replace('.', &pgettext("decimal separator", "."))
        });
        b.append(&ls::zeile(&gettext("Speed:"), &geschwindigkeit));
        let beschleunigung =
            gtk4::CheckButton::with_label(&gettext("Acceleration (fast movements travel further)"));
        b.append(&beschleunigung);
        inhalt.append(&f);

        let (f, b) = ls::rahmen(&gettext("Buttons and Wheel"));
        let doppelklick = gtk4::SpinButton::with_range(100.0, 2000.0, 50.0);
        b.append(&ls::zeile(
            &gettext("Double-click timeout (milliseconds):"),
            &doppelklick,
        ));
        b.append(&ls::grau(&gettext(
            "Applies to labwc (title bars, window frames); GTK programs have their own setting.",
        )));
        let linkshaendig =
            gtk4::CheckButton::with_label(&gettext("Left-handed (swap left and right buttons)"));
        b.append(&linkshaendig);
        let natuerlich = gtk4::CheckButton::with_label(&gettext(
            "Natural scrolling (content follows the fingers)",
        ));
        b.append(&natuerlich);
        b.append(&ls::grau(&gettext(
            "Applies to all mice and touchpads without their own profile in rc.xml.",
        )));
        inhalt.append(&f);

        let (f, b) = ls::rahmen(&gettext("Mouse Pointer"));
        let themen_kasten = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        b.append(&ls::zeile(&gettext("Theme:"), &themen_kasten));
        let gr = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let mut groessen: Vec<gtk4::CheckButton> = Vec::new();
        for (_, text) in GROESSEN {
            let k = gtk4::CheckButton::with_label(&gettext(*text));
            if let Some(erste) = groessen.first() {
                k.set_group(Some(erste));
            }
            gr.append(&k);
            groessen.push(k);
        }
        b.append(&ls::zeile(&gettext("Size:"), &gr));
        b.append(&ls::grau(&gettext("Pointer of labwc (window frames, desktop) and GTK programs immediately after “Apply”, in other programs from the next login.")));
        inhalt.append(&f);

        let m = Maus {
            doppelklick,
            geschwindigkeit,
            beschleunigung,
            linkshaendig,
            natuerlich,
            themen_kasten,
            themen: std::cell::RefCell::new(Vec::new()),
            groessen,
            eigene_groesse: std::cell::Cell::new(24),
            geaendert: geaendert.clone(),
        };
        let g = geaendert.clone();
        m.doppelklick.connect_value_changed(move |_| g());
        let g = geaendert.clone();
        m.geschwindigkeit.connect_value_changed(move |_| g());
        for k in [&m.beschleunigung, &m.linkshaendig, &m.natuerlich] {
            let g = geaendert.clone();
            k.connect_toggled(move |_| g());
        }
        for k in &m.groessen {
            let g = geaendert.clone();
            k.connect_toggled(move |k| {
                if k.is_active() {
                    g();
                }
            });
        }
        (m, inhalt)
    }

    fn zeigen(&self, w: &Werte) {
        self.doppelklick.set_value(w.doppelklick as f64);
        self.geschwindigkeit
            .set_value(w.geschwindigkeit as f64 / 100.0);
        self.beschleunigung.set_active(w.beschleunigung);
        self.linkshaendig.set_active(w.linkshaendig);
        self.natuerlich.set_active(w.natuerlich);
        self.themen_setzen(&w.zeigerthema);
        let i = GROESSEN.iter().position(|(g, _)| *g == w.zeigergroesse);
        match i {
            Some(i) => self.groessen[i].set_active(true),
            None => {
                // eigene Größe aus der Datei: nichts gewählt, Wert bleibt
                for k in &self.groessen {
                    k.set_active(false);
                }
            }
        }
        self.eigene_groesse.set(w.zeigergroesse);
    }

    fn aktuell(&self) -> Werte {
        let thema = self
            .themen
            .borrow()
            .iter()
            .find(|(_, k)| k.is_active())
            .map(|(t, _)| t.clone())
            .unwrap_or_default();
        let groesse = self
            .groessen
            .iter()
            .position(|k| k.is_active())
            .map_or(self.eigene_groesse.get(), |i| GROESSEN[i].0);
        Werte {
            doppelklick: self.doppelklick.value_as_int().max(1) as u32,
            geschwindigkeit: (self.geschwindigkeit.value() * 100.0).round() as i32,
            beschleunigung: self.beschleunigung.is_active(),
            linkshaendig: self.linkshaendig.is_active(),
            natuerlich: self.natuerlich.is_active(),
            zeigerthema: thema,
            zeigergroesse: groesse,
        }
    }
}

impl Maus {
    /// Optionsknöpfe: „Standard“, installierte Themen, ein fremder Wert aus der Datei.
    fn themen_setzen(&self, aktuell: &str) {
        let mut namen = vec![String::new()];
        namen.extend(zeigerthemen());
        if !namen.iter().any(|n| n == aktuell) {
            namen.push(aktuell.to_owned());
        }
        let liste: Vec<(String, String)> = namen
            .into_iter()
            .map(|n| {
                let text = if n.is_empty() {
                    gettext("Default")
                } else {
                    n.clone()
                };
                (n, text)
            })
            .collect();
        ls::optionen(
            &self.themen_kasten,
            &self.themen,
            &liste,
            aktuell,
            &self.geaendert,
        );
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
            rc_basis: rc.into(),
            env: BTreeMap::new(),
            env_ohne_eigene: BTreeMap::new(),
            vorgabe_neu: false,
            fehler: None,
            gs: BTreeMap::new(),
        }
    }

    #[test]
    fn lesen_standard_und_datei() {
        let w = Maus::lesen(&stand("<labwc_config/>"));
        assert_eq!(
            (
                w.doppelklick,
                w.geschwindigkeit,
                w.beschleunigung,
                w.linkshaendig,
                w.natuerlich
            ),
            (500, 0, true, false, false)
        );
        assert_eq!((w.zeigerthema.as_str(), w.zeigergroesse), ("", 24));
        let rc = "<labwc_config><mouse><doubleClickTime>350</doubleClickTime></mouse><libinput><device category=\"touchpad\"><naturalScroll>yes</naturalScroll></device><device><pointerSpeed>-0.5</pointerSpeed><accelProfile>flat</accelProfile><leftHanded>yes</leftHanded></device></libinput></labwc_config>";
        let w = Maus::lesen(&stand(rc));
        assert_eq!(
            (
                w.doppelklick,
                w.geschwindigkeit,
                w.beschleunigung,
                w.linkshaendig,
                w.natuerlich
            ),
            (350, -50, false, true, false)
        );
    }

    #[test]
    fn schreiben_nur_geaendertes() {
        let st = stand("<labwc_config>\n</labwc_config>\n");
        let alt = Maus::lesen(&st);
        let mut neu = alt.clone();
        neu.geschwindigkeit = 35;
        neu.natuerlich = true;
        neu.doppelklick = 400;
        let rc = Maus::rc_aendern(&neu, &alt, &st.rc).unwrap();
        assert_eq!(
            lx::wert(&rc, lx::GERAET, "pointerSpeed").as_deref(),
            Some("0.35")
        );
        assert_eq!(
            lx::wert(&rc, lx::GERAET, "naturalScroll").as_deref(),
            Some("yes")
        );
        assert_eq!(
            lx::wert(&rc, lx::MAUS, "doubleClickTime").as_deref(),
            Some("400")
        );
        assert_eq!(lx::wert(&rc, lx::GERAET, "leftHanded"), None);
        assert_eq!(Maus::lesen(&stand(&rc)), neu);
        neu.zeigergroesse = 48;
        neu.zeigerthema = "Adwaita".into();
        assert_eq!(
            Maus::env_aenderungen(&neu, &alt, &st),
            vec![
                ("XCURSOR_THEME", Some("Adwaita".into())),
                ("XCURSOR_SIZE", Some("48".into()))
            ]
        );
    }
}
