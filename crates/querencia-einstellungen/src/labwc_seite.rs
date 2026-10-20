//! Gemeinsames Gerüst der Bereiche, die labwc einstellen (G2-AG): Tastatur,
//! Maus, Fensterdesign, Arbeitsflächen. Bedienmuster wie „Symbole der
//! Arbeitsfläche“ (G2-X): Ändern → „Übernehmen“ schreibt (atomar, mit
//! Sicherung; ein auf den Vorgabewert zurückgestellter Wert steht danach
//! wie in der Vorgabe), „Standard wiederherstellen“ zeigt die Vorgabe des
//! Runtime-Repos, „Neu laden“ liest die Dateien neu. Jeder Bereich ändert
//! nur seine eigenen Werte auf dem **aktuellen** Dateistand; was andere
//! Bereiche inzwischen geschrieben haben, bleibt stehen.
//! G2-AJ: Bereiche können zusätzlich Werte aus `org.gnome.desktop.interface`
//! führen (Erscheinungsbild, Schriften); die stehen in [`Stand::gs`], werden
//! beim „Übernehmen“ nach rc.xml und Umgebung geschrieben und wirken sofort.
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;

use crate::gsettings;
use crate::i18n::{gettext, gettext_f};
use crate::labwc_konfig::{self as lk, Orte, Stand, Wirkung};
use crate::labwc_xml::Fehler;

