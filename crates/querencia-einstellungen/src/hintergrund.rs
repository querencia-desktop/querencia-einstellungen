//! Dialog „Querencia – Hintergrund“ (QE-01): Vorschau, Hintergrundtyp,
//! Bild, Farbe, Darstellung, Schaltflächen. Schreibt erst bei „Übernehmen“
//! nach `~/.config/querencia/desktop.ini` (atomar, mit Sicherung der
//! vorherigen Datei); die Vorschau ändert den echten Desktop nie.
//! G2-X: ersetzt nur die Gruppe `[Background]`; `[Icons]`, `[Volumes]` und
//! alles andere bleiben byte-gleich (`ini.rs`).
//! G2-AF: Galerie der eingebauten Bilder (`hintergrund_katalog.rs`); gewählt
//! wird die 4:3- oder 16:9-Fassung passend zum Bildschirm, die Vorschau zeigt
//! die Fassung für die gewählte Vorschaugröße.
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};

use crate::faehigkeiten;
use crate::hintergrund_katalog::{self as katalog, Format, Wahl};
use crate::i18n::{gettext, gettext_f, ngettext_f};
use crate::ini;
use crate::modell::{self, Darstellung, Faehigkeiten, Farbe, Hintergrund, Typ};
use crate::vorschau::Vorschau;

/// Fenstertitel des Dialogs (`--hintergrund`).
pub fn titel() -> String {
    gettext("Querencia – Background")
}

/// `~/.config/querencia/desktop.ini` – derselbe Pfad wie im Desktop
/// (`glib::user_config_dir()`, also `XDG_CONFIG_HOME`).
pub fn konfig_pfad() -> PathBuf {
    glib::user_config_dir()
        .join("querencia")
        .join("desktop.ini")
}

/// Ordner der eingebauten Bilder wie im Desktop: `querencia/hintergruende/`
/// in den XDG-Datenordnern, der des Benutzers zuerst.
pub fn eingebaute_ordner() -> Vec<PathBuf> {
    std::iter::once(glib::user_data_dir())
        .chain(glib::system_data_dirs())
        .map(|d| d.join("querencia").join("hintergruende"))
        .collect()
}

/// Das eingebaute Bild „Querencia Blau“ (der Standard).
pub fn eingebautes_bild() -> Option<PathBuf> {
    eingebaute_ordner()
        .into_iter()
        .map(|o| o.join(modell::EINGEBAUT))
        .find(|p| p.is_file())
}

/// G2-AF: Datei einer Galerie-Kachel für ein Format; „Querencia Blau“ ist
/// `None` (leeres `File=`, wie bisher der Standard).
pub(crate) fn eingebaut_datei(wahl: Wahl, format: Format) -> Option<PathBuf> {
    if wahl == Wahl::STANDARD {
        return None;
    }
    wahl.finden(format, &eingebaute_ordner(), Path::is_file)
}

/// Kachel zur Einstellung: leeres `File=` ist „Querencia Blau“.
pub(crate) fn erkannt(datei: Option<&Path>) -> Option<Wahl> {
    match datei {
        None => Some(Wahl::STANDARD),
        Some(p) => katalog::erkennen(p),
    }
}

/// Format des (ersten) Bildschirms; ohne Bildschirm 16:9.
pub(crate) fn bildschirm_format() -> Format {
    gdk::Display::default()
        .and_then(|d| d.monitors().iter::<gdk::Monitor>().find_map(Result::ok))
        .map_or(Format::Breit, |m| {
            let g = m.geometry();
            Format::fuer(f64::from(g.width()), f64::from(g.height()))
        })
}

struct Zustand {
    aktuell: Hintergrund,
    /// Stand der Datei beim Öffnen (Abbrechen kehrt dorthin zurück; „Bestehende
    /// Einstellungen dürfen nicht unbemerkt überschrieben werden“).
    geladen: Hintergrund,
    geladener_text: String,
    textur: Option<gdk::Texture>,
    bildfehler: Option<String>,
    /// G2-AF: Format der gewählten Vorschaugröße (Fassung in der Vorschau).
    vorschau_format: Format,
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
    /// G2-AF: Galerie der eingebauten Bilder und ihre Kacheln.
    galerie: gtk4::FlowBox,
    kacheln: Vec<(Wahl, gtk4::FlowBoxChild)>,
    /// Format dieses Bildschirms: diese Fassung wird geschrieben.
    bildschirm: Format,
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
        .title(titel())
        .default_width(720)
        .default_height(hoehe)
        .build();

