//! Bereich „Tastatur“ (G2-AG, QD-046 ③, QD-057): Belegungen (Umgebung
//! `XKB_DEFAULT_LAYOUT`/`_VARIANT`/`_OPTIONS` in `environment.d`),
//! Tastenwiederholung und Num-Taste (`rc.xml <keyboard>`). labwc übernimmt
//! Belegung und Wiederholung mit `--reconfigure` sofort (labwc 0.9.8 liest die
//! Umgebung beim Neuladen neu und setzt die Tastaturbelegung, `keyboard.c
//! set_layout`); die Num-Taste setzt es nur beim Erkennen einer Tastatur
//! (`seat.c`, `new_keyboard`), also ab der nächsten Anmeldung oder beim
//! Anstecken.
//!
//! Einstellungen 0.1.0 (QD-057 Rest): mehrere Belegungen in fester
//! Reihenfolge (`XKB_DEFAULT_LAYOUT=de,us`, Varianten parallel) und eine
//! Umschalttaste (`XKB_DEFAULT_OPTIONS=grp:…`, xkbcommon wechselt selbst);
//! fremde Optionen in `XKB_DEFAULT_OPTIONS` (z. B. `compose:ralt`) bleiben
//! stehen. Die erste Belegung zeigt der Kasten oben wie bisher, die weiteren
//! und die Umschalttaste stehen in einem eigenen Kasten unten (die Lage der
//! übrigen Bedienelemente bleibt gleich). Einen Belegungsanzeiger hat das
//! Panel noch nicht (Folgepunkt Runtime).
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n::{gettext, gettext_noop};
use crate::labwc_konfig::Stand;
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::{self as lx, Fehler};
use crate::xkb;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    /// Belegungen in Reihenfolge (XKB-Name, Variante); die erste gilt nach
    /// der Anmeldung. `("", "")` = keine Angabe (labwc nimmt `us`).
    pub belegungen: Vec<(String, String)>,
    /// Umschalttaste (`grp:…`), leer = keine
    pub umschalten: String,
    /// übrige Optionen aus `XKB_DEFAULT_OPTIONS`, unverändert weitergegeben
    pub optionen: Vec<String>,
    /// Zeichen pro Sekunde (labwc-Standard 25)
    pub rate: u32,
    /// Millisekunden (labwc-Standard 600)
    pub verzoegerung: u32,
    pub numlock: bool,
}

impl Werte {
    /// `XKB_DEFAULT_OPTIONS` wie geschrieben: ohne zweite Belegung keine
    /// Umschalttaste.
    fn optionen_text(&self) -> String {
        let grp = if self.belegungen.len() > 1 {
            self.umschalten.as_str()
        } else {
            ""
        };
        xkb::optionen_zusammen(grp, &self.optionen)
    }
}

fn zahl(st: &Stand, schluessel: &str, standard: u32) -> u32 {
    lx::wert(&st.rc, lx::TASTATUR, schluessel)
        .and_then(|v| v.parse().ok())
        .unwrap_or(standard)
}

pub fn ja(v: Option<String>) -> bool {
    v.is_some_and(|v| matches!(v.to_ascii_lowercase().as_str(), "yes" | "on" | "true" | "1"))
}

/// Name der Belegung mit XKB-Kürzel als Pango-Markup.
fn markup(liste: &[xkb::Belegung], l: &str, v: &str) -> String {
    format!(
        "<b>{}</b>  <span alpha=\"60%\">({}{})</span>",
        gtk4::glib::markup_escape_text(&xkb::anzeigename(liste, l, v)),
        gtk4::glib::markup_escape_text(if l.is_empty() { "us" } else { l }),
        if v.is_empty() {
            String::new()
        } else {
            format!(", {}", gtk4::glib::markup_escape_text(v))
        }
    )
}

/// Belegungen und Umschalttaste: Zustand und Bedienelemente, die beim
/// Ändern der Liste neu aufgebaut werden.
struct Belegungen {
    werte: RefCell<Vec<(String, String)>>,
    optionen: RefCell<Vec<String>>,
    /// erste Belegung (Kasten oben)
    name: gtk4::Label,
    /// Zeilen der weiteren Belegungen
    weitere: gtk4::Box,
    umschalten_raster: gtk4::Grid,
    umschalten: RefCell<Vec<(String, gtk4::CheckButton)>>,
    liste: Rc<Vec<xkb::Belegung>>,
    geaendert: Rc<dyn Fn()>,
}

impl Belegungen {
    fn umschalten_wahl(&self) -> String {
        self.umschalten
            .borrow()
            .iter()
            .find(|(_, k)| k.is_active())
            .map(|(s, _)| s.clone())
            .unwrap_or_default()
    }

