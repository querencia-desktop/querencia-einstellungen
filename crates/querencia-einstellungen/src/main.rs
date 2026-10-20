//! Querencia Einstellungen – Kontrollzentrum im Stil von GNOME 2.
//!
//!   querencia-einstellungen                 Kontrollzentrum
//!   querencia-einstellungen --hintergrund   nur der Dialog „Querencia – Hintergrund“
//!                                           (für den Desktop-Menüpunkt)
//!   querencia-einstellungen --bildschirm    nur „Querencia – Bildschirm“ (G2-G)
//!   querencia-einstellungen --bildschirm-anwenden   gespeicherte Bildschirm-
//!                                           einstellungen setzen (Sitzungsstart,
//!                                           ohne Fenster)
//!   querencia-einstellungen --bildschirm-zeigen     Ausgaben als Text (Prüfung)
//!   querencia-einstellungen --arbeitsflaeche   nur „Querencia – Symbole der Arbeitsfläche“
//!                                           (G2-X: Symbole und Laufwerke)
//!   querencia-einstellungen --standardanwendungen   nur „Querencia – Standardanwendungen“
//!                                           (G2-Z, schreibt über GIO ~/.config/mimeapps.list)
//!   querencia-einstellungen --standardanwendungen-zeigen   Ist-Stand als Text (Prüfung)
//!   querencia-einstellungen --labwc-zeigen   labwc-Konfiguration: Orte, Wirkung,
//!                                           verwaltete Werte (G2-AG, Prüfung)
//!
//! Beide Einstiege benutzen denselben Dialog (`hintergrund::oeffnen`). Keine
//! libadwaita, keine Kopfleiste: labwc zeichnet die Titelleiste (wie Panel
//! und Desktop). Eindeutigkeit ist absichtlich aus (NON_UNIQUE), damit der
//! Desktop-Menüpunkt und das Kontrollzentrum unabhängig starten können.
mod arbeitsflaeche;
mod arbeitsflaeche_modell;
mod arbeitsflaechen;
mod ausgaben;
mod bereiche;
mod bildschirm;
mod bildschirm_modell;
mod erscheinungsbild;
mod faehigkeiten;
mod fensterdesign;
mod gsettings;
mod hintergrund;
mod hintergrund_katalog;
mod hintergrund_seite;
mod i18n;
mod ini;
mod labwc_konfig;
mod labwc_seite;
mod labwc_xml;
mod logo;
mod maus;
mod modell;
mod panels;
mod schriften;
mod sprache;
mod standardanwendungen;
mod standardanwendungen_modell;
mod tastatur;
mod vorschau;
mod xkb;

use crate::i18n::gettext;
use gtk4::prelude::*;
use gtk4::{gio, glib};

const APP_ID: &str = "org.querencia.Einstellungen";

/// Name des Fenstersymbols, das GTK labwc schickt (xdg-toplevel-icon).
const FENSTERSYMBOL: &str = "org.querencia.Einstellungen-fenster";

/// Eigenes Fenstersymbol (Zahnrad, G2-AA) für alle Fenster und Dialoge.
///
/// Bettet das SVG als Ressource ein, macht es dem Symbolthema bekannt und
/// setzt es als Vorgabe; damit hängt die Titelleiste nicht mehr davon ab,
/// welches Symbol labwc über die Desktop-Datei findet. Bewusst ein eigener,
/// nur als SVG vorhandener Name statt der App-ID: mit den PNG-Symbolen der
/// App-ID stürzt GTK 4.22 beim Abbau eines Fensters ab (Editor, G2-V).
fn fenstersymbol_einrichten() {
    let bytes =
        glib::Bytes::from_static(include_bytes!("../data/resource/einstellungen.gresource"));
    match gio::Resource::from_data(&bytes) {
        Ok(resource) => {
            gio::resources_register(&resource);
            if let Some(display) = gtk4::gdk::Display::default() {
                gtk4::IconTheme::for_display(&display)
                    .add_resource_path("/org/querencia/Einstellungen/icons");
            }
            gtk4::Window::set_default_icon_name(FENSTERSYMBOL);
        }
        Err(err) => eprintln!("querencia-einstellungen: Ressourcen nicht lesbar: {err}"),
    }
}

