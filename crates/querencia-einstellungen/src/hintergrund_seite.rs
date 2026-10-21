//! Seite „Hintergrundbild“ im Kontrollzentrum (G2-AJ): die Galerie der
//! eingebauten Bilder (G2-AF) direkt auf der Seite – Kachel anklicken,
//! Vorschau prüfen, „Übernehmen“. Geschrieben wird wie im Dialog nur die
//! Gruppe `[Background]` von `desktop.ini` (atomar, mit Sicherung), mit
//! derselben Konfliktprüfung; Darstellung und Farbe bleiben, wie sie sind.
//! Eigene Bilder, Farbe und Darstellung stellt weiter der Dialog
//! „Hintergrund …“ ein. Ändert jemand die Datei (der Dialog, der Desktop),
//! zeigt die Seite den neuen Stand, sofern keine Auswahl offen ist.
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};

use crate::bereiche;
use crate::faehigkeiten;
use crate::hintergrund::{self as hg, GRUPPE};
use crate::hintergrund_katalog::{Format, Wahl};
use crate::i18n::{gettext, gettext_f};
use crate::ini;
use crate::modell::{Faehigkeiten, Hintergrund, Typ};
use crate::vorschau::Vorschau;

struct Seite {
    vorschau: Vorschau,
    name: gtk4::Label,
    darstellung: gtk4::Label,
    galerie: gtk4::FlowBox,
    kacheln: Vec<(Wahl, gtk4::FlowBoxChild)>,
    bildschirm: Format,
    status: gtk4::Label,
    verwerfen: gtk4::Button,
    uebernehmen: gtk4::Button,
    geladen: RefCell<Hintergrund>,
    geladener_text: RefCell<String>,
    /// gewählte, noch nicht übernommene Einstellung
    gewaehlt: RefCell<Option<Hintergrund>>,
    faehigkeiten: Cell<Option<Faehigkeiten>>,
    stumm: Cell<bool>,
    /// beobachtet desktop.ini, solange die Seite lebt
    monitor: RefCell<Option<gio::FileMonitor>>,
}

/// Name der Einstellung für Menschen.
fn beschreibung(h: &Hintergrund) -> String {
    if h.typ == Typ::Farbe {
        return gettext_f("Solid color {color}", &[("color", &h.farbe.hex())]);
    }
    // leeres `File=` ist immer eine Kachel (das eingebaute Bild)
    match hg::erkannt(h.datei.as_deref()) {
        Some(w) => gettext_f("Built-in image “{name}”", &[("name", &w.name())]),
        None => h
            .datei
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    }
}

