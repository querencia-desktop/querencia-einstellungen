//! Bereich „Arbeitsfläche“ (QD-042, G2-X): welche Symbole und eingehängten
//! Laufwerke der Desktop zeigt und wie groß. Schreibt erst bei „Übernehmen“
//! die Gruppen `[Icons]` und `[Volumes]` in `~/.config/querencia/desktop.ini`
//! (atomar, mit Sicherung, `[Background]` und alles andere bleibt byte-gleich);
//! Format: `docs/desktop-ini-arbeitsflaeche.md`.
//!
//! Ob der laufende Desktop die Gruppen schon auswertet, sagt seine
//! Fähigkeitsliste (`volumes-types`, `icons-visible`, `icon-size`); ohne sie
//! erscheint ein Hinweis, Übernehmen bleibt möglich (ältere Desktops
//! überspringen die Gruppen).
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gio, glib};

use crate::arbeitsflaeche_modell::{self as am, Arbeitsflaeche, Symbolgroesse};
use crate::modell::Faehigkeiten;
use crate::{faehigkeiten, hintergrund, ini};

pub const TITEL: &str = "Querencia – Arbeitsfläche";

struct Oberflaeche {
    rechner: gtk4::CheckButton,
    persoenlich: gtk4::CheckButton,
    papierkorb: gtk4::CheckButton,
    groesse: gtk4::DropDown,
    wechsel: gtk4::CheckButton,
    optisch: gtk4::CheckButton,
    netz: gtk4::CheckButton,
    eingebaut: gtk4::CheckButton,
    hinweis: gtk4::Label,
    status: gtk4::Label,
    standard: gtk4::Button,
    neu_laden: gtk4::Button,
    uebernehmen: gtk4::Button,
    /// Stand der Datei beim Laden: Werte und der Text der eigenen Gruppen
    /// (für „nicht unbemerkt überschreiben“).
    geladen: RefCell<(Arbeitsflaeche, String)>,
    stumm: Cell<bool>,
    faehigkeiten: Cell<Option<Faehigkeiten>>,
}

/// Eigenes Fenster (`--arbeitsflaeche`).
pub fn oeffnen(app: &gtk4::Application) -> gtk4::Window {
    let hoehe = hintergrund::bildschirmhoehe().map_or(600, |h| (h - 130).clamp(420, 600));
    let fenster = gtk4::Window::builder()
        .application(app)
        .title(TITEL)
        .default_width(640)
        .default_height(hoehe)
        .build();
    let (inhalt, fuss) = seite_teile(true);
    let rollbar = gtk4::ScrolledWindow::new();
    rollbar.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    rollbar.set_child(Some(&inhalt));
    rollbar.set_vexpand(true);
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    aussen.append(&rollbar);
    aussen.append(&fuss);
    fenster.set_child(Some(&aussen));
    fenster
}

/// Seite im Kontrollzentrum.
pub fn seite() -> gtk4::Widget {
    let (inhalt, fuss) = seite_teile(false);
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    aussen.append(&inhalt);
    aussen.append(&fuss);
    aussen.upcast()
}

fn rahmen(titel: &str) -> (gtk4::Frame, gtk4::Box) {
    let f = gtk4::Frame::new(Some(titel));
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    b.set_margin_top(6);
    b.set_margin_bottom(6);
    b.set_margin_start(8);
    b.set_margin_end(8);
    f.set_child(Some(&b));
    (f, b)
}

/// Kästchen mit grauer Erläuterung dahinter.
fn kaestchen(text: &str, erlaeuterung: &str) -> (gtk4::CheckButton, gtk4::Box) {
    let k = gtk4::CheckButton::with_label(text);
    let zeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    zeile.append(&k);
    if !erlaeuterung.is_empty() {
        let l = gtk4::Label::new(Some(erlaeuterung));
        l.add_css_class("dim-label");
        l.set_xalign(0.0);
        l.set_wrap(true);
        l.set_hexpand(true);
        zeile.append(&l);
    }
    (k, zeile)
}

