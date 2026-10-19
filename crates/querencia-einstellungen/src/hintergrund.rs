//! Dialog „Querencia – Hintergrund“ (QE-01): Vorschau, Hintergrundtyp,
//! Bild, Farbe, Darstellung, Schaltflächen. Schreibt erst bei „Übernehmen“
//! nach `~/.config/querencia/desktop.ini` (atomar, mit Sicherung der
//! vorherigen Datei); die Vorschau ändert den echten Desktop nie.
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};

use crate::faehigkeiten;
use crate::modell::{self, Darstellung, Faehigkeiten, Farbe, Hintergrund, Typ};
use crate::vorschau::Vorschau;

pub const TITEL: &str = "Querencia – Hintergrund";

/// `~/.config/querencia/desktop.ini` – derselbe Pfad wie im Desktop
/// (`glib::user_config_dir()`, also `XDG_CONFIG_HOME`).
pub fn konfig_pfad() -> PathBuf {
    glib::user_config_dir()
        .join("querencia")
        .join("desktop.ini")
}

/// Das eingebaute Bild, gesucht wie im Desktop: `querencia/hintergruende/`
/// in den XDG-Datenordnern.
pub fn eingebautes_bild() -> Option<PathBuf> {
    std::iter::once(glib::user_data_dir())
        .chain(glib::system_data_dirs())
        .map(|d| {
            d.join("querencia")
                .join("hintergruende")
                .join(modell::EINGEBAUT)
        })
        .find(|p| p.is_file())
}

struct Zustand {
    aktuell: Hintergrund,
    /// Stand der Datei beim Öffnen (Abbrechen kehrt dorthin zurück; „Bestehende
    /// Einstellungen dürfen nicht unbemerkt überschrieben werden“).
    geladen: Hintergrund,
    geladener_text: String,
    textur: Option<gdk::Texture>,
    bildfehler: Option<String>,
}

struct Oberflaeche {
    fenster: gtk4::Window,
    zustand: RefCell<Zustand>,
    stumm: Cell<bool>,
    vorschau: Vorschau,
    typ_bild: gtk4::CheckButton,
    typ_farbe: gtk4::CheckButton,
    bild_name: gtk4::Label,
    bild_hinweis: gtk4::Label,
    darstellung: gtk4::DropDown,
    hex: gtk4::Entry,
    hex_hinweis: gtk4::Label,
    farbknopf: gtk4::ColorDialogButton,
    farbfeld: gtk4::DrawingArea,
    status: gtk4::Label,
    uebernehmen: gtk4::Button,
    bildzeile: gtk4::Box,
    darstellungszeile: gtk4::Box,
    /// Was der laufende Desktop kann; bis die Abfrage im Hintergrund fertig ist
    /// `None` (dann bleibt alles gesperrt, was HMP-10a nicht kann).
    faehigkeiten: Cell<Option<Faehigkeiten>>,
}

