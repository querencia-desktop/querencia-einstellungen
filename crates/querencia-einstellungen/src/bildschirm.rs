//! Bereich „Bildschirm“ (G2-G) im Stil der GNOME-2-Monitoreinstellungen:
//! oben die Anordnung mit verschiebbaren Rechtecken, darunter je Monitor
//! Ein/Aus, Auflösung, Wiederholrate, Drehung, Skalierung, Hauptmonitor;
//! Anwenden (erst `test`, dann `apply`), Zurücksetzen, Schließen. Nach dem
//! Anwenden fragt ein Dialog mit 15-s-Countdown; ohne Bestätigung wird die
//! vorherige Konfiguration wiederhergestellt. Bestätigt wird nach
//! `~/.config/querencia/bildschirm.ini` gespeichert (atomar); beim
//! Sitzungsstart setzt `--bildschirm-anwenden` sie wieder.
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, glib};

use crate::ausgaben::Verbindung;
use crate::bildschirm_modell::{
    self as bm, Ausgabe, Drehung, FREIE_GROESSEN, Modus, SKALIERUNGEN, Wahl, skalierung_text,
};
use crate::i18n::{gettext, gettext_f, ngettext_f, pgettext};

/// Fenstertitel des eigenen Fensters (`--bildschirm`).
pub fn titel() -> String {
    gettext("Querencia – Screen")
}
const FANG: i32 = 60;
pub const COUNTDOWN_SEKUNDEN: u32 = 15;

struct Zustand {
    ausgaben: Vec<Ausgabe>,
    ist: Vec<Wahl>,
    wahlen: Vec<Wahl>,
    gewaehlt: usize,
    verbindung: Option<Verbindung>,
    /// Maßstab und Versatz der Anordnung, während gezogen wird eingefroren.
    massstab: (f64, f64, f64),
    ziehe: Option<(usize, (i32, i32))>,
}

struct Oberflaeche {
    zustand: RefCell<Zustand>,
    stumm: Cell<bool>,
    anordnung: gtk4::DrawingArea,
    monitor: gtk4::DropDown,
    aktiv: gtk4::CheckButton,
    aufloesung: gtk4::DropDown,
    rate: gtk4::DropDown,
    drehung: gtk4::DropDown,
    skalierung: gtk4::DropDown,
    skalierung_hinweis: gtk4::Label,
    haupt: gtk4::Button,
    details: gtk4::Label,
    status: gtk4::Label,
    anwenden: gtk4::Button,
    zuruecksetzen: gtk4::Button,
    /// Größen (Pixel) hinter der Auflösungsliste, Raten hinter der Ratenliste.
    groessen: RefCell<Vec<(i32, i32)>>,
    raten: RefCell<Vec<i32>>,
    skalierungen: RefCell<Vec<f64>>,
    /// Zuletzt gesetzte Beschriftungen je Liste: ein Modell wird nur ersetzt,
    /// wenn sich die Einträge ändern. Ein Ersetzen aus dem eigenen
    /// Auswahl-Handler heraus (während die Liste zuklappt) lässt GTK 4.22
    /// unter labwc hängen (Liste bleibt offen, keine Eingabe mehr).
    listen: RefCell<[Vec<String>; 4]>,
}

/// Setzt das Modell von `d` neu, wenn `neu` von der gespeicherten Liste
/// `index` abweicht; wählt danach `auswahl`.
fn liste_setzen(
    listen: &RefCell<[Vec<String>; 4]>,
    index: usize,
    d: &gtk4::DropDown,
    neu: &[String],
    auswahl: usize,
) {
    if listen.borrow()[index] != neu {
        let refs: Vec<&str> = neu.iter().map(String::as_str).collect();
        d.set_model(Some(&gtk4::StringList::new(&refs)));
        listen.borrow_mut()[index] = neu.to_vec();
    }
    if d.selected() != auswahl as u32 {
        d.set_selected(auswahl as u32);
    }
}

/// Eigenes Fenster (`--bildschirm`).
pub fn oeffnen(app: &gtk4::Application) -> gtk4::Window {
    // Höhe an den Bildschirm anpassen (1280×720: Panels und Titelleiste
    // abziehen); der Inhalt rollt, falls er trotzdem nicht passt.
    let hoehe = crate::hintergrund::bildschirmhoehe().map_or(660, |h| (h - 100).clamp(480, 660));
    let fenster = gtk4::Window::builder()
        .application(app)
        .title(titel())
        .default_width(760)
        .default_height(hoehe)
        .build();
    // Inhalt rollt, Statuszeile und Schaltflächen bleiben unten sichtbar.
    let (inhalt, fuss) = seite_teile(true);
    let rollbar = gtk4::ScrolledWindow::new();
    rollbar.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    rollbar.set_child(Some(&inhalt));
    rollbar.set_vexpand(true);
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    aussen.append(&rollbar);
    fuss.set_margin_start(14);
    fuss.set_margin_end(14);
    fuss.set_margin_bottom(12);
    aussen.append(&fuss);
    fenster.set_child(Some(&aussen));
    fenster
}

/// Die Seite für das Kontrollzentrum (ohne „Schließen“) oder das eigene
/// Fenster (mit).
pub fn seite(mit_schliessen: bool) -> gtk4::Widget {
    let (inhalt, fuss) = seite_teile(mit_schliessen);
    fuss.set_margin_start(14);
    fuss.set_margin_end(14);
    fuss.set_margin_bottom(12);
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    aussen.append(&inhalt);
    aussen.append(&fuss);
    aussen.upcast()
}