    /// Zeigt Liste und Umschalttaste; `umschalten` = gewählte Option.
    fn zeigen(self: &Rc<Self>, umschalten: &str) {
        let b = self.werte.borrow().clone();
        let (l, v) = b.first().cloned().unwrap_or_default();
        self.name.set_markup(&markup(&self.liste, &l, &v));
        while let Some(c) = self.weitere.first_child() {
            self.weitere.remove(&c);
        }
        if b.len() < 2 {
            self.weitere
                .append(&ls::grau(&gettext("No additional layouts.")));
        }
        for (i, (l, v)) in b.iter().enumerate().skip(1) {
            let zeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
            let name = gtk4::Label::new(None);
            name.set_markup(&format!("{}. {}", i + 1, markup(&self.liste, l, v)));
            name.set_xalign(0.0);
            name.set_hexpand(true);
            name.set_wrap(true);
            zeile.append(&name);
            let hoch = gtk4::Button::with_label(&gettext("Move Up"));
            let runter = gtk4::Button::with_label(&gettext("Move Down"));
            runter.set_sensitive(i + 1 < b.len());
            let weg = gtk4::Button::with_label(&gettext("Remove"));
            for (knopf, aktion) in [(&hoch, -1i32), (&runter, 1), (&weg, 0)] {
                let s = self.clone();
                knopf.connect_clicked(move |_| s.aendern(i, aktion));
                zeile.append(knopf);
            }
            self.weitere.append(&zeile);
        }
        self.umschalten_zeigen(umschalten);
    }

    fn umschalten_zeigen(&self, aktuell: &str) {
        let mut liste: Vec<(String, String)> = vec![(String::new(), gettext("None"))];
        liste.extend(
            xkb::UMSCHALTER
                .iter()
                .map(|(o, t)| ((*o).to_owned(), gettext(*t))),
        );
        if !aktuell.is_empty() && !liste.iter().any(|(o, _)| o == aktuell) {
            let regeln = std::fs::read_to_string(xkb::REGELN).unwrap_or_default();
            let text = xkb::optionsbeschreibung(&regeln, aktuell)
                .map_or_else(|| aktuell.to_owned(), |b| format!("{b} ({aktuell})"));
            liste.push((aktuell.to_owned(), text));
        }
        ls::optionen_raster(
            &self.umschalten_raster,
            &self.umschalten,
            &liste,
            aktuell,
            &self.geaendert,
            2,
        );
        self.umschalten_raster
            .set_sensitive(self.werte.borrow().len() > 1);
    }

    /// Zeile `i` verschieben (`-1` hoch, `1` runter) oder entfernen (`0`).
    fn aendern(self: &Rc<Self>, i: usize, aktion: i32) {
        {
            let mut b = self.werte.borrow_mut();
            match aktion {
                0 if i < b.len() => {
                    b.remove(i);
                }
                -1 if i > 0 && i < b.len() => b.swap(i - 1, i),
                1 if i + 1 < b.len() => b.swap(i, i + 1),
                _ => return,
            }
        }
        let u = self.umschalten_wahl();
        self.zeigen(&u);
        (self.geaendert)();
    }

    /// Belegung anhängen; mit der zweiten Belegung wird eine Umschalttaste
    /// vorgeschlagen (Alt+Umschalt), falls noch keine gewählt ist.
    fn hinzufuegen(self: &Rc<Self>, l: String, v: String) {
        {
            let mut b = self.werte.borrow_mut();
            if b.iter().any(|x| *x == (l.clone(), v.clone())) {
                return;
            }
            b.push((l, v));
        }
        let mut u = self.umschalten_wahl();
        if u.is_empty() {
            u = xkb::UMSCHALTER[0].0.to_owned();
        }
        self.zeigen(&u);
        (self.geaendert)();
    }

    /// Erste Belegung ersetzen (Knopf „Ändern …“ oben).
    fn erste_setzen(self: &Rc<Self>, l: String, v: String) {
        {
            let mut b = self.werte.borrow_mut();
            // steht sie schon weiter unten, wird sie dort entfernt
            if let Some(i) = b.iter().skip(1).position(|x| *x == (l.clone(), v.clone())) {
                b.remove(i + 1);
            }
            match b.first_mut() {
                Some(e) => *e = (l, v),
                None => b.push((l, v)),
            }
        }
        let u = self.umschalten_wahl();
        self.zeigen(&u);
        (self.geaendert)();
    }
}