/// Öffnet den Dialog als Fenster der Anwendung.
pub fn oeffnen(app: &gtk4::Application) -> gtk4::Window {
    let (geladen, geladener_text, probleme) = laden();
    for p in &probleme {
        eprintln!("querencia-einstellungen: desktop.ini: {p}");
    }
    // Höhe an den Bildschirm anpassen (1024×768: Panels und Titelleiste abziehen);
    // der Inhalt rollt, falls er trotzdem nicht passt.
    let hoehe = bildschirmhoehe().map_or(760, |h| (h - 130).clamp(480, 760));
    let fenster = gtk4::Window::builder()
        .application(app)
        .title(TITEL)
        .default_width(720)
        .default_height(hoehe)
        .build();

    let vorschau = Vorschau::default();
    vorschau.set_size_request(400, if hoehe < 700 { 180 } else { 240 });
    vorschau.set_hexpand(true);
    vorschau.set_vexpand(true);

    // --- Vorschau
    let vorschau_rahmen = gtk4::Frame::new(Some("Vorschau"));
    let vorschau_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    vorschau_box.set_margin_top(6);
    vorschau_box.set_margin_bottom(6);
    vorschau_box.set_margin_start(6);
    vorschau_box.set_margin_end(6);
    vorschau_box.append(&vorschau);
    let aufloesung_zeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    aufloesung_zeile.append(&gtk4::Label::new(Some("Vorschau für:")));
    let aufloesungen = aufloesungen();
    let namen: Vec<String> = aufloesungen.iter().map(|(n, _, _)| n.clone()).collect();
    let namen_ref: Vec<&str> = namen.iter().map(String::as_str).collect();
    let aufloesung = gtk4::DropDown::from_strings(&namen_ref);
    aufloesung_zeile.append(&aufloesung);
    vorschau_box.append(&aufloesung_zeile);
    vorschau_rahmen.set_child(Some(&vorschau_box));
    if let Some((_, b, h)) = aufloesungen.first() {
        vorschau.set_flaeche(*b, *h);
    }

    // --- Hintergrundtyp
    let typ_rahmen = gtk4::Frame::new(Some("Hintergrundtyp"));
    let typ_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 18);
    rand(&typ_box);
    let typ_bild = gtk4::CheckButton::with_label("Hintergrundbild");
    let typ_farbe = gtk4::CheckButton::with_label("Einfarbige Hintergrundfarbe");
    typ_farbe.set_group(Some(&typ_bild));
    typ_box.append(&typ_bild);
    typ_box.append(&typ_farbe);
    typ_rahmen.set_child(Some(&typ_box));

    // --- Hintergrundbild
    let bild_rahmen = gtk4::Frame::new(Some("Hintergrundbild"));
    let bild_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    rand(&bild_box);
    let bildzeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let bild_name = gtk4::Label::new(None);
    bild_name.set_hexpand(true);
    bild_name.set_xalign(0.0);
    bild_name.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
    let bild_waehlen = gtk4::Button::with_label("Bild auswählen …");
    let bild_standard = gtk4::Button::with_label("Eingebautes Bild");
    bildzeile.append(&bild_name);
    bildzeile.append(&bild_waehlen);
    bildzeile.append(&bild_standard);
    bild_box.append(&bildzeile);
    let darstellungszeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    darstellungszeile.append(&gtk4::Label::new(Some("Darstellung:")));
    let beschriftungen: Vec<&str> = Darstellung::ALLE.iter().map(|d| d.beschriftung()).collect();
    let darstellung = gtk4::DropDown::from_strings(&beschriftungen);
    darstellungszeile.append(&darstellung);
    bild_box.append(&darstellungszeile);
    let bild_hinweis = gtk4::Label::new(None);
    bild_hinweis.set_xalign(0.0);
    bild_hinweis.set_wrap(true);
    bild_hinweis.add_css_class("dim-label");
    bild_box.append(&bild_hinweis);
    bild_rahmen.set_child(Some(&bild_box));

    // --- Hintergrundfarbe
    let farb_rahmen = gtk4::Frame::new(Some("Hintergrundfarbe"));
    let farb_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    rand(&farb_box);
    let palette = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    let farbfeld = gtk4::DrawingArea::new();
    farbfeld.set_content_width(72);
    farbfeld.set_content_height(28);
    let hex = gtk4::Entry::new();
    hex.set_max_length(7);
    hex.set_width_chars(9);
    hex.set_placeholder_text(Some("#rrggbb"));
    let hex_hinweis = gtk4::Label::new(None);
    hex_hinweis.set_xalign(0.0);
    hex_hinweis.add_css_class("dim-label");
    let farbknopf = gtk4::ColorDialogButton::new(Some(gtk4::ColorDialog::new()));
    let farbzeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    farbzeile.append(&gtk4::Label::new(Some("Palette:")));
    farbzeile.append(&palette);
    farb_box.append(&farbzeile);
    let eigene = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    eigene.append(&gtk4::Label::new(Some("Farbwert:")));
    eigene.append(&hex);
    eigene.append(&gtk4::Label::new(Some("Benutzerdefiniert:")));
    eigene.append(&farbknopf);
    eigene.append(&gtk4::Label::new(Some("Vorschau:")));
    eigene.append(&farbfeld);
    farb_box.append(&eigene);
    farb_box.append(&hex_hinweis);
    farb_rahmen.set_child(Some(&farb_box));

    // --- Schaltflächen
    let status = gtk4::Label::new(None);
    status.set_xalign(0.0);
    status.set_hexpand(true);
    status.set_wrap(true);
    let knopfzeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let abbrechen = gtk4::Button::with_label("Abbrechen");
    let standard = gtk4::Button::with_label("Standard wiederherstellen");
    let uebernehmen = gtk4::Button::with_label("Übernehmen");
    uebernehmen.add_css_class("suggested-action");
    let luecke = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    luecke.set_hexpand(true);
    knopfzeile.append(&standard);
    knopfzeile.append(&luecke);
    knopfzeile.append(&abbrechen);
    knopfzeile.append(&uebernehmen);

    let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    inhalt.set_margin_top(10);
    inhalt.set_margin_bottom(10);
    inhalt.set_margin_start(10);
    inhalt.set_margin_end(10);
    inhalt.append(&vorschau_rahmen);
    inhalt.append(&typ_rahmen);
    inhalt.append(&bild_rahmen);
    inhalt.append(&farb_rahmen);
    let rollbar = gtk4::ScrolledWindow::new();
    rollbar.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    rollbar.set_vexpand(true);
    rollbar.set_child(Some(&inhalt));
    let unten = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    unten.set_margin_start(10);
    unten.set_margin_end(10);
    unten.set_margin_bottom(10);
    unten.append(&status);
    unten.append(&knopfzeile);
    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    aussen.append(&rollbar);
    aussen.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    aussen.append(&unten);
    fenster.set_child(Some(&aussen));

    let textur = textur_fuer(&geladen);
    let ui = Rc::new(Oberflaeche {
        fenster: fenster.clone(),
        zustand: RefCell::new(Zustand {
            aktuell: geladen.clone(),
            geladen,
            geladener_text,
            textur: textur.0,
            bildfehler: textur.1,
        }),
        stumm: Cell::new(false),
        vorschau,
        typ_bild,
        typ_farbe,
        bild_name,
        bild_hinweis,
        darstellung,
        hex,
        hex_hinweis,
        farbknopf,
        farbfeld,
        status,
        uebernehmen,
        bildzeile,
        darstellungszeile,
        faehigkeiten: Cell::new(None),
    });

    // Fähigkeiten des laufenden Desktops im Hintergrund ermitteln: das Fenster
    // erscheint sofort; bis zum Ergebnis gilt der Stand von HMP-10a.
    {
        let ui2 = ui.clone();
        let auftrag = gio::spawn_blocking(faehigkeiten::ermitteln);
        glib::spawn_future_local(async move {
            let f = auftrag.await.unwrap_or(Faehigkeiten::HMP10A);
            ui2.faehigkeiten.set(Some(f));
            ui2.uebernehmen_pruefen();
        });
    }

    // Palette
    for (name, farbe) in modell::PALETTE {
        let knopf = gtk4::Button::new();
        knopf.set_tooltip_text(Some(&format!("{name} ({})", farbe.hex())));
        let feld = gtk4::DrawingArea::new();
        feld.set_content_width(26);
        feld.set_content_height(18);
        let f = *farbe;
        feld.set_draw_func(move |_, cr, w, h| {
            let (r, g, b) = f.anteile();
            cr.set_source_rgb(f64::from(r), f64::from(g), f64::from(b));
            cr.rectangle(0.0, 0.0, f64::from(w), f64::from(h));
            let _ = cr.fill();
        });
        knopf.set_child(Some(&feld));
        let ui2 = ui.clone();
        knopf.connect_clicked(move |_| {
            ui2.zustand.borrow_mut().aktuell.farbe = f;
            ui2.aktualisieren();
        });
        palette.append(&knopf);
    }

    // Farbfeld zeichnet die aktuelle Farbe
    {
        let ui2 = ui.clone();
        ui.farbfeld.set_draw_func(move |_, cr, w, h| {
            let (r, g, b) = ui2.zustand.borrow().aktuell.farbe.anteile();
            cr.set_source_rgb(f64::from(r), f64::from(g), f64::from(b));
            cr.rectangle(0.0, 0.0, f64::from(w), f64::from(h));
            let _ = cr.fill();
        });
    }

    // Typ
    {
        let ui2 = ui.clone();
        ui.typ_bild.connect_toggled(move |b| {
            if ui2.stumm.get() || !b.is_active() {
                return;
            }
            ui2.zustand.borrow_mut().aktuell.typ = Typ::Bild;
            ui2.aktualisieren();
        });
        let ui2 = ui.clone();
        ui.typ_farbe.connect_toggled(move |b| {
            if ui2.stumm.get() || !b.is_active() {
                return;
            }
            ui2.zustand.borrow_mut().aktuell.typ = Typ::Farbe;
            ui2.aktualisieren();
        });
    }

    // Vorschau-Auflösung
    {
        let ui2 = ui.clone();
        aufloesung.connect_selected_notify(move |dd| {
            if let Some((_, b, h)) = aufloesungen.get(dd.selected() as usize) {
                ui2.vorschau.set_flaeche(*b, *h);
            }
        });
    }

    // Bild wählen
    {
        let ui2 = ui.clone();
        bild_waehlen.connect_clicked(move |_| ui2.bild_waehlen());
        let ui2 = ui.clone();
        bild_standard.connect_clicked(move |_| {
            ui2.zustand.borrow_mut().aktuell.datei = None;
            ui2.bild_laden();
            ui2.aktualisieren();
        });
    }

    // Darstellung
    {
        let ui2 = ui.clone();
        ui.darstellung.connect_selected_notify(move |dd| {
            if ui2.stumm.get() {
                return;
            }
            if let Some(d) = Darstellung::ALLE.get(dd.selected() as usize) {
                ui2.zustand.borrow_mut().aktuell.darstellung = *d;
                ui2.aktualisieren();
            }
        });
    }

    // HEX-Eingabe
    {
        let ui2 = ui.clone();
        ui.hex.connect_changed(move |e| {
            if ui2.stumm.get() {
                return;
            }
            match Farbe::aus_hex(&e.text()) {
                Some(f) => {
                    e.remove_css_class("error");
                    ui2.hex_hinweis.set_text("");
                    ui2.zustand.borrow_mut().aktuell.farbe = f;
                    ui2.aktualisieren_ohne_hex();
                }
                None => {
                    e.add_css_class("error");
                    ui2.hex_hinweis.set_text(
                        "Ungültiger Farbwert – erwartet wird #rrggbb, zum Beispiel #204a87.",
                    );
                }
            }
        });
    }

    // Benutzerdefinierte Farbe
    {
        let ui2 = ui.clone();
        ui.farbknopf.connect_rgba_notify(move |k| {
            if ui2.stumm.get() {
                return;
            }
            let c = k.rgba();
            let f = Farbe::neu(
                (c.red() * 255.0).round() as u8,
                (c.green() * 255.0).round() as u8,
                (c.blue() * 255.0).round() as u8,
            );
            ui2.zustand.borrow_mut().aktuell.farbe = f;
            ui2.aktualisieren();
        });
    }

    // Schaltflächen
    {
        let ui2 = ui.clone();
        abbrechen.connect_clicked(move |_| {
            // Verwirft alles, was nicht übernommen wurde.
            let geladen = ui2.zustand.borrow().geladen.clone();
            ui2.zustand.borrow_mut().aktuell = geladen;
            ui2.fenster.close();
        });
        let ui2 = ui.clone();
        standard.connect_clicked(move |_| {
            ui2.zustand.borrow_mut().aktuell = Hintergrund::default();
            ui2.bild_laden();
            ui2.aktualisieren();
            ui2.status.set_text(
                "Standard in der Vorschau wiederhergestellt – gilt erst nach „Übernehmen“.",
            );
        });
        let ui2 = ui.clone();
        ui.uebernehmen.connect_clicked(move |_| ui2.uebernehmen());
    }

    ui.aktualisieren();
    if !probleme.is_empty() {
        ui.status.set_text(&format!(
            "Hinweis: {} Eintrag/Einträge in desktop.ini wurden ignoriert (siehe Protokoll).",
            probleme.len()
        ));
    }
    fenster
}