/// Inhalt (Überschrift, Monitor, Anordnung) und Fuß (Statuszeile,
/// Schaltflächen) getrennt, damit das eigene Fenster den Fuß fest unten hält.
fn seite_teile(mit_schliessen: bool) -> (gtk4::Box, gtk4::Box) {
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    aussen.set_margin_top(12);
    aussen.set_margin_bottom(12);
    aussen.set_margin_start(14);
    aussen.set_margin_end(14);
    let ueberschrift = gtk4::Label::new(None);
    ueberschrift.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(&gettext("Screen"))
    ));
    ueberschrift.set_xalign(0.0);
    aussen.append(&ueberschrift);

    // --- Anordnung
    let anordnung_rahmen = gtk4::Frame::new(Some(&gettext("Monitor Arrangement (drag to move)")));
    let anordnung = gtk4::DrawingArea::new();
    anordnung.set_size_request(400, 130);
    anordnung.set_hexpand(true);
    anordnung.set_vexpand(true);
    anordnung.set_margin_top(6);
    anordnung.set_margin_bottom(6);
    anordnung.set_margin_start(6);
    anordnung.set_margin_end(6);
    anordnung_rahmen.set_child(Some(&anordnung));

    // --- Einstellungen je Monitor (oben, damit die Aufklapplisten nach
    // unten Platz haben: nach oben geklappte Listen bekommen unter labwc
    // 0.9.8 keine Klicks, siehe Bericht G2-G)
    let rahmen = gtk4::Frame::new(Some(&gettext("Monitor")));
    let gitter = gtk4::Grid::new();
    gitter.set_row_spacing(4);
    gitter.set_column_spacing(10);
    gitter.set_margin_top(6);
    gitter.set_margin_bottom(6);
    gitter.set_margin_start(8);
    gitter.set_margin_end(8);
    let beschriftung = |t: String| {
        let l = gtk4::Label::new(Some(&t));
        l.set_xalign(1.0);
        l
    };
    let monitor = gtk4::DropDown::from_strings(&[]);
    monitor.set_hexpand(true);
    let aktiv = gtk4::CheckButton::with_label(&gettext("On"));
    let aufloesung = gtk4::DropDown::from_strings(&[]);
    let rate = gtk4::DropDown::from_strings(&[]);
    let drehung = gtk4::DropDown::from_strings(
        &Drehung::ALLE
            .iter()
            .map(|d| d.beschriftung())
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    let skalierung = gtk4::DropDown::from_strings(&[]);
    let skalierung_hinweis = gtk4::Label::new(None);
    skalierung_hinweis.set_xalign(0.0);
    skalierung_hinweis.set_wrap(true);
    skalierung_hinweis.set_max_width_chars(50);
    skalierung_hinweis.add_css_class("dim-label");
    let haupt = gtk4::Button::with_label(&gettext("Use as Primary Monitor"));
    let details = gtk4::Label::new(None);
    details.set_xalign(0.0);
    details.set_wrap(true);
    details.add_css_class("dim-label");
    gitter.attach(&beschriftung(gettext("Monitor:")), 0, 0, 1, 1);
    gitter.attach(&monitor, 1, 0, 2, 1);
    gitter.attach(&aktiv, 1, 1, 1, 1);
    gitter.attach(&haupt, 2, 1, 1, 1);
    gitter.attach(&beschriftung(gettext("Resolution:")), 0, 2, 1, 1);
    gitter.attach(&aufloesung, 1, 2, 1, 1);
    gitter.attach(&beschriftung(gettext("Refresh rate:")), 0, 3, 1, 1);
    gitter.attach(&rate, 1, 3, 1, 1);
    gitter.attach(&beschriftung(gettext("Rotation:")), 0, 4, 1, 1);
    gitter.attach(&drehung, 1, 4, 1, 1);
    gitter.attach(&beschriftung(gettext("Scale:")), 0, 5, 1, 1);
    gitter.attach(&skalierung, 1, 5, 1, 1);
    gitter.attach(&skalierung_hinweis, 1, 6, 2, 1);
    // Detailzeile außerhalb des Gitters: eine umbrechende, breite Zeile würde
    // sonst die Spaltenbreiten (und damit Klickpositionen) verschieben.
    let rahmen_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    rahmen_box.append(&gitter);
    details.set_margin_start(8);
    details.set_margin_end(8);
    details.set_margin_bottom(6);
    details.set_max_width_chars(60);
    rahmen_box.append(&details);
    rahmen.set_child(Some(&rahmen_box));
    aussen.append(&rahmen);
    aussen.append(&anordnung_rahmen);

    // --- Statuszeile und Schaltflächen (Fuß)
    let fuss = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    let status = gtk4::Label::new(None);
    status.set_xalign(0.0);
    status.set_wrap(true);
    status.set_max_width_chars(70);
    fuss.append(&status);
    let knoepfe = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let zuruecksetzen = gtk4::Button::with_label(&gettext("Reset"));
    let anwenden = gtk4::Button::with_label(&pgettext("screen settings", "Apply"));
    anwenden.add_css_class("suggested-action");
    knoepfe.append(&zuruecksetzen);
    let platz = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    platz.set_hexpand(true);
    knoepfe.append(&platz);
    if mit_schliessen {
        let schliessen = gtk4::Button::with_label(&gettext("Close"));
        schliessen.connect_clicked(|b| {
            if let Some(w) = b.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                w.close();
            }
        });
        knoepfe.append(&schliessen);
    }
    knoepfe.append(&anwenden);
    fuss.append(&knoepfe);

    let (verbindung, ausgaben, fehler) = match Verbindung::oeffnen() {
        Ok(mut v) => match v.lesen() {
            Ok(a) => (Some(v), a, None),
            Err(e) => (None, Vec::new(), Some(e)),
        },
        Err(e) => (None, Vec::new(), Some(e)),
    };
    let ist = bm::wahlen_aus(&ausgaben);
    let o = Rc::new(Oberflaeche {
        zustand: RefCell::new(Zustand {
            wahlen: ist.clone(),
            ist,
            ausgaben,
            gewaehlt: 0,
            verbindung,
            massstab: (0.1, 0.0, 0.0),
            ziehe: None,
        }),
        stumm: Cell::new(false),
        anordnung,
        monitor,
        aktiv,
        aufloesung,
        rate,
        drehung,
        skalierung,
        skalierung_hinweis,
        haupt,
        details,
        status,
        anwenden,
        zuruecksetzen,
        groessen: RefCell::new(Vec::new()),
        raten: RefCell::new(Vec::new()),
        skalierungen: RefCell::new(Vec::new()),
        listen: RefCell::new([vec![], vec![], vec![], vec![]]),
    });
    verdrahten(&o);
    if let Some(e) = fehler {
        o.status.set_text(&gettext_f(
            "The screen cannot be managed: {error}",
            &[("error", &e.to_string())],
        ));
        for w in [
            &o.monitor,
            &o.aufloesung,
            &o.rate,
            &o.drehung,
            &o.skalierung,
        ] {
            w.set_sensitive(false);
        }
        o.aktiv.set_sensitive(false);
        o.haupt.set_sensitive(false);
        o.anwenden.set_sensitive(false);
        o.zuruecksetzen.set_sensitive(false);
    } else {
        o.monitorliste_fuellen();
        o.auswahl_zeigen();
        let n = o.zustand.borrow().ausgaben.len();
        o.status.set_text(&ngettext_f(
            "{n} output reported (wlr-output-management, labwc). Changes take effect only after “Apply”.",
            "{n} outputs reported (wlr-output-management, labwc). Changes take effect only after “Apply”.",
            n as u32,
            &[],
        ));
    }
    (aussen, fuss)
}