pub struct Tastatur {
    belegungen: Rc<Belegungen>,
    rate: gtk4::SpinButton,
    verzoegerung: gtk4::SpinButton,
    numlock: gtk4::CheckButton,
}

impl Bereich for Tastatur {
    type Werte = Werte;
    const NAME: &'static str = gettext_noop("Keyboard");
    const PROTOKOLL: &'static str = "Tastatur";
    const QUELLE: &'static str = gettext_noop(
        "~/.config/querencia/labwc: rc.xml <keyboard> (repeat, Num Lock), environment.d/zz-querencia-einstellungen.env (XKB_DEFAULT_LAYOUT, XKB_DEFAULT_VARIANT, XKB_DEFAULT_OPTIONS; default in environment).",
    );

    fn lesen(st: &Stand) -> Werte {
        let env = |k: &str| st.env.get(k).cloned().unwrap_or_default();
        let (umschalten, optionen) = xkb::optionen_zerlegen(&env("XKB_DEFAULT_OPTIONS"));
        Werte {
            belegungen: xkb::belegungen_zerlegen(
                &env("XKB_DEFAULT_LAYOUT"),
                &env("XKB_DEFAULT_VARIANT"),
            ),
            umschalten,
            optionen,
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
            t = lx::setzen(
                &t,
                lx::TASTATUR,
                "repeatDelay",
                &neu.verzoegerung.to_string(),
            )?;
        }
        if neu.numlock != alt.numlock {
            t = lx::setzen(
                &t,
                lx::TASTATUR,
                "numlock",
                if neu.numlock { "on" } else { "off" },
            )?;
        }
        Ok(t)
    }

    fn env_aenderungen(
        neu: &Werte,
        alt: &Werte,
        st: &Stand,
    ) -> Vec<(&'static str, Option<String>)> {
        let mut v = Vec::new();
        if neu.belegungen != alt.belegungen {
            let (layout, variante) = xkb::belegungen_zusammen(&neu.belegungen);
            v.push(ls::env_wert(st, "XKB_DEFAULT_LAYOUT", &layout));
            v.push(ls::env_wert(st, "XKB_DEFAULT_VARIANT", &variante));
        }
        let optionen = neu.optionen_text();
        if optionen != alt.optionen_text() {
            v.push(ls::env_wert(st, "XKB_DEFAULT_OPTIONS", &optionen));
        }
        v
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        let liste = Rc::new(xkb::liste());

        let (f, b) = ls::rahmen(&gettext("Keyboard Layout"));
        let kopf = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        let name = gtk4::Label::new(None);
        name.set_xalign(0.0);
        name.set_hexpand(true);
        name.set_wrap(true);
        let aendern = gtk4::Button::with_label(&gettext("Change …"));
        kopf.append(&name);
        kopf.append(&aendern);
        b.append(&kopf);
        b.append(&ls::grau(&gettext(
            "Applies to all programs. labwc applies the layout immediately with “Apply” as soon as the session reads the user copy.",
        )));
        inhalt.append(&f);

        let (f, b) = ls::rahmen(&gettext("Key Repeat"));
        let verzoegerung = gtk4::SpinButton::with_range(100.0, 2000.0, 10.0);
        b.append(&ls::zeile(
            &gettext("Delay before repeating (milliseconds):"),
            &verzoegerung,
        ));
        let rate = gtk4::SpinButton::with_range(1.0, 100.0, 1.0);
        b.append(&ls::zeile(
            &gettext("Speed (characters per second):"),
            &rate,
        ));
        let probe = gtk4::Entry::new();
        probe.set_placeholder_text(Some(&gettext("Try it here (after “Apply”)")));
        probe.set_hexpand(true);
        b.append(&probe);
        inhalt.append(&f);

        let numlock = gtk4::CheckButton::with_label(&gettext("Turn on Num Lock at login"));
        inhalt.append(&numlock);
        inhalt.append(&ls::grau(&gettext(
            "labwc sets Num Lock when a keyboard is detected (login, plugging in).",
        )));

        // QD-057: weitere Belegungen und Umschalttaste (unten, damit die
        // Bedienelemente darüber ihre Lage behalten)
        let (f, b) = ls::rahmen(&gettext("Additional Layouts and Switching"));
        b.append(&ls::grau(&gettext(
            "With several layouts the switch key changes to the next one in turn; after login the first one applies.",
        )));
        let weitere = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        b.append(&weitere);
        let hinzufuegen = gtk4::Button::with_label(&gettext("Add Layout …"));
        hinzufuegen.set_halign(gtk4::Align::Start);
        b.append(&hinzufuegen);
        let titel = gtk4::Label::new(Some(&gettext("Switch layouts with:")));
        titel.set_xalign(0.0);
        titel.set_margin_top(6);
        b.append(&titel);
        let umschalten_raster = gtk4::Grid::new();
        umschalten_raster.set_column_spacing(12);
        umschalten_raster.set_row_spacing(2);
        b.append(&umschalten_raster);
        b.append(&ls::grau(&gettext(
            "The panel does not show the active layout yet; a layout indicator follows with a later version of the panel.",
        )));
        inhalt.append(&f);

        let belegungen = Rc::new(Belegungen {
            werte: RefCell::new(Vec::new()),
            optionen: RefCell::new(Vec::new()),
            name,
            weitere,
            umschalten_raster,
            umschalten: RefCell::new(Vec::new()),
            liste,
            geaendert: geaendert.clone(),
        });
        {
            let bl = belegungen.clone();
            aendern.connect_clicked(move |k| {
                let eltern = k.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
                let aktuell = bl.werte.borrow().first().cloned().unwrap_or_default();
                let bl2 = bl.clone();
                auswahl(eltern, bl.liste.clone(), aktuell, move |l, v| {
                    bl2.erste_setzen(l, v)
                });
            });
        }
        {
            let bl = belegungen.clone();
            hinzufuegen.connect_clicked(move |k| {
                let eltern = k.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
                let bl2 = bl.clone();
                auswahl(
                    eltern,
                    bl.liste.clone(),
                    (String::new(), String::new()),
                    move |l, v| bl2.hinzufuegen(l, v),
                );
            });
        }
        let t = Tastatur {
            belegungen,
            rate,
            verzoegerung,
            numlock,
        };
        for s in [&t.rate, &t.verzoegerung] {
            let g = geaendert.clone();
            s.connect_value_changed(move |_| g());
        }
        let g = geaendert.clone();
        t.numlock.connect_toggled(move |_| g());
        (t, inhalt)
    }

    fn zeigen(&self, w: &Werte) {
        *self.belegungen.werte.borrow_mut() = w.belegungen.clone();
        *self.belegungen.optionen.borrow_mut() = w.optionen.clone();
        self.belegungen.zeigen(&w.umschalten);
        self.rate.set_value(w.rate as f64);
        self.verzoegerung.set_value(w.verzoegerung as f64);
        self.numlock.set_active(w.numlock);
    }

    fn aktuell(&self) -> Werte {
        Werte {
            belegungen: self.belegungen.werte.borrow().clone(),
            umschalten: self.belegungen.umschalten_wahl(),
            optionen: self.belegungen.optionen.borrow().clone(),
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
        .title(gettext("Choose Keyboard Layout"))
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
    suche.set_placeholder_text(Some(&gettext("Search, e.g. German, Switzerland, Dvorak")));
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
    let abbrechen = gtk4::Button::with_label(&gettext("Cancel"));
    let waehlen = gtk4::Button::with_label(&gettext("Select"));
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
                b.name.to_lowercase().contains(&t)
                    || b.layout == t
                    || b.variante.to_lowercase() == t
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
                    eprintln!(
                        "querencia-einstellungen: Tastatur: gewählt {} ({}:{})",
                        b.name, b.layout, b.variante
                    );
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
        let m = |l: &[(&str, &str)]| {
            l.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<BTreeMap<_, _>>()
        };
        Stand {
            rc: rc.into(),
            quelle: Quelle::Benutzer,
            rc_vorgabe: rc.into(),
            env: m(env),
            env_ohne_eigene: m(vorgabe_env),
            vorgabe_neu: false,
            fehler: None,
            gs: BTreeMap::new(),
        }
    }

    fn b(l: &str, v: &str) -> (String, String) {
        (l.into(), v.into())
    }

    #[test]
    fn lesen_mit_standardwerten() {
        let st = stand(
            "<labwc_config/>",
            &[("XKB_DEFAULT_LAYOUT", "de")],
            &[("XKB_DEFAULT_LAYOUT", "de")],
        );
        let w = Tastatur::lesen(&st);
        assert_eq!(
            w,
            Werte {
                belegungen: vec![b("de", "")],
                umschalten: String::new(),
                optionen: Vec::new(),
                rate: 25,
                verzoegerung: 600,
                numlock: false
            }
        );
    }

    #[test]
    fn mehrere_belegungen_und_optionen_lesen() {
        let st = stand(
            "<labwc_config/>",
            &[
                ("XKB_DEFAULT_LAYOUT", "de,us"),
                ("XKB_DEFAULT_VARIANT", "nodeadkeys"),
                ("XKB_DEFAULT_OPTIONS", "compose:ralt,grp:alt_shift_toggle"),
            ],
            &[("XKB_DEFAULT_LAYOUT", "de")],
        );
        let w = Tastatur::lesen(&st);
        assert_eq!(w.belegungen, vec![b("de", "nodeadkeys"), b("us", "")]);
        assert_eq!(w.umschalten, "grp:alt_shift_toggle");
        assert_eq!(w.optionen, vec!["compose:ralt".to_string()]);
        // unverändert gelesen = nichts zu schreiben
        assert!(Tastatur::env_aenderungen(&w, &w, &st).is_empty());
    }

    #[test]
    fn nur_geaenderte_werte_schreiben() {
        let st = stand(
            "<labwc_config>\n</labwc_config>\n",
            &[("XKB_DEFAULT_LAYOUT", "de")],
            &[("XKB_DEFAULT_LAYOUT", "de")],
        );
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
        neu.belegungen = vec![b("us", "intl")];
        assert_eq!(
            Tastatur::env_aenderungen(&neu, &alt, &st),
            vec![
                ("XKB_DEFAULT_LAYOUT", Some("us".into())),
                ("XKB_DEFAULT_VARIANT", Some("intl".into()))
            ]
        );
        let zurueck = Werte {
            belegungen: vec![b("de", "")],
            ..neu.clone()
        };
        assert_eq!(
            Tastatur::env_aenderungen(&zurueck, &neu, &st),
            vec![("XKB_DEFAULT_LAYOUT", None), ("XKB_DEFAULT_VARIANT", None)]
        );
    }

    #[test]
    fn zweite_belegung_mit_umschalttaste() {
        let st = stand(
            "<labwc_config/>",
            &[("XKB_DEFAULT_LAYOUT", "de")],
            &[("XKB_DEFAULT_LAYOUT", "de")],
        );
        let alt = Tastatur::lesen(&st);
        let neu = Werte {
            belegungen: vec![b("de", ""), b("us", "intl")],
            umschalten: "grp:alt_shift_toggle".into(),
            ..alt.clone()
        };
        assert_eq!(
            Tastatur::env_aenderungen(&neu, &alt, &st),
            vec![
                ("XKB_DEFAULT_LAYOUT", Some("de,us".into())),
                ("XKB_DEFAULT_VARIANT", Some(",intl".into())),
                ("XKB_DEFAULT_OPTIONS", Some("grp:alt_shift_toggle".into())),
            ]
        );
        // Reihenfolge tauschen: nur die Listen ändern sich
        let getauscht = Werte {
            belegungen: vec![b("us", "intl"), b("de", "")],
            ..neu.clone()
        };
        assert_eq!(
            Tastatur::env_aenderungen(&getauscht, &neu, &st),
            vec![
                ("XKB_DEFAULT_LAYOUT", Some("us,de".into())),
                ("XKB_DEFAULT_VARIANT", Some("intl,".into())),
            ]
        );
        // zurück auf eine Belegung: Umschalttaste fällt weg, fremde Optionen bleiben
        let mit_fremd = Werte {
            optionen: vec!["compose:ralt".into()],
            ..neu.clone()
        };
        let eine = Werte {
            belegungen: vec![b("de", "")],
            ..mit_fremd.clone()
        };
        assert_eq!(
            Tastatur::env_aenderungen(&eine, &mit_fremd, &st),
            vec![
                ("XKB_DEFAULT_LAYOUT", None),
                ("XKB_DEFAULT_VARIANT", None),
                ("XKB_DEFAULT_OPTIONS", Some("compose:ralt".into())),
            ]
        );
    }

    #[test]
    fn leere_erste_belegung_wird_us() {
        assert_eq!(
            xkb::belegungen_zusammen(&[b("", ""), b("de", "")]),
            ("us,de".into(), String::new())
        );
        assert_eq!(
            xkb::belegungen_zusammen(&[b("", "")]),
            (String::new(), String::new())
        );
        assert_eq!(xkb::belegungen_zerlegen("", ""), vec![b("", "")]);
    }

    #[test]
    fn umschalter_sind_xkb_optionen() {
        let regeln = std::fs::read_to_string(xkb::REGELN).unwrap_or_default();
        for (o, _) in xkb::UMSCHALTER {
            assert!(o.starts_with("grp:"), "{o}");
            if !regeln.is_empty() {
                assert!(
                    xkb::optionsbeschreibung(&regeln, o).is_some(),
                    "{o} fehlt in evdev.xml"
                );
            }
        }
        assert!(!xkb::UMSCHALTER.iter().any(|(o, _)| *o == "grp:toggle"));
    }
}