fn rand(b: &gtk4::Box) {
    b.set_margin_top(6);
    b.set_margin_bottom(6);
    b.set_margin_start(8);
    b.set_margin_end(8);
}

pub fn bildschirmhoehe() -> Option<i32> {
    let display = gdk::Display::default()?;
    let m = display
        .monitors()
        .iter::<gdk::Monitor>()
        .find_map(Result::ok)?;
    Some(m.geometry().height())
}

/// Vorschau-Auflösungen: aktueller Bildschirm zuerst, dann die beiden Prüfgrößen.
fn aufloesungen() -> Vec<(String, f64, f64)> {
    let mut liste = Vec::new();
    if let Some(display) = gdk::Display::default()
        && let Some(m) = display
            .monitors()
            .iter::<gdk::Monitor>()
            .find_map(Result::ok)
    {
        let g = m.geometry();
        let (b, h) = (f64::from(g.width()), f64::from(g.height()));
        liste.push((format!("Aktueller Bildschirm ({b}×{h})"), b, h));
    }
    for (b, h) in [(1920.0, 1080.0), (1024.0, 768.0)] {
        if !liste.iter().any(|(_, x, y)| *x == b && *y == h) {
            liste.push((format!("{b}×{h}"), b, h));
        }
    }
    liste
}