impl Oberflaeche {
    fn monitorliste_fuellen(&self) {
        let z = self.zustand.borrow();
        let namen: Vec<String> = z
            .ausgaben
            .iter()
            .zip(&z.wahlen)
            .map(|(a, w)| {
                let mut n = a.anzeigename();
                if w.haupt {
                    n = gettext_f("{name} – primary monitor", &[("name", &n)]);
                }
                if !w.aktiv {
                    n = gettext_f("{name} (off)", &[("name", &n)]);
                }
                n
            })
            .collect();
        let gewaehlt = z.gewaehlt;
        drop(z);
        self.stumm.set(true);
        liste_setzen(&self.listen, 0, &self.monitor, &namen, gewaehlt);
        self.stumm.set(false);
    }

    /// Füllt die Felder für die gewählte Ausgabe (ohne Rückwirkung).
    fn auswahl_zeigen(&self) {
        let z = self.zustand.borrow();
        let Some((a, w)) = z.ausgaben.get(z.gewaehlt).zip(z.wahlen.get(z.gewaehlt)) else {
            drop(z);
            self.stumm.set(true);
            self.aktiv.set_active(false);
            self.stumm.set(false);
            self.anwenden.set_sensitive(false);
            return;
        };
        // Größen: aus den Modi (absteigend, ohne Doppelte), sonst freie Größen
        let mut groessen: Vec<(i32, i32)> = Vec::new();
        let mut bevorzugt: Option<(i32, i32)> = None;
        for m in &a.modi {
            if !groessen.contains(&(m.breite, m.hoehe)) {
                groessen.push((m.breite, m.hoehe));
            }
            if m.bevorzugt && bevorzugt.is_none() {
                bevorzugt = Some((m.breite, m.hoehe));
            }
        }
        // ohne bevorzugten Modus (virtuelle/headless Ausgaben melden nur den
        // aktuellen) stehen die freien Größen zur Wahl
        let frei = !a.modi.iter().any(|m| m.bevorzugt);
        if frei {
            for g in FREIE_GROESSEN {
                if !groessen.contains(g) {
                    groessen.push(*g);
                }
            }
        }
        if !groessen.contains(&(w.modus.breite, w.modus.hoehe)) {
            groessen.push((w.modus.breite, w.modus.hoehe));
        }
        groessen.sort_by(|x, y| (y.0 * y.1, y.0).cmp(&(x.0 * x.1, x.0)));
        let groessen_text: Vec<String> = groessen
            .iter()
            .map(|(b, h)| {
                let mut t = format!("{b} × {h}");
                if Some((*b, *h)) == bevorzugt {
                    t = gettext_f("{size} (preferred)", &[("size", &t)]);
                }
                t
            })
            .collect();
        let groesse_index = groessen
            .iter()
            .position(|g| *g == (w.modus.breite, w.modus.hoehe))
            .unwrap_or(0);
        // Raten zur Größe
        let mut raten: Vec<i32> = a
            .modi
            .iter()
            .filter(|m| (m.breite, m.hoehe) == (w.modus.breite, w.modus.hoehe))
            .map(|m| m.mhz)
            .collect();
        raten.sort_unstable_by(|x, y| y.cmp(x));
        raten.dedup();
        if raten.is_empty() {
            raten.push(w.modus.mhz.max(0));
        } else if !raten.contains(&w.modus.mhz) {
            raten.push(w.modus.mhz);
        }
        let raten_text: Vec<String> = raten
            .iter()
            .map(|r| Modus::neu(0, 0, *r).hz_text())
            .collect();
        let rate_index = raten.iter().position(|r| *r == w.modus.mhz).unwrap_or(0);
        let mut skalierungen: Vec<f64> = SKALIERUNGEN.to_vec();
        if !skalierungen
            .iter()
            .any(|s| (s - w.skalierung).abs() < 0.001)
        {
            skalierungen.push(w.skalierung);
            skalierungen.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
        }
        let skalierung_text_liste: Vec<String> = skalierungen
            .iter()
            .map(|s| format!("{} ×", skalierung_text(*s)))
            .collect();
        let skalierung_index = skalierungen
            .iter()
            .position(|s| (s - w.skalierung).abs() < 0.001)
            .unwrap_or(0);
        let drehung_index = Drehung::ALLE
            .iter()
            .position(|d| *d == w.drehung)
            .unwrap_or(0);
        let aktiv = w.aktiv;
        let haupt = w.haupt;
        let (lb, lh) = w.logische_groesse();
        let details = gettext_f(
            "{output}{description}{size} · Modes: {modes} · logical size {width}×{height} at {x},{y}{serial}",
            &[
                ("output", &a.name),
                (
                    "description",
                    &if a.beschreibung.trim().is_empty() {
                        String::new()
                    } else {
                        format!(" – {}", a.beschreibung.trim())
                    },
                ),
                (
                    "size",
                    &if a.physisch_mm.0 > 0 {
                        format!(" · {}×{} mm", a.physisch_mm.0, a.physisch_mm.1)
                    } else {
                        String::new()
                    },
                ),
                (
                    "modes",
                    &if frei {
                        gettext("no list (free sizes, e.g. virtual output)")
                    } else {
                        a.modi.len().to_string()
                    },
                ),
                ("width", &lb.to_string()),
                ("height", &lh.to_string()),
                ("x", &w.position.0.to_string()),
                ("y", &w.position.1.to_string()),
                (
                    "serial",
                    &if a.seriennummer.trim().is_empty() {
                        String::new()
                    } else {
                        gettext_f(
                            " · serial number {serial}",
                            &[("serial", a.seriennummer.trim())],
                        )
                    },
                ),
            ],
        );
        let unterschied = bm::unterschiedlich(&z.ist, &z.wahlen);
        drop(z);

        self.stumm.set(true);
        *self.groessen.borrow_mut() = groessen;
        *self.raten.borrow_mut() = raten;
        *self.skalierungen.borrow_mut() = skalierungen;
        liste_setzen(
            &self.listen,
            1,
            &self.aufloesung,
            &groessen_text,
            groesse_index,
        );
        liste_setzen(&self.listen, 2, &self.rate, &raten_text, rate_index);
        self.rate.set_sensitive(aktiv && raten_text.len() > 1);
        liste_setzen(
            &self.listen,
            3,
            &self.skalierung,
            &skalierung_text_liste,
            skalierung_index,
        );
        if self.drehung.selected() != drehung_index as u32 {
            self.drehung.set_selected(drehung_index as u32);
        }
        self.aktiv.set_active(aktiv);
        self.stumm.set(false);
        for w in [&self.aufloesung, &self.drehung, &self.skalierung] {
            w.set_sensitive(aktiv);
        }
        self.haupt.set_sensitive(aktiv && !haupt);
        self.details.set_text(&details);
        self.skalierung_hinweis_zeigen();
        self.anwenden.set_sensitive(unterschied);
        self.anordnung.queue_draw();
    }

