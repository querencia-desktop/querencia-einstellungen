//! Bereich „Tastatur“ (G2-AG, QD-046 ③, QD-057): Belegung (Umgebung
//! `XKB_DEFAULT_LAYOUT`/`_VARIANT` in `environment.d`), Tastenwiederholung und
//! Num-Taste (`rc.xml <keyboard>`). labwc übernimmt alles mit
//! `--reconfigure` sofort, auch die Belegung (labwc 0.9.8 liest die Umgebung
//! beim Neuladen neu und setzt die Tastaturbelegung, `keyboard.c set_layout`).
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::labwc_konfig::Stand;
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::{self as lx, Fehler};
use crate::xkb;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    pub layout: String,
    pub variante: String,
    /// Zeichen pro Sekunde (labwc-Standard 25)
    pub rate: u32,
    /// Millisekunden (labwc-Standard 600)
    pub verzoegerung: u32,
    pub numlock: bool,
}

fn zahl(st: &Stand, schluessel: &str, standard: u32) -> u32 {
    lx::wert(&st.rc, lx::TASTATUR, schluessel)
        .and_then(|v| v.parse().ok())
        .unwrap_or(standard)
}

pub fn ja(v: Option<String>) -> bool {
    v.is_some_and(|v| matches!(v.to_ascii_lowercase().as_str(), "yes" | "on" | "true" | "1"))
}

pub struct Tastatur {
    belegung: Rc<RefCell<(String, String)>>,
    name: gtk4::Label,
    rate: gtk4::SpinButton,
    verzoegerung: gtk4::SpinButton,
    numlock: gtk4::CheckButton,
    liste: Rc<Vec<xkb::Belegung>>,
}

/// Name der Belegung (deutsch) mit XKB-Kürzel.
fn namen_zeigen(name: &gtk4::Label, liste: &[xkb::Belegung], belegung: &(String, String)) {
    let (l, v) = belegung;
    name.set_markup(&format!(
        "<b>{}</b>  <span alpha=\"60%\">({}{})</span>",
        gtk4::glib::markup_escape_text(&xkb::anzeigename(liste, l, v)),
        gtk4::glib::markup_escape_text(if l.is_empty() { "us" } else { l }),
        if v.is_empty() { String::new() } else { format!(", {v}") }
    ));
}

impl Bereich for Tastatur {
    type Werte = Werte;
    const NAME: &'static str = "Tastatur";
    const QUELLE: &'static str = "~/.config/querencia/labwc: rc.xml <keyboard> (Wiederholung, Num-Taste), environment.d/querencia-einstellungen.env (XKB_DEFAULT_LAYOUT, XKB_DEFAULT_VARIANT; Vorgabe in environment).";

    fn lesen(st: &Stand) -> Werte {
        Werte {
            layout: st.env.get("XKB_DEFAULT_LAYOUT").cloned().unwrap_or_default(),
            variante: st.env.get("XKB_DEFAULT_VARIANT").cloned().unwrap_or_default(),
            rate: zahl(st, "repeatRate", 25),
            verzoegerung: zahl(st, "repeatDelay", 600),
            numlock: ja(lx::wert(&st.rc, lx::TASTATUR, "numlock")),
        }
    }

    fn rc_aendern(neu: &Werte, alt: &Werte, rc: &str) -> Result<String, Fehler> {
        let mut t = rc.to_owned();
        if neu.rate != alt.rate {
            t = lx::setzen(&t, lx::TASTATUR, "repeatRate", &neu.rate.to_string())?;
        }
        if neu.verzoegerung != alt.verzoegerung {
            t = lx::setzen(&t, lx::TASTATUR, "repeatDelay", &neu.verzoegerung.to_string())?;
        }
        if neu.numlock != alt.numlock {
            t = lx::setzen(&t, lx::TASTATUR, "numlock", if neu.numlock { "on" } else { "off" })?;
        }
        Ok(t)
    }