fn seite_teile(mit_schliessen: bool) -> (gtk4::Box, gtk4::Box) {
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    aussen.set_margin_top(12);
    aussen.set_margin_bottom(12);
    aussen.set_margin_start(14);
    aussen.set_margin_end(14);
    let ueberschrift = gtk4::Label::new(None);
    ueberschrift.set_markup("<b><big>Arbeitsfläche</big></b>");
    ueberschrift.set_xalign(0.0);
    aussen.append(&ueberschrift);

    // Symbolgröße zuoberst: die Liste klappt nach unten auf (nach oben
    // geklappte Listen bekommen unter labwc 0.9.8 keine Klicks, G2-G).
    let (f_symbole, b_symbole) = rahmen("Symbole auf der Arbeitsfläche");
    let groessenzeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    groessenzeile.append(&gtk4::Label::new(Some("Symbolgröße:")));
    let groesse = gtk4::DropDown::from_strings(
        &Symbolgroesse::ALLE
            .iter()
            .map(|g| g.beschriftung())
            .collect::<Vec<_>>(),
    );
    groessenzeile.append(&groesse);
    b_symbole.append(&groessenzeile);
    let (rechner, z) = kaestchen("Rechner", "");
    b_symbole.append(&z);
    let (persoenlich, z) = kaestchen("Persönlicher Ordner", "");
    b_symbole.append(&z);
    let (papierkorb, z) = kaestchen("Papierkorb", "");
    b_symbole.append(&z);
    aussen.append(&f_symbole);

    let (f_laufwerke, b_laufwerke) = rahmen("Eingehängte Laufwerke auf der Arbeitsfläche zeigen");
    let (wechsel, z) = kaestchen(
        "Wechseldatenträger",
        "USB-Sticks, Speicherkarten, externe Festplatten",
    );
    b_laufwerke.append(&z);
    let (optisch, z) = kaestchen("Optische Medien", "CD, DVD");
    b_laufwerke.append(&z);
    let (netz, z) = kaestchen(
        "Netzlaufwerke",
        "eingehängte Freigaben (SMB, SFTP, FTP, WebDAV)",
    );
    b_laufwerke.append(&z);
    let (eingebaut, z) = kaestchen(
        "Eingebaute Laufwerke",
        "interne Festplatten und Partitionen",
    );
    b_laufwerke.append(&z);
    aussen.append(&f_laufwerke);

    let hinweis = gtk4::Label::new(None);
    hinweis.set_xalign(0.0);
    hinweis.set_wrap(true);
    hinweis.set_max_width_chars(70);
    hinweis.set_visible(false);
    aussen.append(&hinweis);
    let quelle = gtk4::Label::new(Some(
        "Quelle: ~/.config/querencia/desktop.ini [Icons] und [Volumes]; Querencia Desktop übernimmt Änderungen sofort.",
    ));
    quelle.set_xalign(0.0);
    quelle.set_wrap(true);
    quelle.add_css_class("dim-label");
    aussen.append(&quelle);

    // Fuß: Statuszeile und Schaltflächen
    let fuss = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    fuss.set_margin_start(14);
    fuss.set_margin_end(14);
    fuss.set_margin_bottom(12);
    let status = gtk4::Label::new(None);
    status.set_xalign(0.0);
    status.set_wrap(true);
    status.set_max_width_chars(70);
    fuss.append(&status);
    let knoepfe = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let standard = gtk4::Button::with_label("Standard wiederherstellen");
    let neu_laden = gtk4::Button::with_label("Neu laden");
    let uebernehmen = gtk4::Button::with_label("Übernehmen");
    uebernehmen.add_css_class("suggested-action");
    knoepfe.append(&standard);
    knoepfe.append(&neu_laden);
    let platz = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    platz.set_hexpand(true);
    knoepfe.append(&platz);
    if mit_schliessen {
        let schliessen = gtk4::Button::with_label("Schließen");
        schliessen.connect_clicked(|b| {
            if let Some(w) = b.root().and_then(|r| r.downcast::<gtk4::Window>().ok()) {
                w.close();
            }
        });
        knoepfe.append(&schliessen);
    }
    knoepfe.append(&uebernehmen);
    fuss.append(&knoepfe);

    let o = Rc::new(Oberflaeche {
        rechner,
        persoenlich,
        papierkorb,
        groesse,
        wechsel,
        optisch,
        netz,
        eingebaut,
        hinweis,
        status,
        standard,
        neu_laden,
        uebernehmen,
        geladen: RefCell::new((Arbeitsflaeche::default(), String::new())),
        stumm: Cell::new(false),
        faehigkeiten: Cell::new(None),
    });
    o.laden();
    verdrahten(&o);

    // Fähigkeiten im Hintergrund abfragen (bis zu 2 s); die Seite ist sofort bedienbar.
    let o2 = o.clone();
    let auftrag = gio::spawn_blocking(faehigkeiten::ermitteln);
    glib::spawn_future_local(async move {
        let f = auftrag.await.unwrap_or(Faehigkeiten::HMP10A);
        o2.faehigkeiten.set(Some(f));
        o2.hinweis_zeigen();
    });
    (aussen, fuss)
}

