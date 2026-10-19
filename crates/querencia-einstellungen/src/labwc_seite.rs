//! Gemeinsames Gerüst der Bereiche, die labwc einstellen (G2-AG): Tastatur,
//! Maus, Fensterdesign, Arbeitsflächen. Bedienmuster wie „Symbole der
//! Arbeitsfläche“ (G2-X): Ändern → „Übernehmen“ schreibt (atomar, mit
//! Sicherung), „Standard wiederherstellen“ zeigt die Vorgabe des
//! Runtime-Repos, „Neu laden“ liest die Dateien neu. Jeder Bereich ändert
//! nur seine eigenen Werte auf dem **aktuellen** Dateistand; was andere
//! Bereiche inzwischen geschrieben haben, bleibt stehen.
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;

use crate::labwc_konfig::{self as lk, Orte, Stand, Wirkung};
use crate::labwc_xml::Fehler;

/// Ein Bereich: Werte lesen und schreiben, Oberfläche zeigen.
pub trait Bereich: Sized + 'static {
    type Werte: Clone + PartialEq + Debug;
    const NAME: &'static str;
    const QUELLE: &'static str;
    /// Die wirksamen Werte aus rc.xml und Umgebung.
    fn lesen(st: &Stand) -> Self::Werte;
    /// rc.xml mit den geänderten Werten (nur, was sich gegenüber `alt` unterscheidet).
    fn rc_aendern(neu: &Self::Werte, alt: &Self::Werte, rc: &str) -> Result<String, Fehler>;
    /// Umgebung: `Some` setzen, `None` = eigene Angabe entfernen (Vorgabe gilt).
    fn env_aenderungen(
        _neu: &Self::Werte,
        _alt: &Self::Werte,
        _st: &Stand,
    ) -> Vec<(&'static str, Option<String>)> {
        Vec::new()
    }
    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box);
    fn zeigen(&self, w: &Self::Werte);
    fn aktuell(&self) -> Self::Werte;
}

/// Die Werte der Vorgabe (rc.xml und Umgebung ohne die eigene Datei).
pub fn standard<B: Bereich>(st: &Stand) -> B::Werte {
    let mut v = st.clone();
    v.rc = st.rc_vorgabe.clone();
    v.env = st.env_ohne_eigene.clone();
    B::lesen(&v)
}

/// Für Umgebungswerte: entfernen, wenn gleich der Vorgabe, sonst setzen.
pub fn env_wert(
    st: &Stand,
    schluessel: &'static str,
    wert: &str,
) -> (&'static str, Option<String>) {
    let vorgabe = st
        .env_ohne_eigene
        .get(schluessel)
        .map_or("", String::as_str);
    (schluessel, (vorgabe != wert).then(|| wert.to_owned()))
}

struct Seite<B: Bereich> {
    bereich: B,
    orte: Orte,
    hinweis: gtk4::Label,
    vorgabe_kasten: gtk4::Frame,
    status: gtk4::Label,
    uebernehmen: gtk4::Button,
    geladen: RefCell<Option<B::Werte>>,
    stand: RefCell<Option<Stand>>,
    stumm: std::cell::Cell<bool>,
}

pub fn rahmen(titel: &str) -> (gtk4::Frame, gtk4::Box) {
    let f = gtk4::Frame::new(Some(titel));
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    b.set_margin_top(6);
    b.set_margin_bottom(6);
    b.set_margin_start(8);
    b.set_margin_end(8);
    f.set_child(Some(&b));
    (f, b)
}

pub fn grau(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.set_xalign(0.0);
    l.set_wrap(true);
    l.set_max_width_chars(70);
    l.add_css_class("dim-label");
    l
}