    fn skalierung_hinweis_zeigen(&self) {
        let z = self.zustand.borrow();
        let s = z.wahlen.get(z.gewaehlt).map_or(1.0, |w| w.skalierung);
        drop(z);
        if (s - s.round()).abs() > 0.001 {
            self.skalierung_hinweis.set_text(&gettext(
                "Fractional scaling: GTK 4.10 draws with the next whole factor and scales down; the desktop then shows “Centered” and “Tiled” at ¾ of the pixel size. Whole factors (1 ×, 2 ×) are pixel-exact.",
            ));
        } else {
            self.skalierung_hinweis.set_text("");
        }
    }

    /// Nach einer Änderung an der Wahl: Liste, Felder, Anordnung, Anwenden.
    fn geaendert(&self) {
        self.monitorliste_fuellen();
        self.auswahl_zeigen();
    }

    /// Ändert die Wahl der gewählten Ausgabe; danach rastet sie neu ein
    /// (eine geänderte Größe darf die Nachbarn nicht überlappen).
    fn wahl_aendern(&self, f: impl FnOnce(&mut Wahl, &Ausgabe)) {
        let mut z = self.zustand.borrow_mut();
        let i = z.gewaehlt;
        let Zustand {
            ausgaben, wahlen, ..
        } = &mut *z;
        if let Some((w, a)) = wahlen.get_mut(i).zip(ausgaben.get(i)) {
            f(w, a);
        }
        bm::einrasten(wahlen, i, FANG);
        drop(z);
        self.geaendert();
    }

    fn neu_lesen(&self) -> Result<(), String> {
        let mut z = self.zustand.borrow_mut();
        let ausgaben = z
            .verbindung
            .as_mut()
            .ok_or_else(|| gettext("no connection"))?
            .lesen()?;
        let ist = bm::wahlen_aus(&ausgaben);
        z.ausgaben = ausgaben;
        z.wahlen = ist.clone();
        z.ist = ist;
        if z.gewaehlt >= z.ausgaben.len() {
            z.gewaehlt = 0;
        }
        Ok(())
    }