fn verdrahten(o: &Rc<Oberflaeche>) {
    for k in o.kaestchen() {
        let o2 = o.clone();
        k.connect_toggled(move |_| o2.geaendert());
    }
    let o2 = o.clone();
    o.groesse.connect_selected_notify(move |_| o2.geaendert());
    let o2 = o.clone();
    o.standard.connect_clicked(move |_| {
        o2.zeigen(&Arbeitsflaeche::default());
        o2.geaendert();
    });
    let o2 = o.clone();
    o.neu_laden.connect_clicked(move |_| {
        o2.laden();
        o2.status.set_text("Neu geladen.");
    });
    let o2 = o.clone();
    o.uebernehmen.connect_clicked(move |_| o2.uebernehmen());
}

impl Oberflaeche {
    fn kaestchen(&self) -> [&gtk4::CheckButton; 7] {
        [
            &self.rechner,
            &self.persoenlich,
            &self.papierkorb,
            &self.wechsel,
            &self.optisch,
            &self.netz,
            &self.eingebaut,
        ]
    }

    fn laden(&self) {
        let text = std::fs::read_to_string(hintergrund::konfig_pfad()).unwrap_or_default();
        let (a, probleme) = Arbeitsflaeche::lesen(&text);
        for p in &probleme {
            eprintln!("querencia-einstellungen: desktop.ini: {p}");
        }
        *self.geladen.borrow_mut() = (a, am::eigene_gruppen(&text));
        self.zeigen(&a);
        self.uebernehmen.set_sensitive(false);
        self.status.set_text(&if probleme.is_empty() {
            "Änderungen gelten erst nach „Übernehmen“.".to_owned()
        } else {
            format!(
                "Hinweis: {} Eintrag/Einträge in desktop.ini wurden ignoriert (siehe Protokoll); es gilt der Standard.",
                probleme.len()
            )
        });
        eprintln!(
            "querencia-einstellungen: Arbeitsfläche geladen: {}",
            kurz(&a)
        );
    }

    fn zeigen(&self, a: &Arbeitsflaeche) {
        self.stumm.set(true);
        self.rechner.set_active(a.rechner);
        self.persoenlich.set_active(a.persoenlich);
        self.papierkorb.set_active(a.papierkorb);
        self.wechsel.set_active(a.wechsel);
        self.optisch.set_active(a.optisch);
        self.netz.set_active(a.netz);
        self.eingebaut.set_active(a.eingebaut);
        let i = Symbolgroesse::ALLE
            .iter()
            .position(|g| *g == a.groesse)
            .unwrap_or(1);
        self.groesse.set_selected(i as u32);
        self.stumm.set(false);
    }

    fn aktuell(&self) -> Arbeitsflaeche {
        Arbeitsflaeche {
            rechner: self.rechner.is_active(),
            persoenlich: self.persoenlich.is_active(),
            papierkorb: self.papierkorb.is_active(),
            groesse: Symbolgroesse::ALLE
                .get(self.groesse.selected() as usize)
                .copied()
                .unwrap_or_default(),
            wechsel: self.wechsel.is_active(),
            optisch: self.optisch.is_active(),
            netz: self.netz.is_active(),
            eingebaut: self.eingebaut.is_active(),
        }
    }

    fn geaendert(&self) {
        if self.stumm.get() {
            return;
        }
        let anders = self.aktuell() != self.geladen.borrow().0;
        self.uebernehmen.set_sensitive(anders);
        self.status.set_text(if anders {
            "Geändert – „Übernehmen“ schreibt desktop.ini."
        } else {
            "Keine Änderung gegenüber desktop.ini."
        });
    }