pub fn seite(app: &gtk4::Application) -> gtk4::Widget {
    let aussen = bereiche::seite(&gettext("Background Image"));

    let kopf = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
    let vorschau = Vorschau::default();
    vorschau.set_size_request(256, 144);
    vorschau.set_valign(gtk4::Align::Start);
    if let Some(m) = gdk::Display::default()
        .and_then(|d| d.monitors().iter::<gdk::Monitor>().find_map(Result::ok))
    {
        let g = m.geometry();
        vorschau.set_flaeche(f64::from(g.width()), f64::from(g.height()));
    }
    kopf.append(&vorschau);
    let rechts = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    let titel = gtk4::Label::new(Some(&gettext("Current background:")));
    titel.set_xalign(0.0);
    titel.add_css_class("dim-label");
    rechts.append(&titel);
    let name = gtk4::Label::new(None);
    name.set_xalign(0.0);
    name.set_wrap(true);
    name.set_max_width_chars(36);
    rechts.append(&name);
    let darstellung = gtk4::Label::new(None);
    darstellung.set_xalign(0.0);
    darstellung.add_css_class("dim-label");
    rechts.append(&darstellung);
    let knopf = gtk4::Button::with_label(&gettext("Background …"));
    knopf.set_halign(gtk4::Align::Start);
    knopf.set_margin_top(6);
    knopf.set_tooltip_text(Some(&gettext(
        "Own image, solid-color background and style",
    )));
    {
        let app = app.clone();
        knopf.connect_clicked(move |_| hg::oeffnen(&app).present());
    }
    rechts.append(&knopf);
    let knopf_hinweis = gtk4::Label::new(Some(&gettext("Own image, color and style")));
    knopf_hinweis.set_xalign(0.0);
    knopf_hinweis.add_css_class("dim-label");
    rechts.append(&knopf_hinweis);
    kopf.append(&rechts);
    aussen.append(&kopf);

    let rahmen = gtk4::Frame::new(Some(&gettext("Built-in Images")));
    let innen = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    innen.set_margin_top(6);
    innen.set_margin_bottom(6);
    innen.set_margin_start(8);
    innen.set_margin_end(8);
    let bildschirm = hg::bildschirm_format();
    let (galerie, kacheln) = hg::galerie_bauen(bildschirm);
    innen.append(&galerie);
    let hinweis = gtk4::Label::new(Some(&gettext_f(
        "A click selects the image for the preview, “Apply” sets it as the background. The {format} version is used for this screen.",
        &[("format", bildschirm.beschriftung())],
    )));
    hinweis.set_xalign(0.0);
    hinweis.set_wrap(true);
    hinweis.set_max_width_chars(70);
    hinweis.add_css_class("dim-label");
    innen.append(&hinweis);
    rahmen.set_child(Some(&innen));
    aussen.append(&rahmen);
    aussen.append(&bereiche::quelle(&gettext(
        "~/.config/querencia/desktop.ini [Background]; Querencia Desktop draws the background and reloads the file immediately when it changes.",
    )));

    let status = gtk4::Label::new(None);
    status.set_xalign(0.0);
    status.set_wrap(true);
    status.set_hexpand(true);
    status.set_max_width_chars(60);
    let verwerfen = gtk4::Button::with_label(&gettext("Discard Selection"));
    let uebernehmen = gtk4::Button::with_label(&gettext("Apply"));
    uebernehmen.add_css_class("suggested-action");
    let fuss = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    fuss.set_margin_start(14);
    fuss.set_margin_end(14);
    fuss.set_margin_top(8);
    fuss.set_margin_bottom(12);
    fuss.append(&status);
    fuss.append(&verwerfen);
    fuss.append(&uebernehmen);

    let s = Rc::new(Seite {
        vorschau,
        name,
        darstellung,
        galerie,
        kacheln,
        bildschirm,
        status,
        verwerfen,
        uebernehmen,
        geladen: RefCell::new(Hintergrund::default()),
        geladener_text: RefCell::new(String::new()),
        gewaehlt: RefCell::new(None),
        faehigkeiten: Cell::new(None),
        stumm: Cell::new(false),
        monitor: RefCell::new(None),
    });
    s.laden();
    {
        let s2 = s.clone();
        s.galerie.connect_child_activated(move |_, kind| {
            if s2.stumm.get() {
                return;
            }
            if let Some(&(wahl, _)) = s2.kacheln.iter().find(|(_, k)| k == kind) {
                s2.waehlen(wahl);
            }
        });
        let s2 = s.clone();
        s.uebernehmen.connect_clicked(move |_| s2.uebernehmen());
        let s2 = s.clone();
        s.verwerfen.connect_clicked(move |_| s2.laden());
    }
    {
        // wie im Dialog: Fähigkeiten des laufenden Desktops im Hintergrund
        let s2 = s.clone();
        let auftrag = gio::spawn_blocking(faehigkeiten::ermitteln);
        glib::spawn_future_local(async move {
            s2.faehigkeiten
                .set(Some(auftrag.await.unwrap_or(Faehigkeiten::HMP10A)));
            s2.knoepfe();
        });
    }
    // Änderungen von außen (Dialog, Desktop) anzeigen
    if let Ok(m) = gio::File::for_path(hg::konfig_pfad())
        .monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
    {
        let s2 = s.clone();
        m.connect_changed(move |_, _, _, ereignis| {
            if matches!(
                ereignis,
                gio::FileMonitorEvent::ChangesDoneHint
                    | gio::FileMonitorEvent::Created
                    | gio::FileMonitorEvent::Deleted
                    | gio::FileMonitorEvent::Renamed
                    | gio::FileMonitorEvent::MovedIn
            ) && s2.gewaehlt.borrow().is_none()
            {
                s2.laden();
            }
        });
        *s.monitor.borrow_mut() = Some(m);
    }

    let rollbar = gtk4::ScrolledWindow::new();
    rollbar.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    rollbar.set_min_content_height(160);
    rollbar.set_vexpand(true);
    rollbar.set_child(Some(&aussen));
    let ganz = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    ganz.append(&rollbar);
    ganz.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    ganz.append(&fuss);
    ganz.upcast()
}

impl Seite {
    fn laden(&self) {
        let text = std::fs::read_to_string(hg::konfig_pfad()).unwrap_or_default();
        let (h, probleme) = Hintergrund::lesen(&text);
        for p in &probleme {
            eprintln!("querencia-einstellungen: desktop.ini: {p}");
        }
        *self.geladen.borrow_mut() = h;
        *self.geladener_text.borrow_mut() = text;
        *self.gewaehlt.borrow_mut() = None;
        self.anzeigen();
    }