    fn anwenden_klick(self: &Rc<Self>) {
        let (vorher, soll) = {
            let mut z = self.zustand.borrow_mut();
            bm::normieren(&mut z.wahlen);
            (z.ist.clone(), z.wahlen.clone())
        };
        if !bm::unterschiedlich(&vorher, &soll) {
            self.status
                .set_text(&gettext("No change compared to the current state."));
            return;
        }
        let ergebnis = {
            let mut z = self.zustand.borrow_mut();
            let v = z.verbindung.as_mut();
            match v {
                None => Err(gettext("no connection to the compositor")),
                Some(v) => v
                    .anwenden(&soll, true)
                    .map_err(|e| format!("Test: {e}"))
                    .and_then(|()| v.anwenden(&soll, false)),
            }
        };
        match ergebnis {
            Err(e) => {
                self.status
                    .set_text(&gettext_f("Not applied – {error}", &[("error", &e)]));
                let _ = self.neu_lesen();
                let mut z = self.zustand.borrow_mut();
                z.wahlen = soll;
                drop(z);
                self.geaendert();
            }
            Ok(()) => {
                eprintln!(
                    "querencia-einstellungen: Bildschirm angewendet: {}",
                    kurz(&soll)
                );
                let _ = self.neu_lesen();
                self.geaendert();
                self.status.set_text(&gettext_f(
                    "Applied. Confirm within {seconds} s, otherwise the previous state is restored.",
                    &[("seconds", &COUNTDOWN_SEKUNDEN.to_string())],
                ));
                self.countdown(vorher, soll);
            }
        }
    }

    /// Dialog mit Countdown: „Behalten“ speichert, „Wiederherstellen“ oder
    /// Ablauf setzt `vorher` wieder.
    fn countdown(self: &Rc<Self>, vorher: Vec<Wahl>, soll: Vec<Wahl>) {
        let eltern = self
            .anordnung
            .root()
            .and_then(|r| r.downcast::<gtk4::Window>().ok());
        let dialog = gtk4::Window::builder()
            .title(gettext("Keep Screen Settings?"))
            .modal(true)
            .default_width(460)
            .resizable(false)
            .build();
        if let Some(e) = &eltern {
            dialog.set_transient_for(Some(e));
        }
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        inhalt.set_margin_top(16);
        inhalt.set_margin_bottom(14);
        inhalt.set_margin_start(16);
        inhalt.set_margin_end(16);
        let frage = gtk4::Label::new(Some(&gettext(
            "The new screen settings are active. Do you want to keep them?",
        )));
        frage.set_xalign(0.0);
        frage.set_wrap(true);
        let zaehler = gtk4::Label::new(None);
        zaehler.set_xalign(0.0);
        zaehler.add_css_class("dim-label");
        let knoepfe = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        knoepfe.set_halign(gtk4::Align::End);
        let wiederherstellen = gtk4::Button::with_label(&gettext("Restore Previous Settings"));
        let behalten = gtk4::Button::with_label(&gettext("Keep Settings"));
        behalten.add_css_class("suggested-action");
        knoepfe.append(&wiederherstellen);
        knoepfe.append(&behalten);
        inhalt.append(&frage);
        inhalt.append(&zaehler);
        inhalt.append(&knoepfe);
        dialog.set_child(Some(&inhalt));

        let verbleibend = Rc::new(Cell::new(COUNTDOWN_SEKUNDEN));
        let zaehler_text = |n: u32| {
            ngettext_f(
                "Without confirmation the previous settings are restored in {n} second.",
                "Without confirmation the previous settings are restored in {n} seconds.",
                n,
                &[],
            )
        };
        zaehler.set_text(&zaehler_text(COUNTDOWN_SEKUNDEN));
        let quelle: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        let entschieden = Rc::new(Cell::new(false));

        let zurueck = {
            let o = self.clone();
            let dialog = dialog.clone();
            let quelle = quelle.clone();
            let entschieden = entschieden.clone();
            let vorher = vorher.clone();
            move || {
                if entschieden.replace(true) {
                    return;
                }
                if let Some(q) = quelle.borrow_mut().take() {
                    q.remove();
                }
                let ergebnis = {
                    let mut z = o.zustand.borrow_mut();
                    match z.verbindung.as_mut() {
                        Some(v) => v.anwenden(&vorher, false),
                        None => Err(gettext("no connection")),
                    }
                };
                let _ = o.neu_lesen();
                o.geaendert();
                match ergebnis {
                    Ok(()) => {
                        eprintln!(
                            "querencia-einstellungen: Bildschirm zurückgesetzt auf {}",
                            kurz(&vorher)
                        );
                        o.status.set_text(&gettext(
                            "Previous screen settings restored; nothing saved.",
                        ));
                    }
                    Err(e) => o.status.set_text(&gettext_f(
                        "Restoring failed – {error}. The new state remains, nothing saved.",
                        &[("error", &e)],
                    )),
                }
                dialog.close();
            }
        };
        let zurueck = Rc::new(zurueck);

        {
            let o = self.clone();
            let dialog = dialog.clone();
            let quelle = quelle.clone();
            let entschieden = entschieden.clone();
            behalten.connect_clicked(move |_| {
                if entschieden.replace(true) {
                    return;
                }
                if let Some(q) = quelle.borrow_mut().take() {
                    q.remove();
                }
                o.speichern(&soll);
                dialog.close();
            });
        }
        {
            let zurueck = zurueck.clone();
            wiederherstellen.connect_clicked(move |_| zurueck());
        }
        {
            // Fenster schließen (×) zählt wie Wiederherstellen.
            let zurueck = zurueck.clone();
            dialog.connect_close_request(move |_| {
                zurueck();
                glib::Propagation::Proceed
            });
        }
        {
            let zurueck = zurueck.clone();
            let zaehler = zaehler.clone();
            let quelle2 = quelle.clone();
            let id = glib::timeout_add_seconds_local(1, move || {
                let n = verbleibend.get().saturating_sub(1);
                verbleibend.set(n);
                zaehler.set_text(&zaehler_text(n));
                if n == 0 {
                    *quelle2.borrow_mut() = None;
                    zurueck();
                    return glib::ControlFlow::Break;
                }
                glib::ControlFlow::Continue
            });
            *quelle.borrow_mut() = Some(id);
        }
        dialog.present();
    }