    /// Hinweis, wenn der laufende Desktop die Gruppen (noch) nicht auswertet.
    fn hinweis_zeigen(&self) {
        let Some(f) = self.faehigkeiten.get() else {
            return;
        };
        let mut fehlt = Vec::new();
        if !f.laufwerkstypen {
            fehlt.push("die Laufwerkstypen");
        }
        if !f.symbole_sichtbar {
            fehlt.push("das Ein- und Ausblenden von Rechner, Persönlichem Ordner und Papierkorb");
        }
        if !f.symbolgroesse {
            fehlt.push("die Symbolgröße");
        }
        if fehlt.is_empty() {
            self.hinweis.set_visible(false);
            eprintln!("querencia-einstellungen: Arbeitsfläche: Desktop wertet alles aus");
            return;
        }
        let was = if fehlt.len() == 3 {
            "diese Einstellungen".to_owned()
        } else {
            fehlt.join(", ")
        };
        let text = format!(
            "Hinweis: Der laufende Desktop wertet {was} noch nicht aus. Sie werden trotzdem gespeichert und gelten, sobald ein neuerer Querencia Desktop läuft."
        );
        eprintln!("querencia-einstellungen: Arbeitsfläche: {text}");
        self.hinweis.set_text(&text);
        self.hinweis.set_visible(true);
    }

    fn uebernehmen(&self) {
        let pfad = hintergrund::konfig_pfad();
        let neu = self.aktuell();
        let jetzt = std::fs::read_to_string(&pfad).unwrap_or_default();
        // Nicht unbemerkt überschreiben – aber nur die eigenen Gruppen zählen;
        // ein inzwischen geänderter Hintergrund bleibt einfach erhalten.
        if am::eigene_gruppen(&jetzt) != self.geladen.borrow().1 {
            self.status.set_text(
                "Die Einstellungen der Arbeitsfläche in desktop.ini wurden inzwischen von anderer Stelle geändert – nicht überschrieben. „Neu laden“ zeigt den aktuellen Stand.",
            );
            eprintln!("querencia-einstellungen: Arbeitsfläche: Konflikt, nicht geschrieben");
            return;
        }
        let text = neu.in_datei(&jetzt);
        let zeit = jetzt_text("%Y%m%d-%H%M%S").unwrap_or_else(|| "0".into());
        match ini::schreiben(&pfad, &text, &jetzt, &zeit) {
            Ok(sicherung) => {
                *self.geladen.borrow_mut() = (neu, am::eigene_gruppen(&text));
                self.uebernehmen.set_sensitive(false);
                let uhr = jetzt_text("%H:%M:%S").unwrap_or_default();
                self.status.set_text(&match sicherung {
                    Some(s) => format!(
                        "Übernommen um {uhr}. Vorherige Einstellung gesichert als {}.",
                        s.file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default()
                    ),
                    None => format!("Übernommen um {uhr}."),
                });
                eprintln!(
                    "querencia-einstellungen: Arbeitsfläche übernommen: {}",
                    kurz(&neu)
                );
            }
            Err(err) => {
                self.status.set_text(&format!("Nicht gespeichert: {err}"));
                eprintln!("querencia-einstellungen: Arbeitsfläche nicht gespeichert: {err}");
            }
        }
    }
}

fn jetzt_text(format: &str) -> Option<String> {
    glib::DateTime::now_local()
        .ok()
        .and_then(|d| d.format(format).ok())
        .map(|s| s.to_string())
}

/// Für das Protokoll (und die GUI-Prüfung).
fn kurz(a: &Arbeitsflaeche) -> String {
    let j = |b: bool| if b { "an" } else { "aus" };
    format!(
        "Rechner {} Persönlich {} Papierkorb {} Größe {} | Wechsel {} Optisch {} Netz {} Eingebaut {}",
        j(a.rechner),
        j(a.persoenlich),
        j(a.papierkorb),
        a.groesse.pixel(),
        j(a.wechsel),
        j(a.optisch),
        j(a.netz),
        j(a.eingebaut)
    )
}