    /// Vorschau, Name, Kachel und Knöpfe für die gewählte (sonst die geladene) Einstellung.
    fn anzeigen(&self) {
        let h = self
            .gewaehlt
            .borrow()
            .clone()
            .unwrap_or_else(|| self.geladen.borrow().clone());
        let (textur, fehler) = hg::textur_fuer(&h, self.bildschirm);
        self.vorschau.zeigen(&h, textur.as_ref());
        self.name.set_text(&beschreibung(&self.geladen.borrow()));
        self.darstellung.set_text(&gettext_f(
            "Style: {style}",
            &[("style", &self.geladen.borrow().darstellung.beschriftung())],
        ));
        self.stumm.set(true);
        match (h.typ == Typ::Bild)
            .then(|| hg::erkannt(h.datei.as_deref()))
            .flatten()
            .and_then(|w| self.kacheln.iter().find(|(k, _)| *k == w))
        {
            Some((_, kind)) => self.galerie.select_child(kind),
            None => self.galerie.unselect_all(),
        }
        self.stumm.set(false);
        self.knoepfe();
        if let Some(f) = fehler {
            self.status.set_text(&f);
        }
    }

    fn waehlen(&self, wahl: Wahl) {
        let mut neu = self.geladen.borrow().clone();
        neu.typ = Typ::Bild;
        neu.datei = hg::eingebaut_datei(wahl, self.bildschirm);
        let gleich = neu == *self.geladen.borrow();
        *self.gewaehlt.borrow_mut() = (!gleich).then_some(neu);
        self.anzeigen();
    }

    fn knoepfe(&self) {
        self.verwerfen
            .set_sensitive(self.gewaehlt.borrow().is_some());
        let Some(neu) = self.gewaehlt.borrow().clone() else {
            self.uebernehmen.set_sensitive(false);
            self.status
                .set_text(&gettext("Choose an image from the gallery and “Apply”."));
            return;
        };
        let name = beschreibung(&neu);
        match (
            self.faehigkeiten.get(),
            neu.tauglich(&self.faehigkeiten.get().unwrap_or(Faehigkeiten::HMP10A)),
        ) {
            (_, Ok(())) => {
                self.uebernehmen.set_sensitive(true);
                self.status.set_text(&gettext_f(
                    "Selected: {name} – not applied yet.",
                    &[("name", &name)],
                ));
            }
            (None, Err(_)) => {
                self.uebernehmen.set_sensitive(false);
                self.status.set_text(&gettext(
                    "Determining the capabilities of the running desktop …",
                ));
            }
            (Some(_), Err(grund)) => {
                self.uebernehmen.set_sensitive(false);
                self.status.set_text(&gettext_f(
                    "The running desktop cannot display this: {reason}",
                    &[("reason", &grund)],
                ));
            }
        }
    }

    fn uebernehmen(&self) {
        let Some(neu) = self.gewaehlt.borrow().clone() else {
            return;
        };
        if neu
            .tauglich(&self.faehigkeiten.get().unwrap_or(Faehigkeiten::HMP10A))
            .is_err()
        {
            return;
        }
        let pfad = hg::konfig_pfad();
        let jetzt = std::fs::read_to_string(&pfad).unwrap_or_default();
        if ini::gruppe(&jetzt, GRUPPE) != ini::gruppe(&self.geladener_text.borrow(), GRUPPE) {
            self.status.set_text(&gettext("desktop.ini has been changed elsewhere in the meantime – not overwritten. “Discard Selection” shows the current state."));
            return;
        }
        match hg::schreiben(&pfad, &neu, &jetzt) {
            Ok(sicherung) => {
                let zeit = glib::DateTime::now_local()
                    .ok()
                    .and_then(|d| d.format("%H:%M:%S").ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                eprintln!(
                    "querencia-einstellungen: Hintergrundbild übernommen: {}",
                    beschreibung(&neu)
                );
                self.laden();
                self.status.set_text(&match sicherung {
                    Some(s) => gettext_f(
                        "Applied at {time}; the desktop shows the image immediately. Previous setting saved as {name}.",
                        &[
                            ("time", &zeit),
                            (
                                "name",
                                &s.file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_default(),
                            ),
                        ],
                    ),
                    None => gettext_f(
                        "Applied at {time}; the desktop shows the image immediately.",
                        &[("time", &zeit)],
                    ),
                });
            }
            Err(e) => self.status.set_text(&gettext_f(
                "Not saved: {error}",
                &[("error", &e.to_string())],
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beschreibung_farbe_und_standard() {
        let mut h = Hintergrund::default();
        // leeres `File=`: das eingebaute Bild (Tiefblau, sonst Blau)
        assert!(beschreibung(&h).contains(&hg::eingebaut_wahl().name()));
        h.typ = Typ::Farbe;
        assert_eq!(beschreibung(&h), format!("Solid color {}", h.farbe.hex()));
    }
}