    let vorschau = Vorschau::default();
    vorschau.set_size_request(400, if hoehe < 700 { 180 } else { 240 });
    vorschau.set_hexpand(true);
    vorschau.set_vexpand(true);

    // --- Vorschau
    let vorschau_rahmen = gtk4::Frame::new(Some(&gettext("Preview")));
    let vorschau_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    vorschau_box.set_margin_top(6);
    vorschau_box.set_margin_bottom(6);
    vorschau_box.set_margin_start(6);
    vorschau_box.set_margin_end(6);
    vorschau_box.append(&vorschau);
    let aufloesung_zeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    aufloesung_zeile.append(&gtk4::Label::new(Some(&gettext("Preview for:"))));
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
    let typ_rahmen = gtk4::Frame::new(Some(&gettext("Background Type")));
    let typ_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 18);
    rand(&typ_box);
    let typ_bild = gtk4::CheckButton::with_label(&gettext("Background image"));
    let typ_farbe = gtk4::CheckButton::with_label(&gettext("Solid background color"));
    typ_farbe.set_group(Some(&typ_bild));
    typ_box.append(&typ_bild);
    typ_box.append(&typ_farbe);
    typ_rahmen.set_child(Some(&typ_box));

    // --- Hintergrundbild
    let bild_rahmen = gtk4::Frame::new(Some(&gettext("Background image")));
    let bild_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    rand(&bild_box);
    let bildschirm = bildschirm_format();
    let (galerie, kacheln) = galerie_bauen(bildschirm);
    let galerie_titel = gtk4::Label::new(Some(&gettext("Built-in images:")));
    galerie_titel.set_xalign(0.0);
    bild_box.append(&galerie_titel);
    bild_box.append(&galerie);
    let galerie_hinweis = gtk4::Label::new(Some(&gettext_f(
        "The {format} version is used for this screen.",
        &[("format", bildschirm.beschriftung())],
    )));
    galerie_hinweis.set_xalign(0.0);
    galerie_hinweis.add_css_class("dim-label");
    bild_box.append(&galerie_hinweis);
    let bildzeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let bild_name = gtk4::Label::new(None);
    bild_name.set_hexpand(true);
    bild_name.set_xalign(0.0);
    bild_name.set_ellipsize(gtk4::pango::EllipsizeMode::Middle);
    let bild_waehlen = gtk4::Button::with_label(&gettext("Choose Image …"));
    let bild_standard = gtk4::Button::with_label(&gettext("Built-in Image"));
    bildzeile.append(&bild_name);
    bildzeile.append(&bild_waehlen);
    bildzeile.append(&bild_standard);
    bild_box.append(&bildzeile);
    let darstellungszeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    darstellungszeile.append(&gtk4::Label::new(Some(&gettext("Style:"))));
    let beschriftungen: Vec<String> = Darstellung::ALLE.iter().map(|d| d.beschriftung()).collect();
    let darstellung = gtk4::DropDown::from_strings(
        &beschriftungen
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    darstellungszeile.append(&darstellung);
    bild_box.append(&darstellungszeile);
    let bild_hinweis = gtk4::Label::new(None);
    bild_hinweis.set_xalign(0.0);
    bild_hinweis.set_wrap(true);
    bild_hinweis.add_css_class("dim-label");
    bild_box.append(&bild_hinweis);
    bild_rahmen.set_child(Some(&bild_box));

    // --- Hintergrundfarbe
    let farb_rahmen = gtk4::Frame::new(Some(&gettext("Background Color")));
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
    farbzeile.append(&gtk4::Label::new(Some(&gettext("Palette:"))));
    farbzeile.append(&palette);
    farb_box.append(&farbzeile);
    let eigene = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    eigene.append(&gtk4::Label::new(Some(&gettext("Color value:"))));
    eigene.append(&hex);
    eigene.append(&gtk4::Label::new(Some(&gettext("Custom:"))));
    eigene.append(&farbknopf);
    eigene.append(&gtk4::Label::new(Some(&gettext("Preview:"))));
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
    let abbrechen = gtk4::Button::with_label(&gettext("Cancel"));
    let standard = gtk4::Button::with_label(&gettext("Restore Defaults"));
    let uebernehmen = gtk4::Button::with_label(&gettext("Apply"));
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

    let vorschau_format = aufloesungen
        .first()
        .map_or(bildschirm, |(_, b, h)| Format::fuer(*b, *h));
    let textur = textur_fuer(&geladen, vorschau_format);
    let ui = Rc::new(Oberflaeche {
        fenster: fenster.clone(),
        zustand: RefCell::new(Zustand {
            aktuell: geladen.clone(),
            geladen,
            geladener_text,
            textur: textur.0,
            bildfehler: textur.1,
            vorschau_format,
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
        galerie,
        kacheln,
        bildschirm,
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
        knopf.set_tooltip_text(Some(&format!("{} ({})", gettext(*name), farbe.hex())));
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
                // G2-AF: eingebautes Bild in der Fassung für diese Größe zeigen
                let format = Format::fuer(*b, *h);
                if ui2.zustand.borrow().vorschau_format != format {
                    ui2.zustand.borrow_mut().vorschau_format = format;
                    ui2.bild_laden();
                    ui2.aktualisieren();
                }
            }
        });
    }

    // G2-AF: Galerie
    {
        let ui2 = ui.clone();
        ui.galerie.connect_child_activated(move |_, kind| {
            let Some(&(wahl, _)) = ui2.kacheln.iter().find(|(_, k)| k == kind) else {
                return;
            };
            {
                let mut z = ui2.zustand.borrow_mut();
                z.aktuell.datei = eingebaut_datei(wahl, ui2.bildschirm);
                z.aktuell.typ = Typ::Bild;
            }
            ui2.bild_laden();
            ui2.aktualisieren();
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
                    ui2.hex_hinweis.set_text(&gettext(
                        "Invalid color value – expected #rrggbb, for example #204a87.",
                    ));
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
            ui2.status.set_text(&gettext(
                "Default restored in the preview – takes effect only after “Apply”.",
            ));
        });
        let ui2 = ui.clone();
        ui.uebernehmen.connect_clicked(move |_| ui2.uebernehmen());
    }

    ui.aktualisieren();
    if !probleme.is_empty() {
        ui.status.set_text(&ngettext_f(
            "Note: {n} entry in desktop.ini was ignored (see the log).",
            "Note: {n} entries in desktop.ini were ignored (see the log).",
            probleme.len() as u32,
            &[],
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

/// Größe der Vorschaubilder in der Galerie (16:9).
const KACHEL: (i32, i32) = (112, 63);

/// G2-AF: Galerie mit je einer Kachel (Vorschaubild, Name) für jedes
/// installierte eingebaute Bild; die Vorschaubilder lädt ein Hintergrundfaden.
pub(crate) fn galerie_bauen(
    bildschirm: Format,
) -> (gtk4::FlowBox, Vec<(Wahl, gtk4::FlowBoxChild)>) {
    let galerie = gtk4::FlowBox::new();
    galerie.set_selection_mode(gtk4::SelectionMode::Single);
    galerie.set_activate_on_single_click(true);
    galerie.set_homogeneous(true);
    galerie.set_min_children_per_line(3);
    galerie.set_max_children_per_line(5);
    galerie.set_column_spacing(4);
    galerie.set_row_spacing(4);
    let mut kacheln = Vec::new();
    for wahl in katalog::kacheln() {
        let Some(bild) = wahl.finden(Format::Breit, &eingebaute_ordner(), Path::is_file) else {
            continue;
        };
        let feld = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        let vorschau = gtk4::Picture::new();
        vorschau.set_size_request(KACHEL.0, KACHEL.1);
        vorschau.set_halign(gtk4::Align::Center);
        vorschau.set_valign(gtk4::Align::Center);
        vorschau.set_content_fit(gtk4::ContentFit::Cover);
        // Kurzname in einer Zeile, damit alle Kacheln gleich hoch sind; der
        // volle Name steht im Tooltip.
        let name = gtk4::Label::new(Some(&wahl.kurzname()));
        name.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name.set_max_width_chars(16);
        feld.append(&vorschau);
        feld.append(&name);
        let kind = gtk4::FlowBoxChild::new();
        kind.set_child(Some(&feld));
        let datei = eingebaut_datei(wahl, bildschirm)
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| modell::EINGEBAUT.to_string());
        kind.set_tooltip_text(Some(&format!("{} – {datei}", wahl.name())));
        galerie.insert(&kind, -1);
        kacheln.push((wahl, kind));
        let auftrag = gio::spawn_blocking(move || vorschaubild(&bild));
        glib::spawn_future_local(async move {
            if let Ok(Some(textur)) = auftrag.await {
                vorschau.set_paintable(Some(&textur.textur()));
            }
        });
    }
    (galerie, kacheln)
}

/// Verkleinertes Bild als Rohdaten (im Hintergrundfaden erzeugt, `Send`).
struct Vorschaubild {
    breite: i32,
    hoehe: i32,
    zeile: usize,
    alpha: bool,
    daten: glib::Bytes,
}

impl Vorschaubild {
    fn textur(&self) -> gdk::Texture {
        let format = if self.alpha {
            gdk::MemoryFormat::R8g8b8a8
        } else {
            gdk::MemoryFormat::R8g8b8
        };
        gdk::MemoryTexture::new(self.breite, self.hoehe, format, &self.daten, self.zeile).upcast()
    }
}

fn vorschaubild(pfad: &Path) -> Option<Vorschaubild> {
    let pb = gtk4::gdk_pixbuf::Pixbuf::from_file_at_scale(pfad, KACHEL.0, KACHEL.1, true).ok()?;
    Some(Vorschaubild {
        breite: pb.width(),
        hoehe: pb.height(),
        zeile: usize::try_from(pb.rowstride()).ok()?,
        alpha: pb.has_alpha(),
        daten: pb.read_pixel_bytes(),
    })
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
        liste.push((
            gettext_f(
                "Current screen ({width}×{height})",
                &[("width", &b.to_string()), ("height", &h.to_string())],
            ),
            b,
            h,
        ));
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
/// G2-AF: Ein eingebautes Bild erscheint in der Fassung für `format`.
pub(crate) fn textur_fuer(
    h: &Hintergrund,
    format: Format,
) -> (Option<gdk::Texture>, Option<String>) {
    let fassung = h
        .datei
        .as_deref()
        .and_then(katalog::erkennen)
        .and_then(|w| eingebaut_datei(w, format));
    match fassung.as_ref().or(h.datei.as_ref()) {
        Some(pfad) => match gdk::Texture::from_filename(pfad) {
            Ok(t) => (Some(t), None),
            Err(err) => (
                eingebautes_bild().and_then(|p| gdk::Texture::from_filename(p).ok()),
                Some(gettext_f(
                    "Image “{file}” not readable ({error}). The desktop then shows the built-in image.",
                    &[
                        ("file", &pfad.display().to_string()),
                        ("error", &kurz(&err.to_string())),
                    ],
                )),
            ),
        },
        None => match eingebautes_bild() {
            Some(p) => (gdk::Texture::from_filename(p).ok(), None),
            None => (
                None,
                Some(gettext("Built-in image not found – color only.")),
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
        let (t, fehler) = {
            let z = self.zustand.borrow();
            textur_fuer(&z.aktuell, z.vorschau_format)
        };
        let mut z = self.zustand.borrow_mut();
        z.textur = t;
        z.bildfehler = fehler;
    }

    fn bild_waehlen(self: &Rc<Self>) {
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some(&gettext("Images (PNG, JPEG, WebP, …)")));
        filter.add_mime_type("image/*");
        let filters = gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);
        let dialog = gtk4::FileDialog::builder()
            .title(gettext("Choose Background Image"))
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
            self.bild_name
                .set_text(&match (&a.datei, erkannt(a.datei.as_deref())) {
                    (Some(p), Some(w)) => gettext_f(
                        "Built-in image “{name}” ({file})",
                        &[
                            ("name", &w.name()),
                            (
                                "file",
                                &p.file_name()
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_default(),
                            ),
                        ],
                    ),
                    (Some(p), None) => p.display().to_string(),
                    (None, _) => gettext_f(
                        "Built-in image “Querencia Blue” ({file})",
                        &[("file", modell::EINGEBAUT)],
                    ),
                });
            // G2-AF: Kachel der Einstellung markieren (fremdes Bild: keine)
            match erkannt(a.datei.as_deref())
                .and_then(|w| self.kacheln.iter().find(|(k, _)| *k == w))
            {
                Some((_, kind)) => self.galerie.select_child(kind),
                None => self.galerie.unselect_all(),
            }
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
                self.uebernehmen.set_tooltip_text(Some(&gettext(
                    "Writes desktop.ini; the desktop applies it immediately.",
                )));
                if z.aktuell == z.geladen {
                    self.status
                        .set_text(&gettext("No change compared to the saved setting."));
                } else {
                    self.status.set_text(&gettext("Changed – not applied yet."));
                }
            }
            Err(_) if self.faehigkeiten.get().is_none() => {
                self.uebernehmen.set_sensitive(false);
                self.uebernehmen
                    .set_tooltip_text(Some(&gettext("Determining the desktop’s capabilities …")));
                self.status.set_text(&gettext(
                    "Determining the capabilities of the running desktop …",
                ));
            }
            Err(grund) => {
                self.uebernehmen.set_sensitive(false);
                self.uebernehmen.set_tooltip_text(Some(&grund));
                self.status.set_text(&gettext_f(
                    "The running desktop cannot display this: {reason}",
                    &[("reason", &grund)],
                ));
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
        // Nicht unbemerkt überschreiben: Hat jemand den Hintergrund seit dem
        // Öffnen geändert? (G2-X: nur die eigene Gruppe zählt; geänderte
        // Arbeitsflächen-Einstellungen bleiben beim Schreiben einfach erhalten.)
        let jetzt = std::fs::read_to_string(&pfad).unwrap_or_default();
        if ini::gruppe(&jetzt, GRUPPE) != ini::gruppe(&geladener_text, GRUPPE) {
            self.status.set_text(&gettext(
                "desktop.ini has been changed elsewhere in the meantime – not overwritten. \
                 Please close the dialog and open it again.",
            ));
            return;
        }
        match schreiben(&pfad, &neu, &jetzt) {
            Ok(sicherung) => {
                let mut z = self.zustand.borrow_mut();
                z.geladen = neu.clone();
                z.geladener_text = ini::ersetzen(&jetzt, GRUPPE, &neu.schreiben());
                drop(z);
                let zeit = glib::DateTime::now_local()
                    .ok()
                    .and_then(|d| d.format("%H:%M:%S").ok())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                self.status.set_text(&match sicherung {
                    Some(s) => gettext_f(
                        "Applied at {time}. Previous setting saved as {name}.",
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
                    None => gettext_f("Applied at {time}.", &[("time", &zeit)]),
                });
            }
            Err(err) => self.status.set_text(&gettext_f(
                "Not saved: {error}",
                &[("error", &err.to_string())],
            )),
        }
    }
}

/// Die Gruppe dieses Dialogs in `desktop.ini`.
pub(crate) const GRUPPE: &str = "Background";

/// Schreibt die Einstellung atomar (temporäre Datei, `rename`); eine vorhandene
/// Datei wird vorher als `desktop.ini.vorher-<Zeit>` gesichert. G2-X: nur
/// `[Background]` wird ersetzt, der Rest von `bisher` bleibt byte-gleich; ohne
/// andere Gruppen entsteht genau der Text wie bisher.
pub fn schreiben(pfad: &Path, neu: &Hintergrund, bisher: &str) -> std::io::Result<Option<PathBuf>> {
    let zeit = glib::DateTime::now_local()
        .ok()
        .and_then(|d| d.format("%Y%m%d-%H%M%S").ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "0".into());
    let text = ini::ersetzen(bisher, GRUPPE, &neu.schreiben());
    ini::schreiben(pfad, &text, bisher, &zeit)
}