fn main() -> glib::ExitCode {
    vollstaendig_zeichnen();
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--version") {
        println!("querencia-einstellungen {}", env!("CARGO_PKG_VERSION"));
        return glib::ExitCode::SUCCESS;
    }
    // Diagnose ohne Fenster: was erkennt der Dialog am laufenden Desktop?
    if args.iter().any(|a| a == "--faehigkeiten-zeigen") {
        let f = faehigkeiten::ermitteln();
        println!(
            "farbe={} zentrieren={} kacheln={} laufwerkstypen={} symbole={} symbolgroesse={}",
            f.farbe, f.zentrieren, f.kacheln, f.laufwerkstypen, f.symbole_sichtbar, f.symbolgroesse
        );
        return glib::ExitCode::SUCCESS;
    }
    if args.iter().any(|a| a == "--bildschirm-anwenden") {
        return glib::ExitCode::from(bildschirm::gespeicherte_anwenden() as u8);
    }
    if args.iter().any(|a| a == "--bildschirm-zeigen") {
        return glib::ExitCode::from(bildschirm::zeigen() as u8);
    }
    if args.iter().any(|a| a == "--labwc-zeigen") {
        return glib::ExitCode::from(labwc_konfig::zeigen() as u8);
    }
    if args.iter().any(|a| a == "--standardanwendungen-zeigen") {
        return glib::ExitCode::from(standardanwendungen::zeigen() as u8);
    }
    i18n::init();
    let nur_hintergrund = args.iter().any(|a| a == "--hintergrund");
    let nur_bildschirm = args.iter().any(|a| a == "--bildschirm");
    let nur_arbeitsflaeche = args.iter().any(|a| a == "--arbeitsflaeche");
    let nur_standardanwendungen = args.iter().any(|a| a == "--standardanwendungen");
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_startup(|_| fenstersymbol_einrichten());
    app.connect_activate(move |app| {
        if nur_hintergrund {
            hintergrund::oeffnen(app).present();
        } else if nur_bildschirm {
            bildschirm::oeffnen(app).present();
        } else if nur_arbeitsflaeche {
            arbeitsflaeche::oeffnen(app).present();
        } else if nur_standardanwendungen {
            standardanwendungen::oeffnen(app).present();
        } else {
            kontrollzentrum(app).present();
        }
    });
    app.run_with_args::<&str>(&[])
}

/// Links die Bereiche, rechts die Seite; oben Logo und Name.
fn kontrollzentrum(app: &gtk4::Application) -> gtk4::Window {
    let fenster = gtk4::Window::builder()
        .application(app)
        .title(gettext("Querencia Settings"))
        .default_width(860)
        .default_height(600)
        .build();

    let kopf = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    kopf.set_margin_top(8);
    kopf.set_margin_bottom(4);
    kopf.set_margin_start(12);
    kopf.append(&logo::bild(32));
    let titel = gtk4::Label::new(None);
    titel.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(&gettext("Querencia Settings"))
    ));
    titel.set_xalign(0.0);
    kopf.append(&titel);

    let stapel = gtk4::Stack::new();
    stapel.set_transition_type(gtk4::StackTransitionType::None);
    for b in bereiche::alle(app) {
        let rollbar = gtk4::ScrolledWindow::new();
        rollbar.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        rollbar.set_child(Some(&b.seite));
        stapel.add_titled(&rollbar, Some(b.id), &b.name);
    }
    let liste = gtk4::StackSidebar::new();
    liste.set_stack(&stapel);
    liste.set_size_request(220, -1);

    let teilung = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    teilung.append(&liste);
    teilung.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
    stapel.set_hexpand(true);
    stapel.set_vexpand(true);
    teilung.append(&stapel);

    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    aussen.append(&kopf);
    aussen.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    aussen.append(&teilung);
    fenster.set_child(Some(&aussen));
    fenster
}

/// G2-AJ, Zeichenfehler: Nach einem Seitenwechsel (und beim Filtern im
/// Schriftwähler) blieben einzelne Pixelzeilen alter Texte stehen. GTK 4.22
/// zeichnet nur den Bereich neu, in dem sich die Darstellung geändert hat,
/// und bestimmt ihn aus den Grenzen der Textknoten; mit Hinting (Querencia:
/// „slight“) ragen Glyphen bis zu einem Pixel darüber hinaus, diese Reihe
/// wird nicht übermalt – mit jedem Renderer (Vulkan, GL, Cairo), belegt mit
/// `werkzeug/g2aj-reste.py`. `GSK_DEBUG=full-redraw` (dokumentierte
/// GTK-Variable) lässt GTK jedes Bild ganz zeichnen; für ein Fenster dieser
/// Größe kostet das nichts Spürbares. Wer `GSK_DEBUG` selbst setzt (auch
/// leer), behält seinen Wert.
fn vollstaendig_zeichnen() {
    if std::env::var_os("GSK_DEBUG").is_none() {
        // SAFETY: läuft als Erstes in `main`, bevor GTK oder ein anderer
        // Faden startet; niemand liest die Umgebung gleichzeitig.
        unsafe { std::env::set_var("GSK_DEBUG", "full-redraw") };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die eingecheckte GResource enthält das Zahnrad byte-gleich zum SVG im
    /// Repo (sonst `werkzeug/ressourcen.sh` vergessen).
    #[test]
    fn fenstersymbol_in_der_ressource() {
        let bytes =
            glib::Bytes::from_static(include_bytes!("../data/resource/einstellungen.gresource"));
        let res = gio::Resource::from_data(&bytes).expect("GResource lesbar");
        let pfad = format!("/org/querencia/Einstellungen/icons/scalable/apps/{FENSTERSYMBOL}.svg");
        let svg = res
            .lookup_data(&pfad, gio::ResourceLookupFlags::NONE)
            .expect("Fenstersymbol in der Ressource");
        let quelle =
            include_bytes!("../data/icons/hicolor/scalable/apps/org.querencia.Einstellungen.svg");
        assert_eq!(&svg[..], &quelle[..]);
    }
}