fn laden() -> (Hintergrund, String, Vec<String>) {
    let text = std::fs::read_to_string(konfig_pfad()).unwrap_or_default();
    let (h, probleme) = Hintergrund::lesen(&text);
    (h, text, probleme)
}

/// Textur für die Einstellung: die Datei, sonst das eingebaute Bild. Der
/// zweite Wert ist eine Fehlermeldung für ein unlesbares oder fehlendes Bild.
fn textur_fuer(h: &Hintergrund) -> (Option<gdk::Texture>, Option<String>) {
    match &h.datei {
        Some(pfad) => match gdk::Texture::from_filename(pfad) {
            Ok(t) => (Some(t), None),
            Err(err) => (
                eingebautes_bild().and_then(|p| gdk::Texture::from_filename(p).ok()),
                Some(format!(
                    "Bild „{}“ nicht lesbar ({}). Der Desktop zeigt dann das eingebaute Bild.",
                    pfad.display(),
                    kurz(&err.to_string())
                )),
            ),
        },
        None => match eingebautes_bild() {
            Some(p) => (gdk::Texture::from_filename(p).ok(), None),
            None => (
                None,
                Some("Eingebautes Bild nicht gefunden – nur Farbe.".into()),
            ),
        },
    }
}

fn kurz(s: &str) -> String {
    let s = s.lines().next().unwrap_or(s);
    if s.len() > 90 {
        format!(
            "{}…",
            &s[..s.char_indices().nth(90).map_or(s.len(), |(i, _)| i)]
        )
    } else {
        s.to_string()
    }
}