    fn env_aenderungen(neu: &Werte, alt: &Werte, st: &Stand) -> Vec<(&'static str, Option<String>)> {
        if (&neu.layout, &neu.variante) == (&alt.layout, &alt.variante) {
            return Vec::new();
        }
        vec![
            ls::env_wert(st, "XKB_DEFAULT_LAYOUT", &neu.layout),
            ls::env_wert(st, "XKB_DEFAULT_VARIANT", &neu.variante),
        ]
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        let liste = Rc::new(xkb::liste());

        let (f, b) = ls::rahmen("Tastaturbelegung");
        let kopf = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        let name = gtk4::Label::new(None);
        name.set_xalign(0.0);
        name.set_hexpand(true);
        name.set_wrap(true);
        let aendern = gtk4::Button::with_label("Ändern …");
        kopf.append(&name);
        kopf.append(&aendern);
        b.append(&kopf);
        b.append(&ls::grau("Gilt für alle Programme. labwc übernimmt die Belegung mit „Übernehmen“ sofort, sobald die Sitzung die Benutzerkopie liest."));
        inhalt.append(&f);

        let (f, b) = ls::rahmen("Tastenwiederholung");
        let verzoegerung = gtk4::SpinButton::with_range(100.0, 2000.0, 10.0);
        b.append(&ls::zeile("Verzögerung bis zur Wiederholung (Millisekunden):", &verzoegerung));
        let rate = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
        b.append(&ls::zeile("Geschwindigkeit (Zeichen pro Sekunde):", &rate));
        let probe = gtk4::Entry::new();
        probe.set_placeholder_text(Some("Hier ausprobieren (nach „Übernehmen“)"));
        probe.set_hexpand(true);
        b.append(&probe);
        inhalt.append(&f);

        let numlock = gtk4::CheckButton::with_label("Num-Taste beim Anmelden einschalten");
        inhalt.append(&numlock);
        inhalt.append(&ls::grau("Die Num-Taste setzt labwc, wenn eine Tastatur erkannt wird (Anmeldung, Anstecken)."));

        let t = Tastatur {
            belegung: Rc::new(RefCell::new((String::new(), String::new()))),
            name,
            rate,
            verzoegerung,
            numlock,
            liste,
        };
        {
            let (belegung, name, liste, g) = (t.belegung.clone(), t.name.clone(), t.liste.clone(), geaendert.clone());
            aendern.connect_clicked(move |k| {
                let eltern = k.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
                let (belegung, name, liste2, g) = (belegung.clone(), name.clone(), liste.clone(), g.clone());
                let aktuell = belegung.borrow().clone();
                auswahl(eltern, liste.clone(), aktuell, move |l, v| {
                    *belegung.borrow_mut() = (l, v);
                    namen_zeigen(&name, &liste2, &belegung.borrow());
                    g();
                });
            });
        }
        for s in [&t.rate, &t.verzoegerung] {
            let g = geaendert.clone();
            s.connect_value_changed(move |_| g());
        }
        let g = geaendert.clone();
        t.numlock.connect_toggled(move |_| g());
        (t, inhalt)
    }

    fn zeigen(&self, w: &Werte) {
        *self.belegung.borrow_mut() = (w.layout.clone(), w.variante.clone());
        namen_zeigen(&self.name, &self.liste, &self.belegung.borrow());
        self.rate.set_value(w.rate as f64);
        self.verzoegerung.set_value(w.verzoegerung as f64);
        self.numlock.set_active(w.numlock);
    }

    fn aktuell(&self) -> Werte {
        let (layout, variante) = self.belegung.borrow().clone();
        Werte {
            layout,
            variante,
            rate: self.rate.value_as_int().max(1) as u32,
            verzoegerung: self.verzoegerung.value_as_int().max(1) as u32,
            numlock: self.numlock.is_active(),
        }
    }
}