    fn speichern(&self, soll: &[Wahl]) {
        let (ausgaben, aktuell) = {
            let z = self.zustand.borrow();
            (z.ausgaben.clone(), z.ist.clone())
        };
        // gespeichert wird der bestätigte Zustand, wie der Compositor ihn jetzt meldet
        let mut wahlen = aktuell;
        for w in wahlen.iter_mut() {
            if let Some(s) = soll.iter().find(|s| s.name == w.name) {
                w.haupt = s.haupt;
            }
        }
        let pfad = bm::konfig_pfad();
        match bm::konfig_speichern(&pfad, &bm::konfig_schreiben(&wahlen, &ausgaben)) {
            Ok(()) => {
                eprintln!("querencia-einstellungen: {} geschrieben", pfad.display());
                self.status.set_text(&gettext_f(
                    "Screen settings kept and saved in {file}. They are applied again at session start.",
                    &[("file", &pfad.display().to_string())],
                ));
            }
            Err(e) => self.status.set_text(&gettext_f(
                "Settings active, but not saved: {error}",
                &[("error", &e)],
            )),
        }
        let mut z = self.zustand.borrow_mut();
        z.wahlen = wahlen.clone();
        z.ist = wahlen;
        drop(z);
        self.geaendert();
    }

    // ------------------------------------------------------------ Anordnung

    /// Maßstab und Versatz, damit alle aktiven Ausgaben in die Fläche passen.
    fn massstab_berechnen(&self, breite: f64, hoehe: f64) -> (f64, f64, f64) {
        let z = self.zustand.borrow();
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for w in z.wahlen.iter().filter(|w| w.aktiv) {
            let (b, h) = w.logische_groesse();
            min_x = min_x.min(w.position.0);
            min_y = min_y.min(w.position.1);
            max_x = max_x.max(w.position.0 + b);
            max_y = max_y.max(w.position.1 + h);
        }
        if min_x == i32::MAX {
            return (0.1, 0.0, 0.0);
        }
        let bb = f64::from((max_x - min_x).max(1));
        let bh = f64::from((max_y - min_y).max(1));
        let rand = 16.0;
        let m = ((breite - 2.0 * rand) / bb)
            .min((hoehe - 2.0 * rand) / bh)
            .clamp(0.01, 0.5);
        let vx = (breite - bb * m) / 2.0 - f64::from(min_x) * m;
        let vy = (hoehe - bh * m) / 2.0 - f64::from(min_y) * m;
        (m, vx, vy)
    }

    fn zeichnen(&self, cr: &gtk4::cairo::Context, breite: i32, hoehe: i32) {
        let (b, h) = (f64::from(breite), f64::from(hoehe));
        cr.set_source_rgb(0.97, 0.97, 0.97);
        cr.rectangle(0.0, 0.0, b, h);
        let _ = cr.fill();
        let ziehe = self.zustand.borrow().ziehe.is_some();
        let (m, vx, vy) = if ziehe {
            self.zustand.borrow().massstab
        } else {
            let ms = self.massstab_berechnen(b, h);
            self.zustand.borrow_mut().massstab = ms;
            ms
        };
        let z = self.zustand.borrow();
        if z.wahlen.iter().all(|w| !w.aktiv) {
            cr.set_source_rgb(0.4, 0.4, 0.4);
            cr.move_to(16.0, 30.0);
            cr.set_font_size(14.0);
            let _ = cr.show_text(&gettext("No output switched on"));
            return;
        }
        for (i, w) in z.wahlen.iter().enumerate() {
            if !w.aktiv {
                continue;
            }
            let (lb, lh) = w.logische_groesse();
            let x = f64::from(w.position.0) * m + vx;
            let y = f64::from(w.position.1) * m + vy;
            let rb = f64::from(lb) * m;
            let rh = f64::from(lh) * m;
            let gewaehlt = i == z.gewaehlt;
            if gewaehlt {
                cr.set_source_rgb(0.21, 0.52, 0.89);
            } else {
                cr.set_source_rgb(0.72, 0.74, 0.78);
            }
            cr.rectangle(x, y, rb, rh);
            let _ = cr.fill();
            cr.set_source_rgb(0.25, 0.25, 0.25);
            cr.set_line_width(if gewaehlt { 2.0 } else { 1.0 });
            cr.rectangle(x + 0.5, y + 0.5, rb - 1.0, rh - 1.0);
            let _ = cr.stroke();
            if w.haupt {
                // Panel-Andeutung oben: der Hauptmonitor trägt die Leiste
                cr.set_source_rgb(0.15, 0.15, 0.15);
                cr.rectangle(x + 1.0, y + 1.0, (rb - 2.0).max(0.0), 5.0_f64.min(rh));
                let _ = cr.fill();
            }
            if gewaehlt {
                cr.set_source_rgb(1.0, 1.0, 1.0);
            } else {
                cr.set_source_rgb(0.1, 0.1, 0.1);
            }
            cr.set_font_size(if rb < 120.0 { 10.0 } else { 13.0 });
            let name = z
                .ausgaben
                .get(i)
                .map(|a| a.name.clone())
                .unwrap_or_else(|| w.name.clone());
            let text2 = format!("{lb} × {lh}");
            cr.save().ok();
            cr.rectangle(x + 2.0, y + 2.0, (rb - 4.0).max(0.0), (rh - 4.0).max(0.0));
            cr.clip();
            cr.move_to(x + 6.0, y + 20.0);
            let _ = cr.show_text(&name);
            if rh >= 44.0 {
                cr.move_to(x + 6.0, y + 36.0);
                let _ = cr.show_text(&text2);
            }
            cr.restore().ok();
        }
    }