impl Oberflaeche {
    fn bild_laden(&self) {
        let (t, fehler) = textur_fuer(&self.zustand.borrow().aktuell);
        let mut z = self.zustand.borrow_mut();
        z.textur = t;
        z.bildfehler = fehler;
    }

    fn bild_waehlen(self: &Rc<Self>) {
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("Bilder (PNG, JPEG, WebP, …)"));
        filter.add_mime_type("image/*");
        let filters = gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);
        let dialog = gtk4::FileDialog::builder()
            .title("Hintergrundbild auswählen")
            .modal(true)
            .filters(&filters)
            .default_filter(&filter)
            .build();
        if let Some(bilder) = glib::user_special_dir(glib::UserDirectory::Pictures) {
            dialog.set_initial_folder(Some(&gio::File::for_path(bilder)));
        }
        let ui = self.clone();
        dialog.open(
            Some(&self.fenster),
            gio::Cancellable::NONE,
            move |ergebnis| {
                let Ok(datei) = ergebnis else { return };
                let Some(pfad) = datei.path() else { return };
                {
                    let mut z = ui.zustand.borrow_mut();
                    z.aktuell.datei = Some(pfad);
                    z.aktuell.typ = Typ::Bild;
                }
                ui.bild_laden();
                ui.aktualisieren();
            },
        );
    }

    /// Widgets aus dem Modell setzen und die Vorschau zeichnen.
    fn aktualisieren(&self) {
        self.stumm.set(true);
        {
            let z = self.zustand.borrow();
            let a = &z.aktuell;
            self.typ_bild.set_active(a.typ == Typ::Bild);
            self.typ_farbe.set_active(a.typ == Typ::Farbe);
            self.bild_name.set_text(&match &a.datei {
                Some(p) => p.display().to_string(),
                None => format!("Eingebautes Bild „Querencia Blau“ ({})", modell::EINGEBAUT),
            });
            self.bild_hinweis
                .set_text(z.bildfehler.as_deref().unwrap_or(""));
            if let Some(i) = Darstellung::ALLE.iter().position(|d| *d == a.darstellung) {
                self.darstellung.set_selected(i as u32);
            }
            self.hex.set_text(&a.farbe.hex());
            self.hex.remove_css_class("error");
            self.hex_hinweis.set_text("");
            let (r, g, b) = a.farbe.anteile();
            self.farbknopf.set_rgba(&gdk::RGBA::new(r, g, b, 1.0));
            // Bildzeilen bei Farbe ausgegraut, nie versteckt (klassisch)
            let bild = a.typ == Typ::Bild;
            self.bildzeile.set_sensitive(bild);
            self.darstellungszeile.set_sensitive(bild);
            self.vorschau.zeigen(a, z.textur.as_ref());
        }
        self.farbfeld.queue_draw();
        self.stumm.set(false);
        self.uebernehmen_pruefen();
    }

    /// Wie `aktualisieren`, lässt aber das HEX-Feld in Ruhe (der Benutzer tippt gerade).
    fn aktualisieren_ohne_hex(&self) {
        self.stumm.set(true);
        {
            let z = self.zustand.borrow();
            let (r, g, b) = z.aktuell.farbe.anteile();
            self.farbknopf.set_rgba(&gdk::RGBA::new(r, g, b, 1.0));
            self.vorschau.zeigen(&z.aktuell, z.textur.as_ref());
        }
        self.farbfeld.queue_draw();
        self.stumm.set(false);
        self.uebernehmen_pruefen();
    }

    /// „Übernehmen“ nur, wenn der laufende Desktop (HMP-10a) die Einstellung
    /// darstellen kann – sonst ehrlich erklären statt scheinbar zu wirken.
    fn uebernehmen_pruefen(&self) {
        let z = self.zustand.borrow();
        match z
            .aktuell
            .tauglich(&self.faehigkeiten.get().unwrap_or(Faehigkeiten::HMP10A))
        {
            Ok(()) => {
                self.uebernehmen.set_sensitive(true);
                self.uebernehmen
                    .set_tooltip_text(Some("Schreibt desktop.ini; der Desktop übernimmt sofort."));
                if z.aktuell == z.geladen {
                    self.status
                        .set_text("Keine Änderung gegenüber der gespeicherten Einstellung.");
                } else {
                    self.status.set_text("Geändert – noch nicht übernommen.");
                }
            }
            Err(_) if self.faehigkeiten.get().is_none() => {
                self.uebernehmen.set_sensitive(false);
                self.uebernehmen
                    .set_tooltip_text(Some("Fähigkeiten des Desktops werden ermittelt …"));
                self.status
                    .set_text("Fähigkeiten des laufenden Desktops werden ermittelt …");
            }
            Err(grund) => {
                self.uebernehmen.set_sensitive(false);
                self.uebernehmen.set_tooltip_text(Some(&grund));
                self.status
                    .set_text(&format!("Noch nicht verfügbar: {grund}"));
            }
        }
    }

    fn uebernehmen(&self) {
        let pfad = konfig_pfad();
        let (neu, geladener_text) = {
            let z = self.zustand.borrow();
            (z.aktuell.clone(), z.geladener_text.clone())
        };
        if neu
            .tauglich(&self.faehigkeiten.get().unwrap_or(Faehigkeiten::HMP10A))
            .is_err()
        {
            return;
        }
        // Nicht unbemerkt überschreiben: Hat jemand die Datei seit dem Öffnen geändert?
        let jetzt = std::fs::read_to_string(&pfad).unwrap_or_default();
        if jetzt != geladener_text {
            self.status.set_text(
                "desktop.ini wurde inzwischen von anderer Stelle geändert – nicht überschrieben. \
                 Bitte den Dialog schließen und neu öffnen.",
            );
            return;
        }
        match schreiben(&pfad, &neu, &jetzt) {
            Ok(sicherung) => {
                let mut z = self.zustand.borrow_mut();
                z.geladen = neu.clone();
                z.geladener_text = neu.schreiben();
                drop(z);
                let zeit = glib::DateTime::now_local()
                    .ok()
                    .and_then(|d| d.format("%H:%M:%S").ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                self.status.set_text(&match sicherung {
                    Some(s) => format!(
                        "Übernommen um {zeit}. Vorherige Einstellung gesichert als {}.",
                        s.file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default()
                    ),
                    None => format!("Übernommen um {zeit}."),
                });
            }
            Err(err) => self.status.set_text(&format!("Nicht gespeichert: {err}")),
        }
    }
}

/// Schreibt die Einstellung atomar (temporäre Datei, `rename`); eine vorhandene
/// Datei wird vorher als `desktop.ini.vorher-<Zeit>` gesichert.
pub fn schreiben(pfad: &Path, neu: &Hintergrund, bisher: &str) -> std::io::Result<Option<PathBuf>> {
    if let Some(eltern) = pfad.parent() {
        std::fs::create_dir_all(eltern)?;
    }
    let mut sicherung = None;
    if !bisher.is_empty() && pfad.exists() {
        let zeit = glib::DateTime::now_local()
            .ok()
            .and_then(|d| d.format("%Y%m%d-%H%M%S").ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "0".into());
        let ziel = pfad.with_file_name(format!("desktop.ini.vorher-{zeit}"));
        if !ziel.exists() {
            std::fs::copy(pfad, &ziel)?;
            sicherung = Some(ziel);
        }
    }
    let temporaer = pfad.with_extension("ini.neu");
    std::fs::write(&temporaer, neu.schreiben())?;
    std::fs::rename(&temporaer, pfad)?;
    Ok(sicherung)
}