/// Optionsknöpfe (Schlüssel, Beschriftung) neu aufbauen, `aktuell` gewählt.
/// Ein unsichtbarer Anker hält die Gruppe zusammen, damit auch eine einzige
/// Möglichkeit als Optionsknopf (nicht als Kästchen) erscheint.
pub fn optionen(
    kasten: &gtk4::Box,
    ziel: &RefCell<Vec<(String, gtk4::CheckButton)>>,
    liste: &[(String, String)],
    aktuell: &str,
    geaendert: &Rc<dyn Fn()>,
) {
    while let Some(c) = kasten.first_child() {
        kasten.remove(&c);
    }
    let anker = gtk4::CheckButton::new();
    anker.set_visible(false);
    kasten.append(&anker);
    let mut knoepfe: Vec<(String, gtk4::CheckButton)> = Vec::new();
    for (schluessel, text) in liste {
        let k = gtk4::CheckButton::with_label(text);
        k.set_group(Some(&anker));
        k.set_active(schluessel == aktuell);
        let g = geaendert.clone();
        k.connect_toggled(move |k| {
            if k.is_active() {
                g();
            }
        });
        kasten.append(&k);
        knoepfe.push((schluessel.clone(), k));
    }
    if !knoepfe.iter().any(|(_, k)| k.is_active()) {
        anker.set_active(true);
    }
    *ziel.borrow_mut() = knoepfe;
}

/// Beschriftung links, Element rechts.
pub fn zeile(text: &str, w: &impl IsA<gtk4::Widget>) -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let l = gtk4::Label::new(Some(text));
    l.set_xalign(0.0);
    b.append(&l);
    b.append(w);
    b
}

fn jetzt_text(format: &str) -> String {
    glib::DateTime::now_local()
        .ok()
        .and_then(|d| d.format(format).ok())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// Seite für das Kontrollzentrum.
pub fn seite<B: Bereich>() -> gtk4::Widget {
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    aussen.set_margin_top(12);
    aussen.set_margin_bottom(12);
    aussen.set_margin_start(14);
    aussen.set_margin_end(14);
    let ueberschrift = gtk4::Label::new(None);
    ueberschrift.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(B::NAME)
    ));
    ueberschrift.set_xalign(0.0);
    aussen.append(&ueberschrift);

    let hinweis = gtk4::Label::new(None);
    hinweis.set_xalign(0.0);
    hinweis.set_wrap(true);
    hinweis.set_max_width_chars(70);
    aussen.append(&hinweis);

    let (vorgabe_kasten, vb) = rahmen("Neue Vorgabe verfügbar");
    vb.append(&grau(
        "Querencia hat eine neuere Vorgabe für rc.xml (z. B. neue Tastenkürzel). Ihre Datei wurde bei der Anmeldung nicht ersetzt, weil sie eigene Einstellungen enthält. „Neue Vorgabe übernehmen“ überträgt Ihre Einstellungen auf die neue Vorgabe; die bisherige Datei wird gesichert.",
    ));
    let vorgabe_knopf = gtk4::Button::with_label("Neue Vorgabe übernehmen");
    vorgabe_knopf.set_halign(gtk4::Align::Start);
    vb.append(&vorgabe_knopf);
    vorgabe_kasten.set_visible(false);
    aussen.append(&vorgabe_kasten);

    let status = gtk4::Label::new(None);
    status.set_xalign(0.0);
    status.set_wrap(true);
    status.set_max_width_chars(70);
    let standard_knopf = gtk4::Button::with_label("Standard wiederherstellen");
    let neu_laden = gtk4::Button::with_label("Neu laden");
    let uebernehmen = gtk4::Button::with_label("Übernehmen");
    uebernehmen.add_css_class("suggested-action");

    let seite_cell: Rc<RefCell<Option<Rc<Seite<B>>>>> = Rc::new(RefCell::new(None));
    let sc = seite_cell.clone();
    let geaendert: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(s) = sc.borrow().as_ref() {
            s.geaendert();
        }
    });
    let (bereich, inhalt) = B::bauen(geaendert);
    aussen.append(&inhalt);
    aussen.append(&grau(&format!("Quelle: {}", B::QUELLE)));

    let fuss = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    fuss.set_margin_start(14);
    fuss.set_margin_end(14);
    fuss.set_margin_bottom(12);
    fuss.append(&status);
    let knoepfe = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    knoepfe.append(&standard_knopf);
    knoepfe.append(&neu_laden);
    let platz = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    platz.set_hexpand(true);
    knoepfe.append(&platz);
    knoepfe.append(&uebernehmen);
    fuss.append(&knoepfe);

    let s = Rc::new(Seite {
        bereich,
        orte: Orte::standard(),
        hinweis,
        vorgabe_kasten,
        status,
        uebernehmen,
        geladen: RefCell::new(None),
        stand: RefCell::new(None),
        stumm: std::cell::Cell::new(false),
    });
    *seite_cell.borrow_mut() = Some(s.clone());
    s.laden();
    let s2 = s.clone();
    standard_knopf.connect_clicked(move |_| s2.standard());
    let s2 = s.clone();
    neu_laden.connect_clicked(move |_| {
        s2.laden();
        s2.status.set_text("Neu geladen.");
    });
    let s2 = s.clone();
    s.uebernehmen.connect_clicked(move |_| s2.uebernehmen());
    let s2 = s.clone();
    vorgabe_knopf.connect_clicked(move |_| s2.vorgabe());

    // Der Inhalt rollt für sich, die Knopfleiste bleibt immer sichtbar (das
    // Kontrollzentrum gibt der Seite die Höhe seines Rollbereichs).
    let rollbar = gtk4::ScrolledWindow::new();
    rollbar.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    rollbar.set_min_content_height(160);
    rollbar.set_vexpand(true);
    rollbar.set_child(Some(&aussen));
    let ganz = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    ganz.append(&rollbar);
    ganz.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    fuss.set_margin_top(8);
    ganz.append(&fuss);
    ganz.upcast()
}