/// Auswahldialog: Suchfeld und Liste aller Belegungen.
pub fn auswahl(
    eltern: Option<gtk4::Window>,
    liste: Rc<Vec<xkb::Belegung>>,
    aktuell: (String, String),
    fertig: impl Fn(String, String) + 'static,
) {
    let fenster = gtk4::Window::builder()
        .title("Tastaturbelegung wählen")
        .modal(true)
        .default_width(520)
        .default_height(520)
        .build();
    fenster.set_transient_for(eltern.as_ref());
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    aussen.set_margin_top(10);
    aussen.set_margin_bottom(10);
    aussen.set_margin_start(10);
    aussen.set_margin_end(10);
    let suche = gtk4::SearchEntry::new();
    suche.set_placeholder_text(Some("Suchen, z. B. Deutsch, Schweiz, Dvorak"));
    aussen.append(&suche);
    let lb = gtk4::ListBox::new();
    lb.set_selection_mode(gtk4::SelectionMode::Single);
    let mut auswahl_zeile = None;
    for (i, b) in liste.iter().enumerate() {
        let l = gtk4::Label::new(Some(&b.name));
        l.set_xalign(0.0);
        l.set_margin_top(3);
        l.set_margin_bottom(3);
        l.set_margin_start(6);
        let z = gtk4::ListBoxRow::new();
        z.set_child(Some(&l));
        z.set_widget_name(&i.to_string());
        lb.append(&z);
        if (b.layout.as_str(), b.variante.as_str()) == (aktuell.0.as_str(), aktuell.1.as_str()) {
            auswahl_zeile = Some(z);
        }
    }
    let rollbar = gtk4::ScrolledWindow::new();
    rollbar.set_child(Some(&lb));
    rollbar.set_vexpand(true);
    aussen.append(&rollbar);
    let knoepfe = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    knoepfe.set_halign(gtk4::Align::End);
    let abbrechen = gtk4::Button::with_label("Abbrechen");
    let waehlen = gtk4::Button::with_label("Auswählen");
    waehlen.add_css_class("suggested-action");
    knoepfe.append(&abbrechen);
    knoepfe.append(&waehlen);
    aussen.append(&knoepfe);
    fenster.set_child(Some(&aussen));

    let l2 = liste.clone();
    let s2 = suche.clone();
    lb.set_filter_func(move |z| {
        let t = s2.text().to_lowercase();
        let i: usize = z.widget_name().parse().unwrap_or(0);
        t.is_empty()
            || l2.get(i).is_some_and(|b| {
                b.name.to_lowercase().contains(&t) || b.layout == t || b.variante.to_lowercase() == t
            })
    });
    let lb2 = lb.clone();
    suche.connect_search_changed(move |_| lb2.invalidate_filter());
    // Tippen landet immer im Suchfeld; die gewählte Zeile ist sichtbar.
    suche.set_key_capture_widget(Some(&fenster));
    if let Some(z) = auswahl_zeile {
        lb.select_row(Some(&z));
    }
    let (s3, lb3) = (suche.clone(), lb.clone());
    gtk4::glib::idle_add_local_once(move || {
        if let Some(z) = lb3.selected_row() {
            z.grab_focus();
        }
        s3.grab_focus();
    });
    let f2 = fenster.clone();
    abbrechen.connect_clicked(move |_| f2.close());
    let fertig = Rc::new(fertig);
    let annehmen = {
        let f2 = fenster.clone();
        let lb = lb.clone();
        let liste = liste.clone();
        let fertig = fertig.clone();
        move || {
            if let Some(z) = lb.selected_row() {
                let i: usize = z.widget_name().parse().unwrap_or(0);
                if let Some(b) = liste.get(i) {
                    eprintln!("querencia-einstellungen: Tastatur: gewählt {} ({}:{})", b.name, b.layout, b.variante);
                    fertig(b.layout.clone(), b.variante.clone());
                }
            }
            f2.close();
        }
    };
    let a2 = Rc::new(annehmen);
    let a3 = a2.clone();
    waehlen.connect_clicked(move |_| a3());
    lb.connect_row_activated(move |_, _| a2());
    fenster.present();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labwc_konfig::Quelle;
    use std::collections::BTreeMap;

    fn stand(rc: &str, env: &[(&str, &str)], vorgabe_env: &[(&str, &str)]) -> Stand {
        let m = |l: &[(&str, &str)]| l.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<BTreeMap<_, _>>();
        Stand {
            rc: rc.into(),
            quelle: Quelle::Benutzer,
            rc_vorgabe: rc.into(),
            env: m(env),
            env_ohne_eigene: m(vorgabe_env),
            vorgabe_neu: false,
        }
    }

    #[test]
    fn lesen_mit_standardwerten() {
        let st = stand("<labwc_config/>", &[("XKB_DEFAULT_LAYOUT", "de")], &[("XKB_DEFAULT_LAYOUT", "de")]);
        let w = Tastatur::lesen(&st);
        assert_eq!(
            w,
            Werte {
                layout: "de".into(),
                variante: String::new(),
                rate: 25,
                verzoegerung: 600,
                numlock: false
            }
        );
    }

    #[test]
    fn nur_geaenderte_werte_schreiben() {
        let st = stand("<labwc_config>\n</labwc_config>\n", &[("XKB_DEFAULT_LAYOUT", "de")], &[("XKB_DEFAULT_LAYOUT", "de")]);
        let alt = Tastatur::lesen(&st);
        let mut neu = alt.clone();
        neu.rate = 40;
        neu.numlock = true;
        let rc = Tastatur::rc_aendern(&neu, &alt, &st.rc).unwrap();
        assert_eq!(
            rc,
            "<labwc_config>\n  <keyboard>\n    <repeatRate>40</repeatRate>\n    <numlock>on</numlock>\n  </keyboard>\n</labwc_config>\n"
        );
        assert!(Tastatur::env_aenderungen(&neu, &alt, &st).is_empty());
        // Belegung: Abweichung setzen, Rückkehr zur Vorgabe entfernt
        neu.layout = "us".into();
        neu.variante = "intl".into();
        assert_eq!(
            Tastatur::env_aenderungen(&neu, &alt, &st),
            vec![("XKB_DEFAULT_LAYOUT", Some("us".into())), ("XKB_DEFAULT_VARIANT", Some("intl".into()))]
        );
        let zurueck = Werte { layout: "de".into(), variante: String::new(), ..neu.clone() };
        assert_eq!(
            Tastatur::env_aenderungen(&zurueck, &neu, &st),
            vec![("XKB_DEFAULT_LAYOUT", None), ("XKB_DEFAULT_VARIANT", None)]
        );
    }
}