    fn treffer(&self, px: f64, py: f64) -> Option<usize> {
        let z = self.zustand.borrow();
        let (m, vx, vy) = z.massstab;
        z.wahlen
            .iter()
            .enumerate()
            .filter(|(_, w)| w.aktiv)
            .find(|(_, w)| {
                let (lb, lh) = w.logische_groesse();
                let x = f64::from(w.position.0) * m + vx;
                let y = f64::from(w.position.1) * m + vy;
                px >= x && px <= x + f64::from(lb) * m && py >= y && py <= y + f64::from(lh) * m
            })
            .map(|(i, _)| i)
    }
}

fn kurz(wahlen: &[Wahl]) -> String {
    wahlen
        .iter()
        .map(|w| {
            if w.aktiv {
                format!(
                    "{} {} @{},{} ×{} {}",
                    w.name,
                    w.modus.dateiwert(),
                    w.position.0,
                    w.position.1,
                    skalierung_text(w.skalierung),
                    w.drehung.dateiwert()
                )
            } else {
                format!("{} aus", w.name)
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn verdrahten(o: &Rc<Oberflaeche>) {
    {
        let o = o.clone();
        o.clone()
            .anordnung
            .set_draw_func(move |_, cr, b, h| o.zeichnen(cr, b, h));
    }
    {
        let o2 = o.clone();
        o.monitor.connect_selected_notify(move |d| {
            if o2.stumm.get() {
                return;
            }
            let i = d.selected() as usize;
            let mut z = o2.zustand.borrow_mut();
            if i < z.wahlen.len() {
                z.gewaehlt = i;
            }
            drop(z);
            o2.auswahl_zeigen();
        });
    }
    {
        let o2 = o.clone();
        o.aktiv.connect_toggled(move |c| {
            if o2.stumm.get() {
                return;
            }
            let an = c.is_active();
            o2.wahl_aendern(|w, a| {
                w.aktiv = an;
                if an && w.modus.breite <= 0 {
                    w.modus = a.bevorzugter_modus().unwrap_or(Modus::neu(1024, 768, 0));
                }
            });
            // ausgeschaltete Hauptmonitore verlieren das Merkmal
            let mut z = o2.zustand.borrow_mut();
            if !an && z.wahlen.get(z.gewaehlt).is_some_and(|w| w.haupt) {
                let i = z.gewaehlt;
                z.wahlen[i].haupt = false;
                if let Some(j) = z.wahlen.iter().position(|w| w.aktiv) {
                    z.wahlen[j].haupt = true;
                }
            }
            drop(z);
            o2.geaendert();
        });
    }
    {
        let o2 = o.clone();
        o.aufloesung.connect_selected_notify(move |d| {
            if o2.stumm.get() {
                return;
            }
            let i = d.selected() as usize;
            let Some(&(b, h)) = o2.groessen.borrow().get(i) else {
                return;
            };
            o2.wahl_aendern(|w, a| {
                // bevorzugte Rate dieser Größe, sonst die höchste, sonst automatisch
                let mut raten: Vec<&Modus> = a
                    .modi
                    .iter()
                    .filter(|m| (m.breite, m.hoehe) == (b, h))
                    .collect();
                raten.sort_by(|x, y| (y.bevorzugt, y.mhz).cmp(&(x.bevorzugt, x.mhz)));
                w.modus = raten.first().map(|m| **m).unwrap_or(Modus::neu(b, h, 0));
            });
        });
    }
    {
        let o2 = o.clone();
        o.rate.connect_selected_notify(move |d| {
            if o2.stumm.get() {
                return;
            }
            let i = d.selected() as usize;
            let Some(&mhz) = o2.raten.borrow().get(i) else {
                return;
            };
            o2.wahl_aendern(|w, _| w.modus.mhz = mhz);
        });
    }
    {
        let o2 = o.clone();
        o.drehung.connect_selected_notify(move |d| {
            if o2.stumm.get() {
                return;
            }
            let dr = Drehung::ALLE[(d.selected() as usize).min(3)];
            o2.wahl_aendern(|w, _| w.drehung = dr);
        });
    }
    {
        let o2 = o.clone();
        o.skalierung.connect_selected_notify(move |d| {
            if o2.stumm.get() {
                return;
            }
            let i = d.selected() as usize;
            let Some(&s) = o2.skalierungen.borrow().get(i) else {
                return;
            };
            o2.wahl_aendern(|w, _| w.skalierung = s);
        });
    }
    {
        let o2 = o.clone();
        o.haupt.connect_clicked(move |_| {
            let mut z = o2.zustand.borrow_mut();
            let i = z.gewaehlt;
            bm::hauptmonitor_setzen(&mut z.wahlen, i);
            drop(z);
            o2.geaendert();
        });
    }
    {
        let o2 = o.clone();
        o.zuruecksetzen
            .connect_clicked(move |_| match o2.neu_lesen() {
                Ok(()) => {
                    o2.geaendert();
                    o2.status
                        .set_text(&gettext("Reset to the current state of the compositor."));
                }
                Err(e) => o2
                    .status
                    .set_text(&gettext_f("Reading failed – {error}", &[("error", &e)])),
            });
    }
    {
        let o2 = o.clone();
        o.anwenden.connect_clicked(move |_| o2.anwenden_klick());
    }
    // Ziehen in der Anordnung
    let geste = gtk4::GestureDrag::new();
    geste.set_button(gdk::BUTTON_PRIMARY);
    {
        let o2 = o.clone();
        geste.connect_drag_begin(move |_, x, y| {
            let Some(i) = o2.treffer(x, y) else {
                return;
            };
            let mut z = o2.zustand.borrow_mut();
            let start = z.wahlen[i].position;
            z.ziehe = Some((i, start));
            z.gewaehlt = i;
            drop(z);
            o2.monitorliste_fuellen();
            o2.auswahl_zeigen();
        });
    }
    {
        let o2 = o.clone();
        geste.connect_drag_update(move |_, dx, dy| {
            let mut z = o2.zustand.borrow_mut();
            let Some((i, start)) = z.ziehe else {
                return;
            };
            let m = z.massstab.0.max(0.001);
            z.wahlen[i].position = (
                start.0 + (dx / m).round() as i32,
                start.1 + (dy / m).round() as i32,
            );
            drop(z);
            o2.anordnung.queue_draw();
        });
    }
    {
        let o2 = o.clone();
        geste.connect_drag_end(move |_, _, _| {
            let mut z = o2.zustand.borrow_mut();
            let Some((i, _)) = z.ziehe.take() else {
                return;
            };
            bm::einrasten(&mut z.wahlen, i, FANG);
            drop(z);
            o2.geaendert();
        });
    }
    o.anordnung.add_controller(geste);
}

/// `--bildschirm-anwenden`: gespeicherte Konfiguration setzen (Sitzungsstart).
/// Exit 0 gesetzt oder nichts zu tun, 1 Fehler, 2 keine Datei.
pub fn gespeicherte_anwenden() -> i32 {
    let pfad = bm::konfig_pfad();
    let text = match std::fs::read_to_string(&pfad) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "querencia-einstellungen: {}: keine Datei, nichts zu tun",
                pfad.display()
            );
            return 2;
        }
        Err(e) => {
            eprintln!("querencia-einstellungen: {}: {e}", pfad.display());
            return 1;
        }
    };
    let (eintraege, probleme) = bm::konfig_lesen(&text);
    for p in &probleme {
        eprintln!("querencia-einstellungen: {}: {p}", pfad.display());
    }
    let mut v = match Verbindung::oeffnen() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("querencia-einstellungen: Bildschirm: {e}");
            return 1;
        }
    };
    let ausgaben = match v.lesen() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("querencia-einstellungen: Bildschirm: {e}");
            return 1;
        }
    };
    let (wahlen, treffer) = bm::zuordnen(&eintraege, &ausgaben);
    if treffer == 0 {
        eprintln!(
            "querencia-einstellungen: Bildschirm: keine der {} Ausgaben steht in {}; nichts geändert",
            ausgaben.len(),
            pfad.display()
        );
        return 0;
    }
    let ist = bm::wahlen_aus(&ausgaben);
    if !bm::unterschiedlich(&ist, &wahlen) {
        eprintln!(
            "querencia-einstellungen: Bildschirm: Zustand entspricht schon {}",
            pfad.display()
        );
        return 0;
    }
    if wahlen.iter().all(|w| !w.aktiv) {
        eprintln!(
            "querencia-einstellungen: Bildschirm: die Datei würde alle Ausgaben ausschalten; nichts geändert"
        );
        return 1;
    }
    match v
        .anwenden(&wahlen, true)
        .map_err(|e| format!("Test: {e}"))
        .and_then(|()| v.anwenden(&wahlen, false))
    {
        Ok(()) => {
            eprintln!(
                "querencia-einstellungen: Bildschirm gesetzt aus {} ({} von {} Ausgaben): {}",
                pfad.display(),
                treffer,
                ausgaben.len(),
                kurz(&wahlen)
            );
            0
        }
        Err(e) => {
            eprintln!("querencia-einstellungen: Bildschirm nicht gesetzt – {e}");
            1
        }
    }
}

/// `--bildschirm-zeigen`: Ist-Zustand als Text (für Prüfwerkzeuge).
pub fn zeigen() -> i32 {
    match Verbindung::oeffnen().and_then(|mut v| v.lesen().map(|a| (v.protokollversion(), a))) {
        Ok((version, a)) => {
            print!("{}", crate::ausgaben::zeilen(&a));
            eprintln!(
                "querencia-einstellungen: zwlr_output_manager_v1 Version {version}, {} Ausgaben",
                a.len()
            );
            0
        }
        Err(e) => {
            eprintln!("querencia-einstellungen: Bildschirm: {e}");
            1
        }
    }
}