impl<B: Bereich> Seite<B> {
    fn laden(&self) {
        let st = lk::lesen(&self.orte);
        let w = B::lesen(&st);
        self.stumm.set(true);
        self.bereich.zeigen(&w);
        self.stumm.set(false);
        let wirkung = lk::wirkung(&self.orte);
        let mut hinweis = lk::wirkung_text(&wirkung, &st);
        if let Err(e) = crate::labwc_xml::gueltig(&st.rc) {
            hinweis = format!(
                "{hinweis}\nAchtung: {e}. Angezeigt werden labwcs Standardwerte; „Übernehmen“ schreibt nichts, bis die Datei repariert ist."
            );
        }
        // erster Satz fett: wann es wirkt
        let (kopf, rest) = hinweis
            .split_once(". ")
            .map_or((hinweis.as_str(), ""), |(a, b)| (a, b));
        self.hinweis.set_markup(&format!(
            "<b>{}.</b> {}",
            glib::markup_escape_text(kopf.trim_end_matches('.')),
            glib::markup_escape_text(rest)
        ));
        self.vorgabe_kasten.set_visible(st.vorgabe_neu);
        eprintln!(
            "querencia-einstellungen: {} geladen ({:?}, {}): {w:?}",
            B::NAME,
            st.quelle,
            match &wirkung {
                Wirkung::Sofort { .. } => "sofort",
                Wirkung::NaechsteAnmeldung { .. } => "nächste Anmeldung",
            }
        );
        *self.geladen.borrow_mut() = Some(w);
        *self.stand.borrow_mut() = Some(st);
        self.uebernehmen.set_sensitive(false);
        self.status
            .set_text("Änderungen gelten erst nach „Übernehmen“.");
    }

    fn geaendert(&self) {
        if self.stumm.get() {
            return;
        }
        let anders = self.geladen.borrow().as_ref() != Some(&self.bereich.aktuell());
        self.uebernehmen.set_sensitive(anders);
        self.status.set_text(if anders {
            "Geändert – „Übernehmen“ speichert."
        } else {
            "Keine Änderung."
        });
    }

    fn standard(&self) {
        let Some(st) = self.stand.borrow().clone() else {
            return;
        };
        let w = standard::<B>(&st);
        self.stumm.set(true);
        self.bereich.zeigen(&w);
        self.stumm.set(false);
        self.geaendert();
    }