/// Ein Bereich: Werte lesen und schreiben, Oberfläche zeigen.
pub trait Bereich: Sized + 'static {
    type Werte: Clone + PartialEq + Debug;
    /// Name (msgid, mit `gettext_noop` markiert; angezeigt über `gettext`).
    const NAME: &'static str;
    /// Name im Protokoll (stderr), fest deutsch wie bis 0.0.9 und unabhängig
    /// von der Oberflächensprache: Prüfwerkzeuge suchen diese Zeilen.
    const PROTOKOLL: &'static str;
    /// Quelle der Werte (msgid wie `NAME`).
    const QUELLE: &'static str;
    /// `labwc --reconfigure` nach dem Schreiben, wenn labwc die Benutzerkopie
    /// liest. „Sprache und Region“ wirkt erst ab der Anmeldung und lässt es.
    const NEU_LADEN: bool = true;
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
    /// Schlüssel aus `org.gnome.desktop.interface`, die `lesen` in `st.gs` braucht.
    const GSETTINGS: &'static [&'static str] = &[];
    /// gsettings: die geänderten Werte (wirken sofort).
    fn gs_aenderungen(_neu: &Self::Werte, _alt: &Self::Werte) -> Vec<(&'static str, String)> {
        Vec::new()
    }
    /// Eigener Hinweis oben auf der Seite (erster Satz fett) statt des
    /// labwc-Wirkungstexts.
    fn hinweis(_w: &Wirkung, _st: &Stand) -> Option<String> {
        None
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
    v.gs = gsettings::standard(B::GSETTINGS);
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

/// Wie [`optionen`], aber in einem Raster mit `spalten` Spalten (G2-AJ:
/// lange Listen wie die Symbolthemen bleiben so auf einen Blick sichtbar).
pub fn optionen_raster(
    raster: &gtk4::Grid,
    ziel: &RefCell<Vec<(String, gtk4::CheckButton)>>,
    liste: &[(String, String)],
    aktuell: &str,
    geaendert: &Rc<dyn Fn()>,
    spalten: usize,
) {
    while let Some(c) = raster.first_child() {
        raster.remove(&c);
    }
    let anker = gtk4::CheckButton::new();
    anker.set_visible(false);
    let mut knoepfe: Vec<(String, gtk4::CheckButton)> = Vec::new();
    for (i, (schluessel, text)) in liste.iter().enumerate() {
        let k = gtk4::CheckButton::with_label(text);
        k.set_group(Some(&anker));
        k.set_active(schluessel == aktuell);
        k.set_hexpand(true);
        let g = geaendert.clone();
        k.connect_toggled(move |k| {
            if k.is_active() {
                g();
            }
        });
        raster.attach(&k, (i % spalten) as i32, (i / spalten) as i32, 1, 1);
        knoepfe.push((schluessel.clone(), k));
    }
    if !knoepfe.iter().any(|(_, k)| k.is_active()) {
        anker.set_active(true);
    }
    // der Anker gehört zur Gruppe, braucht aber keinen Platz im Raster
    raster.attach(&anker, 0, liste.len().div_ceil(spalten) as i32, 1, 1);
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

/// Was „Übernehmen“ in die labwc-Benutzerkopie schreibt (ohne Oberfläche
/// und gsettings): rc.xml mit den geänderten Werten, zurückgestellte Werte
/// wie in der Vorgabe (0.1.1, Werkstatt QREL-06 A17.6), und die eigene
/// Umgebungsdatei. Rückgabe: ob geschrieben wurde, und die Sicherungen.
pub fn dateien_schreiben<B: Bereich>(
    orte: &Orte,
    st: &Stand,
    neu: &B::Werte,
    alt: &B::Werte,
    zeit: &str,
) -> Result<(bool, Vec<std::path::PathBuf>), String> {
    let mut gesichert = Vec::new();
    let mut geschrieben = false;
    let rc_neu = B::rc_aendern(neu, alt, &st.rc)
        .and_then(|t| crate::labwc_xml::angleichen(&t, &st.rc, &st.rc_basis))
        .map_err(|e| e.to_string())?;
    if rc_neu != st.rc {
        gesichert.extend(lk::rc_schreiben(orte, &rc_neu, st, zeit)?);
        geschrieben = true;
    }
    let env = B::env_aenderungen(neu, alt, st);
    if !env.is_empty() {
        gesichert.extend(lk::env_schreiben(orte, &env, zeit)?);
        geschrieben = true;
    }
    Ok((geschrieben, gesichert))
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
        glib::markup_escape_text(&gettext(B::NAME))
    ));
    ueberschrift.set_xalign(0.0);
    aussen.append(&ueberschrift);

    let hinweis = gtk4::Label::new(None);
    hinweis.set_xalign(0.0);
    hinweis.set_wrap(true);
    hinweis.set_max_width_chars(70);
    aussen.append(&hinweis);

    let (vorgabe_kasten, vb) = rahmen(&gettext("New Default Available"));
    vb.append(&grau(&gettext(
        "Querencia has a newer default for rc.xml (e.g. new keyboard shortcuts). Your file was not replaced at login because it contains your own settings. “Adopt New Default” transfers your settings to the new default; the previous file is backed up.",
    )));
    let vorgabe_knopf = gtk4::Button::with_label(&gettext("Adopt New Default"));
    vorgabe_knopf.set_halign(gtk4::Align::Start);
    vb.append(&vorgabe_knopf);
    vorgabe_kasten.set_visible(false);
    aussen.append(&vorgabe_kasten);

    let status = gtk4::Label::new(None);
    status.set_xalign(0.0);
    status.set_wrap(true);
    status.set_max_width_chars(70);
    let standard_knopf = gtk4::Button::with_label(&gettext("Restore Defaults"));
    let neu_laden = gtk4::Button::with_label(&gettext("Reload"));
    let uebernehmen = gtk4::Button::with_label(&gettext("Apply"));
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
    aussen.append(&grau(&gettext_f(
        "Source: {source}",
        &[("source", &gettext(B::QUELLE))],
    )));

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
        s2.status.set_text(&gettext("Reloaded."));
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
    fn stand_lesen(&self) -> Stand {
        let mut st = lk::lesen(&self.orte);
        st.gs = gsettings::lesen(B::GSETTINGS);
        st
    }

    fn laden(&self) {
        let st = self.stand_lesen();
        let w = B::lesen(&st);
        self.stumm.set(true);
        self.bereich.zeigen(&w);
        self.stumm.set(false);
        let wirkung = lk::wirkung(&self.orte);
        let mut hinweis =
            B::hinweis(&wirkung, &st).unwrap_or_else(|| lk::wirkung_text(&wirkung, &st));
        if let Some(f) = &st.fehler {
            hinweis = format!(
                "{hinweis}\n{}",
                gettext_f(
                    "Attention: {error}. labwc’s default values are shown; “Apply” writes nothing until the file is readable.",
                    &[("error", f)],
                )
            );
        } else if let Err(e) = crate::labwc_xml::gueltig(&st.rc) {
            hinweis = format!(
                "{hinweis}\n{}",
                gettext_f(
                    "Attention: {error}. labwc’s default values are shown; “Apply” does not write rc.xml until the file is repaired (layout and pointer are saved anyway).",
                    &[("error", &e.to_string())],
                )
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
            B::PROTOKOLL,
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
            .set_text(&gettext("Changes take effect only after “Apply”."));
    }

    fn geaendert(&self) {
        if self.stumm.get() {
            return;
        }
        let anders = self.geladen.borrow().as_ref() != Some(&self.bereich.aktuell());
        self.uebernehmen.set_sensitive(anders);
        self.status.set_text(&if anders {
            gettext("Changed – “Apply” saves.")
        } else {
            gettext("No change.")
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
        let st = self.stand_lesen();
        if let Some(f) = &st.fehler {
            return self.fehler(f);
        }
        // nicht unbemerkt überschreiben: nur die eigenen Werte zählen
        if B::lesen(&st) != alt {
            self.status.set_text(&gettext("The settings have been changed elsewhere in the meantime – not overwritten. “Reload” shows the current state."));
            eprintln!(
                "querencia-einstellungen: {}: Konflikt, nicht geschrieben",
                B::PROTOKOLL
            );
            return;
        }
        let zeit = jetzt_text("%Y%m%d-%H%M%S");
        let (geschrieben, gesichert) =
            match dateien_schreiben::<B>(&self.orte, &st, &neu, &alt, &zeit) {
                Ok(e) => e,
                Err(e) => return self.fehler(&e),
            };
        let gs = B::gs_aenderungen(&neu, &alt);
        if let Err(e) = gsettings::schreiben(&gs) {
            return self.fehler(&if geschrieben {
                gettext_f(
                    "{error} (labwc configuration is already saved)",
                    &[("error", &e)],
                )
            } else {
                e
            });
        }
        let uhr = jetzt_text("%H:%M:%S");
        let mut text = gettext_f("Applied at {time}.", &[("time", &uhr)]);
        if !gs.is_empty() {
            text.push(' ');
            text.push_str(&gettext(
                "GTK programs, panel and desktop pick it up immediately.",
            ));
        }
        if geschrieben {
            let w = lk::wirkung(&self.orte);
            text.push(' ');
            match (&w, B::NEU_LADEN.then(|| lk::neu_laden(&w))) {
                (Wirkung::Sofort { .. }, Some(Ok(()))) => {
                    text.push_str(&gettext("labwc has reloaded its configuration."))
                }
                (Wirkung::Sofort { .. }, Some(Err(e))) => {
                    text.push_str(&gettext_f("Reloading failed: {error}.", &[("error", &e)]))
                }
                _ => text.push_str(&gettext(
                    "Saved in ~/.config/querencia/labwc; effective from the next login.",
                )),
            }
        }
        if !gesichert.is_empty() {
            let namen: Vec<String> = gesichert
                .iter()
                .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
                .collect();
            text.push(' ');
            text.push_str(&gettext_f(
                "Previously saved as {names}.",
                &[("names", &namen.join(", "))],
            ));
        }
        eprintln!(
            "querencia-einstellungen: {} übernommen: {neu:?} (gsettings {gs:?}) – {text}",
            B::PROTOKOLL
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
                let mut text = gettext("New default adopted, your settings remain.");
                if geladen {
                    text.push(' ');
                    text.push_str(&gettext("labwc has reloaded."));
                }
                if let Some(n) = s.as_ref().and_then(|p| p.file_name()) {
                    text.push(' ');
                    text.push_str(&gettext_f(
                        "Previously saved as {names}.",
                        &[("names", &n.to_string_lossy())],
                    ));
                }
                self.status.set_text(&text);
                eprintln!(
                    "querencia-einstellungen: {}: neue Vorgabe übernommen",
                    B::PROTOKOLL
                );
            }
            Err(e) => self.fehler(&gettext_f(
                "New default not adopted: {error}",
                &[("error", &e)],
            )),
        }
    }

    fn fehler(&self, text: &str) {
        self.status
            .set_text(&gettext_f("Not saved: {error}", &[("error", text)]));
        eprintln!(
            "querencia-einstellungen: {}: nicht gespeichert: {text}",
            B::PROTOKOLL
        );
    }
}

#[cfg(test)]
mod tests {
    //! Werkstatt QREL-06 A17.6: Ändern und Zurückstellen in Tastatur, Maus,
    //! Fensterdesign und Arbeitsflächen über denselben Schreibweg wie
    //! „Übernehmen“ ([`dateien_schreiben`]).
    use super::*;
    use crate::arbeitsflaechen::Arbeitsflaechen;
    use crate::fensterdesign::Fensterdesign;
    use crate::labwc_xml as lx;
    use crate::maus::Maus;
    use crate::tastatur::Tastatur;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    /// Runtime `main` 548483c, `session/querencia/labwc/rc.xml` (live seit 20.10.)
    const VORGABE: &str = include_str!("../testdaten/rc-vorgabe-548483c.xml");
    /// feste Kopie der GUI-Läufe (`szenarien/labwc/vorgabe`, Runtime bb90803)
    const VORGABE_BB90803: &str = include_str!("../../../szenarien/labwc/vorgabe/rc.xml");

    fn ordner(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("qe-a176-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    /// Vorgabe im „Repo“; mit `kopie` wie nach der Anmeldung: Benutzerkopie
    /// und `.vorgabe/` vom Sitzungsstarter angelegt.
    fn orte(w: &Path, vorgabe: &str, kopie: bool) -> Orte {
        let o = Orte {
            benutzer: w.join("config/querencia/labwc"),
            vorgabe: w.join("repo/labwc"),
        };
        std::fs::create_dir_all(&o.vorgabe).unwrap();
        std::fs::write(o.vorgabe.join("rc.xml"), vorgabe).unwrap();
        std::fs::write(o.vorgabe.join("environment"), "XKB_DEFAULT_LAYOUT=de\n").unwrap();
        if kopie {
            std::fs::create_dir_all(o.benutzer.join(".vorgabe")).unwrap();
            for d in [o.rc(), o.vorgabe_basis()] {
                std::fs::write(d, vorgabe).unwrap();
            }
            std::fs::write(o.benutzer.join("environment"), "XKB_DEFAULT_LAYOUT=de\n").unwrap();
            std::fs::write(o.benutzer.join("rc.xml.vorher-20261020-112812"), "Starter").unwrap();
        }
        o
    }

    /// Alle Dateien der Benutzerkopie mit Inhalt.
    fn dateien(o: &Orte) -> BTreeMap<String, Vec<u8>> {
        fn sammeln(d: &Path, wurzel: &Path, aus: &mut BTreeMap<String, Vec<u8>>) {
            for e in std::fs::read_dir(d).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    sammeln(&p, wurzel, aus);
                } else {
                    let n = p.strip_prefix(wurzel).unwrap().display().to_string();
                    aus.insert(n, std::fs::read(&p).unwrap());
                }
            }
        }
        let mut m = BTreeMap::new();
        sammeln(&o.benutzer, &o.benutzer, &mut m);
        m
    }

    fn uebernehmen<B: Bereich>(o: &Orte, zeit: &str, aendern: impl Fn(&mut B::Werte)) {
        let st = lk::lesen(o);
        let alt = B::lesen(&st);
        let mut neu = alt.clone();
        aendern(&mut neu);
        assert_ne!(neu, alt, "{}: keine Änderung", B::PROTOKOLL);
        dateien_schreiben::<B>(o, &st, &neu, &alt, zeit).unwrap();
        assert_eq!(B::lesen(&lk::lesen(o)), neu, "{}", B::PROTOKOLL);
    }

    /// Der Ablauf der Werkstatt, 20:19–20:20: je Bereich ändern, übernehmen,
    /// zurückstellen, übernehmen. `neuer_lauf`: das Programm zwischen den
    /// Schritten schließen und wieder öffnen.
    fn werkstatt_ablauf(o: &Orte, w: &Path, neuer_lauf: bool) {
        let mut n = 0;
        let mut zeit = || {
            if neuer_lauf {
                lk::lauf_vergessen(w);
            }
            n += 1;
            format!("20261020-2019{n:02}")
        };
        uebernehmen::<Tastatur>(o, &zeit(), |v| v.verzoegerung = 610);
        uebernehmen::<Tastatur>(o, &zeit(), |v| v.verzoegerung = 600);
        uebernehmen::<Maus>(o, &zeit(), |v| v.beschleunigung = false);
        uebernehmen::<Maus>(o, &zeit(), |v| v.beschleunigung = true);
        uebernehmen::<Fensterdesign>(o, &zeit(), |v| v.eckenradius = 9);
        uebernehmen::<Fensterdesign>(o, &zeit(), |v| v.eckenradius = 8);
        uebernehmen::<Arbeitsflaechen>(o, &zeit(), |v| v.anzeige = !v.anzeige);
        uebernehmen::<Arbeitsflaechen>(o, &zeit(), |v| v.anzeige = !v.anzeige);
    }

    fn eigene_sicherungen(o: &Orte) -> Vec<String> {
        dateien(o)
            .into_keys()
            .filter(|n| n.contains(lk::SICHERUNG))
            .collect()
    }

    #[test]
    fn a176_ein_lauf_ohne_jeden_rest() {
        for (i, vorgabe) in [VORGABE, VORGABE_BB90803].into_iter().enumerate() {
            let w = ordner(&format!("lauf{i}"));
            let o = orte(&w, vorgabe, true);
            let vorher = dateien(&o);
            werkstatt_ablauf(&o, &w, false);
            // rc.xml byte-gleich mit der Basis: für den Starter unverändert
            assert_eq!(std::fs::read_to_string(o.rc()).unwrap(), vorgabe);
            assert_eq!(std::fs::read_to_string(o.vorgabe_basis()).unwrap(), vorgabe);
            // keine Sicherungen, keine Umgebungsdatei, nichts sonst
            assert_eq!(dateien(&o), vorher);
            std::fs::remove_dir_all(&w).unwrap();
        }
    }

    #[test]
    fn a176_neue_laeufe_hoechstens_fuenf_sicherungen() {
        let w = ordner("laeufe");
        let o = orte(&w, VORGABE, true);
        werkstatt_ablauf(&o, &w, true);
        assert_eq!(std::fs::read_to_string(o.rc()).unwrap(), VORGABE);
        let s = eigene_sicherungen(&o);
        assert!(s.len() <= lk::SICHERUNGEN, "{s:?}");
        // keine Sicherung gleicht der jetzigen Datei, die des Starters bleibt
        for n in &s {
            assert_ne!(
                std::fs::read_to_string(o.benutzer.join(n)).unwrap(),
                VORGABE
            );
        }
        assert_eq!(
            std::fs::read_to_string(o.benutzer.join("rc.xml.vorher-20261020-112812")).unwrap(),
            "Starter"
        );
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn a176_ohne_benutzerkopie() {
        let w = ordner("ohne");
        let o = orte(&w, VORGABE, false);
        werkstatt_ablauf(&o, &w, false);
        // das erste Übernehmen legt Kopie und .vorgabe an; zurückgestellt gleicht sie der Vorgabe
        assert_eq!(std::fs::read_to_string(o.rc()).unwrap(), VORGABE);
        assert_eq!(std::fs::read_to_string(o.vorgabe_basis()).unwrap(), VORGABE);
        assert!(eigene_sicherungen(&o).is_empty());
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn eigene_werte_bleiben_zurueckgestelltes_geht() {
        let w = ordner("eigen");
        let o = orte(&w, VORGABE, true);
        uebernehmen::<Tastatur>(&o, "1", |v| v.rate = 40);
        uebernehmen::<Fensterdesign>(&o, "2", |v| v.schatten = true);
        let mit_eigenen = std::fs::read_to_string(o.rc()).unwrap();
        werkstatt_ablauf(&o, &w, false);
        assert_eq!(std::fs::read_to_string(o.rc()).unwrap(), mit_eigenen);
        // „Standard wiederherstellen“ + Übernehmen: wieder die Vorgabe
        for _ in 0..2 {
            let st = lk::lesen(&o);
            let alt = Tastatur::lesen(&st);
            dateien_schreiben::<Tastatur>(&o, &st, &standard::<Tastatur>(&st), &alt, "3").unwrap();
            let st = lk::lesen(&o);
            let alt = Fensterdesign::lesen(&st);
            let neu = standard::<Fensterdesign>(&st);
            if neu != alt {
                dateien_schreiben::<Fensterdesign>(&o, &st, &neu, &alt, "4").unwrap();
            }
        }
        assert_eq!(std::fs::read_to_string(o.rc()).unwrap(), VORGABE);
        assert!(eigene_sicherungen(&o).is_empty());
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn belegung_zurueckgestellt_ohne_umgebungsdatei() {
        let w = ordner("belegung");
        let o = orte(&w, VORGABE, true);
        let vorher = dateien(&o);
        uebernehmen::<Tastatur>(&o, "1", |v| {
            v.belegungen = vec![("us".into(), String::new())]
        });
        assert!(o.env_datei().is_file());
        uebernehmen::<Tastatur>(&o, "2", |v| {
            v.belegungen = vec![("de".into(), String::new())]
        });
        assert_eq!(dateien(&o), vorher);
        assert!(!o.benutzer.join("environment.d").exists());
        std::fs::remove_dir_all(&w).unwrap();
    }

    /// Status-Session 21.10.: fremde Sicherungen in der Benutzerkopie (Rückfallpfad
    /// aus 00, Starter-Abgleich 21:35, autostart/environment des Starters) bleiben
    /// nach 20 Übernahmen in 20 Programmläufen byte-gleich; eigene höchstens 5.
    #[test]
    fn fremde_vorher_dateien_bleiben_nach_20_uebernahmen() {
        let w = ordner("fremd20");
        let o = orte(&w, VORGABE, true);
        let fremde = [
            "rc.xml.vorher-20261020-112812",
            "rc.xml.vorher-20261020-213520",
            "autostart.vorher-20261020-141708",
            "environment.vorher-20261020-141708",
            "environment.d/zz-querencia-einstellungen.env.vorher-20261020-120000",
        ];
        std::fs::create_dir_all(o.benutzer.join("environment.d")).unwrap();
        for (i, f) in fremde.iter().enumerate() {
            std::fs::write(o.benutzer.join(f), format!("fremd {i}\n")).unwrap();
        }
        for i in 0..20u32 {
            lk::lauf_vergessen(&w);
            let zeit = format!("20261021-01{i:02}00");
            if i % 2 == 0 {
                uebernehmen::<Tastatur>(&o, &zeit, |v| v.verzoegerung = 610 + 10 * i);
                uebernehmen::<Tastatur>(&o, &zeit, |v| {
                    v.belegungen =
                        vec![(if i % 4 == 0 { "us" } else { "ch" }.into(), String::new())]
                });
            } else {
                uebernehmen::<Fensterdesign>(&o, &zeit, |v| v.eckenradius = 1 + i % 7);
            }
        }
        for (i, f) in fremde.iter().enumerate() {
            assert_eq!(
                std::fs::read_to_string(o.benutzer.join(f)).unwrap(),
                format!("fremd {i}\n"),
                "{f}"
            );
        }
        let s = eigene_sicherungen(&o);
        let rc = s.iter().filter(|n| n.starts_with("rc.xml.")).count();
        let env = s.iter().filter(|n| n.starts_with("environment.d/")).count();
        assert!(rc <= lk::SICHERUNGEN && env <= lk::SICHERUNGEN, "{s:?}");
        assert!(rc > 0 && env > 0, "{s:?}");
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn labwc_standardwerte_wie_die_bereiche() {
        // ausdrücklich eingetragene labwc-Standardwerte lesen die Bereiche
        // genau wie fehlende: angleichen darf sie also weglassen
        let leer = lk::lesen(&Orte {
            benutzer: PathBuf::from("/nicht/da"),
            vorgabe: PathBuf::from("/nicht/da"),
        });
        let mut voll = leer.clone();
        voll.rc = lx::LEER.to_owned();
        for (pfad, k, v) in lx::LABWC_STANDARD {
            voll.rc = lx::setzen(&voll.rc, pfad, k, v).unwrap();
        }
        assert_eq!(Tastatur::lesen(&leer), Tastatur::lesen(&voll));
        assert_eq!(Maus::lesen(&leer), Maus::lesen(&voll));
        assert_eq!(Fensterdesign::lesen(&leer), Fensterdesign::lesen(&voll));
        assert_eq!(Arbeitsflaechen::lesen(&leer), Arbeitsflaechen::lesen(&voll));
        assert_eq!(
            crate::schriften::ist_fett(lx::wert(&voll.rc, lx::SCHRIFT_AKTIV, "weight")),
            crate::schriften::ist_fett(None)
        );
    }
}