    fn uebernehmen(&self) {
        let Some(alt) = self.geladen.borrow().clone() else {
            return;
        };
        let neu = self.bereich.aktuell();
        let st = lk::lesen(&self.orte);
        // nicht unbemerkt überschreiben: nur die eigenen Werte zählen
        if B::lesen(&st) != alt {
            self.status.set_text("Die Einstellungen wurden inzwischen von anderer Stelle geändert – nicht überschrieben. „Neu laden“ zeigt den aktuellen Stand.");
            eprintln!(
                "querencia-einstellungen: {}: Konflikt, nicht geschrieben",
                B::NAME
            );
            return;
        }
        let zeit = jetzt_text("%Y%m%d-%H%M%S");
        let mut gesichert = Vec::new();
        let mut geschrieben = false;
        let rc_neu = match B::rc_aendern(&neu, &alt, &st.rc) {
            Ok(t) => t,
            Err(e) => return self.fehler(&e.to_string()),
        };
        if rc_neu != st.rc {
            match lk::rc_schreiben(&self.orte, &rc_neu, &st, &zeit) {
                Ok(s) => gesichert.extend(s),
                Err(e) => return self.fehler(&e),
            }
            geschrieben = true;
        }
        let env = B::env_aenderungen(&neu, &alt, &st);
        if !env.is_empty() {
            match lk::env_schreiben(&self.orte, &env, &zeit) {
                Ok(s) => gesichert.extend(s),
                Err(e) => return self.fehler(&e),
            }
            geschrieben = true;
        }
        let uhr = jetzt_text("%H:%M:%S");
        let mut text = format!("Übernommen um {uhr}.");
        if geschrieben {
            let w = lk::wirkung(&self.orte);
            match (&w, lk::neu_laden(&w)) {
                (Wirkung::Sofort { .. }, Ok(())) => {
                    text.push_str(" labwc hat die Konfiguration neu geladen.")
                }
                (Wirkung::Sofort { .. }, Err(e)) => {
                    text.push_str(&format!(" Neu laden fehlgeschlagen: {e}."))
                }
                _ => text.push_str(
                    " Gespeichert in ~/.config/querencia/labwc; wirksam ab der nächsten Anmeldung.",
                ),
            }
        }
        if !gesichert.is_empty() {
            let namen: Vec<String> = gesichert
                .iter()
                .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .collect();
            text.push_str(&format!(" Vorher gesichert als {}.", namen.join(", ")));
        }
        eprintln!(
            "querencia-einstellungen: {} übernommen: {neu:?} – {text}",
            B::NAME
        );
        self.laden();
        self.status.set_text(&text);
    }

    fn vorgabe(&self) {
        let zeit = jetzt_text("%Y%m%d-%H%M%S");
        match lk::vorgabe_uebernehmen(&self.orte, &zeit) {
            Ok(s) => {
                let w = lk::wirkung(&self.orte);
                let geladen = matches!(w, Wirkung::Sofort { .. }) && lk::neu_laden(&w).is_ok();
                self.laden();
                self.status.set_text(&format!(
                    "Neue Vorgabe übernommen, Ihre Einstellungen bleiben.{}{}",
                    if geladen {
                        " labwc hat neu geladen."
                    } else {
                        ""
                    },
                    s.and_then(|p| p
                        .file_name()
                        .map(|n| format!(" Vorher gesichert als {}.", n.to_string_lossy())))
                        .unwrap_or_default()
                ));
                eprintln!(
                    "querencia-einstellungen: {}: neue Vorgabe übernommen",
                    B::NAME
                );
            }
            Err(e) => self.fehler(&format!("Neue Vorgabe nicht übernommen: {e}")),
        }
    }

    fn fehler(&self, text: &str) {
        self.status.set_text(&format!("Nicht gespeichert: {text}"));
        eprintln!(
            "querencia-einstellungen: {}: nicht gespeichert: {text}",
            B::NAME
        );
    }
}
