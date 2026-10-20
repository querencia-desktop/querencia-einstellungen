//! Bereich „Standardanwendungen“ (QD-046 ①, G2-Z): je Kategorie die
//! zuständige Anwendung wie in den „Bevorzugten Anwendungen“ von GNOME 2,
//! wählbar aus den installierten Anwendungen. Gelesen und geschrieben wird
//! **nur über GIO**: `g_app_info_set_as_default_for_type` schreibt
//! `$XDG_CONFIG_HOME/mimeapps.list` (Benutzerdatei), nie systemweit, keine
//! andere Datei. Kategorien und Vorschläge: `standardanwendungen_modell.rs`.
//!
//! Auswahl in einem eigenen Dialog statt in einer Klappliste: nach oben
//! geklappte Listen bekommen unter labwc 0.9.8 keine Klicks (G2-G), und
//! die untersten Zeilen lägen genau dort.
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gio, glib};

use crate::i18n::{gettext, gettext_f, gettext_noop};
use crate::standardanwendungen_modell::{self as sm, Art, KATEGORIEN, Kategorie};

pub const NAME: &str = gettext_noop("Default Applications");

/// Fenstertitel des eigenen Fensters (`--standardanwendungen`).
pub fn titel() -> String {
    gettext_f("Querencia – {name}", &[("name", &gettext(NAME))])
}

// ------------------------------------------------------------------ GIO

/// Die Datei, in die GIO schreibt.
pub fn mimeapps_pfad() -> PathBuf {
    glib::user_config_dir().join("mimeapps.list")
}

/// Desktop-spezifische Listen (`<desktop>-mimeapps.list`), die GIO vor der
/// allgemeinen liest – nur zur Anzeige, falls vorhanden.
pub fn vorrangige_listen() -> Vec<PathBuf> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .split(':')
        .filter(|d| !d.is_empty())
        .map(|d| glib::user_config_dir().join(format!("{}-mimeapps.list", d.to_lowercase())))
        .filter(|p| p.is_file())
        .collect()
}

fn id(a: &gio::AppInfo) -> String {
    a.id().map(|s| s.to_string()).unwrap_or_default()
}

/// Die zuständige Anwendung für den Haupttyp der Kategorie.
pub fn aktuell(k: &Kategorie) -> Option<gio::AppInfo> {
    gio::AppInfo::default_for_type(k.haupttyp()?, false)
}

/// Wählbare Anwendungen: alle, die den Haupttyp öffnen (und im Menü
/// erscheinen), dazu die aktuelle; nach Name sortiert, ohne Doppelte.
pub fn kandidaten(k: &Kategorie) -> Vec<gio::AppInfo> {
    let Some(typ) = k.haupttyp() else {
        return Vec::new();
    };
    let mut liste: Vec<gio::AppInfo> = gio::AppInfo::all_for_type(typ)
        .into_iter()
        .filter(|a| a.should_show())
        .collect();
    if let Some(a) = aktuell(k) {
        liste.push(a);
    }
    liste.sort_by_cached_key(|a| (a.display_name().to_lowercase(), id(a)));
    liste.dedup_by_key(|a| id(a));
    liste
}

/// Anwendung als Standard der Kategorie festlegen. Gibt die gesetzten Typen
/// zurück.
pub fn festlegen(k: &Kategorie, app: &gio::AppInfo) -> Result<Vec<&'static str>, glib::Error> {
    let unterstuetzt: Vec<String> = app
        .supported_types()
        .iter()
        .map(|s| s.to_string())
        .collect();
    let typen = k.zu_setzende_typen(&unterstuetzt);
    for t in &typen {
        app.set_as_default_for_type(t)?;
    }
    Ok(typen)
}

/// Anwendung mit dieser Desktop-ID (für Terminal und Vorschläge).
fn anwendung(desktop_id: &str) -> Option<gio::AppInfo> {
    gio::AppInfo::all()
        .into_iter()
        .find(|a| id(a) == desktop_id)
}

/// Starter des Terminals (neue ID, sonst die Brücke älterer Installationen).
fn terminal() -> &'static str {
    sm::terminal_id(|i| anwendung(i).is_some())
}

/// `--standardanwendungen-zeigen`: Ist-Stand als Text (Prüfung), schreibt nichts.
pub fn zeigen() -> i32 {
    let pfad = mimeapps_pfad();
    println!(
        "datei={} vorhanden={}",
        pfad.display(),
        if pfad.is_file() { "ja" } else { "nein" }
    );
    for p in vorrangige_listen() {
        println!("vorrang={}", p.display());
    }
    let eigene = sm::gruppe_lesen(
        &std::fs::read_to_string(&pfad).unwrap_or_default(),
        "Default Applications",
    );
    for k in KATEGORIEN {
        let vorschlag = k.vorschlag.unwrap_or("-");
        match k.art {
            Art::Mime => {
                let ak = aktuell(k).map_or_else(|| "-".to_string(), |a| id(&a));
                let ka: Vec<String> = kandidaten(k).iter().map(id).collect();
                let quelle = if eigene.iter().any(|(t, _)| Some(t.as_str()) == k.haupttyp()) {
                    "benutzerdatei"
                } else {
                    "system"
                };
                println!(
                    "{}\t{}\taktuell={}\tquelle={}\tkandidaten={}\tvorschlag={}",
                    k.schluessel,
                    k.haupttyp().unwrap_or("-"),
                    ak,
                    quelle,
                    ka.join(","),
                    vorschlag
                );
            }
            Art::NurAnzeige => println!(
                "{}\t-\tfest={}\tvorhanden={}\tvorschlag={}",
                k.schluessel,
                terminal(),
                if anwendung(terminal()).is_some() {
                    "ja"
                } else {
                    "nein"
                },
                vorschlag
            ),
        }
    }
    0
}

// ------------------------------------------------------------------ Oberfläche

struct Zeile {
    kategorie: &'static Kategorie,
    symbol: gtk4::Image,
    name: gtk4::Label,
    aendern: Option<gtk4::Button>,
}

struct Seite {
    zeilen: Vec<Zeile>,
    status: gtk4::Label,
}

impl Seite {
    fn auffrischen(&self) {
        for z in &self.zeilen {
            let k = z.kategorie;
            let (app, text) = match k.art {
                Art::NurAnzeige => {
                    let a = anwendung(terminal());
                    let t = a.as_ref().map_or_else(
                        || gettext_f("{program} (not installed)", &[("program", terminal())]),
                        |a| a.display_name().to_string(),
                    );
                    (a, t)
                }
                Art::Mime => {
                    let a = aktuell(k);
                    let t = match &a {
                        Some(a) => a.display_name().to_string(),
                        None if kandidaten(k).is_empty() => {
                            gettext("No suitable application installed")
                        }
                        None => gettext("(not set)"),
                    };
                    (a, t)
                }
            };
            z.name.set_text(&text);
            if app.is_some() {
                z.name.remove_css_class("dim-label");
            } else {
                z.name.add_css_class("dim-label");
            }
            match (app.is_some(), app.and_then(|a| a.icon())) {
                (true, Some(i)) => z.symbol.set_from_gicon(&i),
                (true, None) => z.symbol.set_icon_name(Some("application-x-executable")),
                // Ohne Anwendung kein Symbol (die Zeile bleibt bündig).
                (false, _) => z.symbol.clear(),
            }
            if let Some(b) = &z.aendern {
                b.set_visible(!kandidaten(k).is_empty());
            }
        }
    }
}

/// Eigenes Fenster (`--standardanwendungen`).
pub fn oeffnen(app: &gtk4::Application) -> gtk4::Window {
    let fenster = gtk4::Window::builder()
        .application(app)
        .title(titel())
        .default_width(600)
        .default_height(640)
        .build();
    let rollbar = gtk4::ScrolledWindow::new();
    rollbar.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    rollbar.set_child(Some(&seite()));
    fenster.set_child(Some(&rollbar));
    fenster
}

/// Seite im Kontrollzentrum.
pub fn seite() -> gtk4::Widget {
    let s = crate::bereiche::seite(&gettext(NAME));
    let einleitung = gtk4::Label::new(Some(&gettext(
        "Which application opens files and addresses – in Querencia Files, on the desktop and in all programs that open via GIO. A change takes effect immediately.",
    )));
    einleitung.set_xalign(0.0);
    einleitung.set_wrap(true);
    s.append(&einleitung);

    let raster = gtk4::Grid::new();
    raster.set_row_spacing(6);
    raster.set_column_spacing(12);
    raster.set_margin_top(4);
    let mut zeilen = Vec::new();
    for (i, k) in KATEGORIEN.iter().enumerate() {
        let i = i as i32;
        let beschriftung = gtk4::Label::new(Some(&format!("{}:", gettext(k.name))));
        beschriftung.set_xalign(1.0);
        beschriftung.add_css_class("dim-label");
        if !k.typen.is_empty() {
            beschriftung.set_tooltip_text(Some(&k.typen.join(", ")));
        }
        let wert = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let symbol = gtk4::Image::new();
        symbol.set_pixel_size(16);
        symbol.set_size_request(16, 16);
        let name = gtk4::Label::new(None);
        name.set_xalign(0.0);
        name.set_hexpand(true);
        name.set_wrap(true);
        wert.append(&symbol);
        wert.append(&name);
        raster.attach(&beschriftung, 0, i, 1, 1);
        raster.attach(&wert, 1, i, 1, 1);
        let aendern = (k.art == Art::Mime).then(|| {
            let b = gtk4::Button::with_label(&gettext("Change …"));
            b.set_valign(gtk4::Align::Center);
            raster.attach(&b, 2, i, 1, 1);
            b
        });
        zeilen.push(Zeile {
            kategorie: k,
            symbol,
            name,
            aendern,
        });
    }
    s.append(&raster);

    let terminal = gtk4::Label::new(Some(&gettext(
        "Terminal: panel and desktop always open Querencia Terminal. There is no MIME type and no way to write it via GIO for terminals; the choice follows with a separate building block.",
    )));
    terminal.set_xalign(0.0);
    terminal.set_wrap(true);
    terminal.add_css_class("dim-label");
    s.append(&terminal);

    let status = gtk4::Label::new(None);
    status.set_xalign(0.0);
    status.set_wrap(true);
    status.set_selectable(true);
    s.append(&status);

    let mut herkunft = gettext_f(
        "GIO: {file} (user file; if it is missing, the system default applies). Only this file is written.",
        &[("file", &mimeapps_pfad().display().to_string())],
    );
    for p in vorrangige_listen() {
        herkunft.push(' ');
        herkunft.push_str(&gettext_f(
            "{file} takes precedence – entries there apply before this page.",
            &[("file", &p.display().to_string())],
        ));
    }
    s.append(&crate::bereiche::quelle(&herkunft));

    let seite = Rc::new(Seite { zeilen, status });
    seite.auffrischen();
    for z in &seite.zeilen {
        if let Some(b) = &z.aendern {
            let k = z.kategorie;
            let seite = seite.clone();
            b.connect_clicked(move |b| {
                let eltern = b.root().and_downcast::<gtk4::Window>();
                waehlen(eltern.as_ref(), k, seite.clone());
            });
        }
    }
    s.upcast()
}

/// Auswahldialog einer Kategorie. Der Vorschlag ist nur markiert, nie
/// vorausgewählt; vorausgewählt ist die aktuelle Anwendung.
fn waehlen(eltern: Option<&gtk4::Window>, k: &'static Kategorie, seite: Rc<Seite>) {
    let dialog = gtk4::Window::builder()
        .title(gettext_f(
            "Choose Default Application – {category}",
            &[("category", &gettext(k.name))],
        ))
        .modal(true)
        .default_width(460)
        .default_height(380)
        .build();
    dialog.set_transient_for(eltern);

    let aussen = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    aussen.set_margin_top(12);
    aussen.set_margin_bottom(12);
    aussen.set_margin_start(12);
    aussen.set_margin_end(12);
    let kopf = gtk4::Label::new(None);
    kopf.set_markup(&gettext_f(
        "<b>{category}</b> opens: {types}",
        &[
            ("category", &glib::markup_escape_text(&gettext(k.name))),
            ("types", &glib::markup_escape_text(&k.typen.join(", "))),
        ],
    ));
    kopf.set_xalign(0.0);
    kopf.set_wrap(true);
    aussen.append(&kopf);

    let ist = aktuell(k).map(|a| id(&a));
    let apps = kandidaten(k);
    let liste = gtk4::ListBox::new();
    liste.set_selection_mode(gtk4::SelectionMode::Single);
    for a in &apps {
        let zeile = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        zeile.set_margin_top(4);
        zeile.set_margin_bottom(4);
        zeile.set_margin_start(6);
        zeile.set_margin_end(6);
        let bild = a.icon().map_or_else(
            || gtk4::Image::from_icon_name("application-x-executable"),
            |i| gtk4::Image::from_gicon(&i),
        );
        bild.set_pixel_size(24);
        zeile.append(&bild);
        let texte = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let mut name = a.display_name().to_string();
        let aid = id(a);
        if k.vorschlag == Some(aid.as_str()) {
            name = gettext_f("{name} (suggestion)", &[("name", &name)]);
        }
        if ist.as_deref() == Some(aid.as_str()) {
            name = gettext_f("{name} – current", &[("name", &name)]);
        }
        let n = gtk4::Label::new(Some(&name));
        n.set_xalign(0.0);
        let d = gtk4::Label::new(Some(&aid));
        d.set_xalign(0.0);
        d.add_css_class("dim-label");
        texte.append(&n);
        texte.append(&d);
        zeile.append(&texte);
        liste.append(&zeile);
    }
    if let Some(pos) = apps.iter().position(|a| Some(id(a)) == ist) {
        liste.select_row(liste.row_at_index(pos as i32).as_ref());
    }
    let rollbar = gtk4::ScrolledWindow::new();
    rollbar.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    rollbar.set_vexpand(true);
    rollbar.set_child(Some(&liste));
    rollbar.add_css_class("frame");
    aussen.append(&rollbar);

    if let Some(v) = k.vorschlag
        && !apps.iter().any(|a| id(a) == v)
    {
        let l = gtk4::Label::new(Some(&gettext_f(
            "Suggestion {id} is not installed.",
            &[("id", v)],
        )));
        l.set_xalign(0.0);
        l.add_css_class("dim-label");
        aussen.append(&l);
    }

    let knoepfe = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    knoepfe.set_halign(gtk4::Align::End);
    let abbrechen = gtk4::Button::with_label(&gettext("Cancel"));
    let uebernehmen = gtk4::Button::with_label(&gettext("Apply"));
    uebernehmen.add_css_class("suggested-action");
    knoepfe.append(&abbrechen);
    knoepfe.append(&uebernehmen);
    aussen.append(&knoepfe);
    dialog.set_child(Some(&aussen));
    dialog.set_default_widget(Some(&uebernehmen));

    let apps = Rc::new(apps);
    let ausfuehren = Rc::new({
        let dialog = dialog.clone();
        let liste = liste.clone();
        let apps = apps.clone();
        move || {
            let Some(zeile) = liste.selected_row() else {
                dialog.close();
                return;
            };
            let Some(app) = apps.get(zeile.index() as usize) else {
                dialog.close();
                return;
            };
            let neu = id(app);
            if aktuell(k).map(|a| id(&a)).as_deref() == Some(neu.as_str()) {
                dialog.close();
                return;
            }
            let meldung = match festlegen(k, app) {
                Ok(typen) => {
                    let jetzt = aktuell(k).map(|a| id(&a));
                    if jetzt.as_deref() == Some(neu.as_str()) {
                        gettext_f(
                            "{category}: “{application}” is now the default for {types}. Saved in {file}.",
                            &[
                                ("category", &gettext(k.name)),
                                ("application", &app.display_name()),
                                ("types", &typen.join(", ")),
                                ("file", &mimeapps_pfad().display().to_string()),
                            ],
                        )
                    } else {
                        gettext_f(
                            "{category}: saved, but GIO still reports “{application}” – a list with precedence ({lists}) sets something else.",
                            &[
                                ("category", &gettext(k.name)),
                                ("application", &jetzt.unwrap_or_else(|| gettext("(none)"))),
                                (
                                    "lists",
                                    &vorrangige_listen()
                                        .iter()
                                        .map(|p| p.display().to_string())
                                        .collect::<Vec<_>>()
                                        .join(", "),
                                ),
                            ],
                        )
                    }
                }
                Err(e) => gettext_f(
                    "{category}: not saved – {error}",
                    &[("category", &gettext(k.name)), ("error", &e.to_string())],
                ),
            };
            eprintln!("standardanwendungen: {meldung}");
            seite.status.set_text(&meldung);
            seite.auffrischen();
            dialog.close();
        }
    });
    {
        let a = ausfuehren.clone();
        uebernehmen.connect_clicked(move |_| a());
    }
    {
        let a = ausfuehren.clone();
        liste.connect_row_activated(move |_, _| a());
    }
    {
        let dialog = dialog.clone();
        abbrechen.connect_clicked(move |_| dialog.close());
    }
    let tasten = gtk4::EventControllerKey::new();
    {
        let dialog = dialog.clone();
        tasten.connect_key_pressed(move |_, taste, _, _| {
            if taste == gtk4::gdk::Key::Escape {
                dialog.close();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
    }
    dialog.add_controller(tasten);
    dialog.present();
}

// ------------------------------------------------------------------ Tests

/// GIO merkt sich `XDG_CONFIG_HOME` und die Anwendungsordner beim ersten
/// Zugriff für den ganzen Prozess. Deshalb läuft der Rundlauf in einem
/// eigenen Kindprozess (dasselbe Testprogramm, nur der innere Test) mit
/// umgelenktem `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_DATA_DIRS`.
/// Der innere Test verweigert, wenn die Umlenkung fehlt – er schreibt nie
/// in die echte `~/.config/mimeapps.list`.
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const MARKE: &str = "QE_G2Z_TESTWURZEL";

    fn schreiben(p: &Path, text: &str) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn starter(ordner: &Path, id: &str, name: &str, mime: &str) {
        schreiben(
            &ordner.join(id),
            &format!(
                "[Desktop Entry]\nType=Application\nName={name}\nExec=true %F\nMimeType={mime}\n"
            ),
        );
    }

    fn dateien(wurzel: &Path) -> Vec<String> {
        let mut aus = Vec::new();
        let mut stapel = vec![wurzel.to_path_buf()];
        while let Some(d) = stapel.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if e.file_type().unwrap().is_dir() {
                    stapel.push(p);
                } else {
                    aus.push(p.strip_prefix(wurzel).unwrap().display().to_string());
                }
            }
        }
        aus.sort();
        aus
    }

    /// Startet den inneren Test im Kindprozess; `vorher` = Inhalt einer
    /// vorhandenen mimeapps.list (oder keine).
    fn kind(fall: &str, vorher: Option<&str>) {
        let wurzel = std::env::temp_dir().join(format!("qe-g2z-{}-{}", fall, std::process::id()));
        let _ = std::fs::remove_dir_all(&wurzel);
        let apps = wurzel.join("daten/applications");
        starter(
            &apps,
            "test.editor.desktop",
            "Testeditor",
            "text/plain;text/markdown;text/x-markdown;",
        );
        starter(&apps, "test.vim.desktop", "Testvim", "text/plain;");
        starter(
            &apps,
            "test.bilder.desktop",
            "Testbilder",
            "image/png;image/jpeg;",
        );
        starter(
            &apps,
            "test.web.desktop",
            "Testweb",
            "image/png;x-scheme-handler/http;x-scheme-handler/https;text/html;",
        );
        // GIO nimmt die registrierten Anwendungen aus mimeinfo.cache, nicht
        // aus den Startern selbst (wie im echten ~/.local/share/applications).
        let udd = std::process::Command::new("update-desktop-database")
            .arg("-q")
            .arg(&apps)
            .status()
            .unwrap();
        assert!(udd.success());
        // Der MIME-Katalog des Systems (nur gelesen), ohne die System-Starter.
        std::fs::create_dir_all(wurzel.join("system")).unwrap();
        std::os::unix::fs::symlink("/usr/share/mime", wurzel.join("system/mime")).unwrap();
        std::fs::create_dir_all(wurzel.join("konfig")).unwrap();
        if let Some(t) = vorher {
            schreiben(&wurzel.join("konfig/mimeapps.list"), t);
        }
        let ausgabe = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                &format!("standardanwendungen::tests::innen_{fall}"),
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(MARKE, &wurzel)
            .env("HOME", wurzel.join("heim"))
            .env("XDG_CONFIG_HOME", wurzel.join("konfig"))
            .env("XDG_DATA_HOME", wurzel.join("daten"))
            .env("XDG_DATA_DIRS", wurzel.join("system"))
            .env("XDG_CONFIG_DIRS", wurzel.join("system"))
            .env_remove("XDG_CURRENT_DESKTOP")
            .output()
            .unwrap();
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&ausgabe.stdout),
            String::from_utf8_lossy(&ausgabe.stderr)
        );
        assert!(ausgabe.status.success(), "Kindprozess {fall}:\n{text}");
        assert!(
            text.contains("1 passed"),
            "innerer Test lief nicht:\n{text}"
        );
        // Außer der mimeapps.list entsteht nichts (keine Starter, keine
        // anderen Dateien, nichts unter HOME).
        let mut erwartet: Vec<String> = [
            "daten/applications/test.bilder.desktop",
            "daten/applications/test.editor.desktop",
            "daten/applications/test.vim.desktop",
            "daten/applications/mimeinfo.cache",
            "daten/applications/test.web.desktop",
            "konfig/mimeapps.list",
            "system/mime",
        ]
        .map(str::to_string)
        .to_vec();
        erwartet.sort();
        assert_eq!(dateien(&wurzel), erwartet);
        std::fs::remove_dir_all(&wurzel).unwrap();
    }

    /// Nur im Kindprozess: verweigert ohne Umlenkung.
    fn umgelenkt() -> PathBuf {
        let w = PathBuf::from(std::env::var_os(MARKE).expect("nur über kind() starten"));
        assert!(w.starts_with(std::env::temp_dir()));
        assert_eq!(glib::user_config_dir(), w.join("konfig"));
        assert_eq!(mimeapps_pfad(), w.join("konfig/mimeapps.list"));
        w
    }

    fn standard(text: &str, typ: &str) -> Option<Vec<String>> {
        sm::gruppe_lesen(text, "Default Applications")
            .into_iter()
            .find(|(t, _)| t == typ)
            .map(|(_, ids)| ids)
    }

    #[test]
    fn rundlauf_ohne_datei() {
        kind("ohne_datei", None);
    }

    #[test]
    #[ignore = "nur als Kindprozess von rundlauf_ohne_datei"]
    fn innen_ohne_datei() {
        let w = umgelenkt();
        let datei = w.join("konfig/mimeapps.list");
        assert!(!datei.exists());
        let te = sm::kategorie("texteditor").unwrap();
        let ids: Vec<String> = kandidaten(te).iter().map(id).collect();
        assert_eq!(ids, ["test.editor.desktop", "test.vim.desktop"]);
        // Lesen ohne Datei schreibt nichts.
        let _ = aktuell(te);
        assert!(!datei.exists());
        // Vim nennt nur text/plain.
        let vim = kandidaten(te)
            .into_iter()
            .find(|a| id(a) == "test.vim.desktop")
            .unwrap();
        assert_eq!(festlegen(te, &vim).unwrap(), ["text/plain"]);
        assert_eq!(id(&aktuell(te).unwrap()), "test.vim.desktop");
        let text = std::fs::read_to_string(&datei).unwrap();
        assert_eq!(standard(&text, "text/plain").unwrap(), ["test.vim.desktop"]);
        assert_eq!(standard(&text, "text/markdown"), None);
        // Editor: alle drei Texttypen.
        let ed = kandidaten(te)
            .into_iter()
            .find(|a| id(a) == "test.editor.desktop")
            .unwrap();
        assert_eq!(
            festlegen(te, &ed).unwrap(),
            ["text/plain", "text/markdown", "text/x-markdown"]
        );
        let text = std::fs::read_to_string(&datei).unwrap();
        for t in ["text/plain", "text/markdown", "text/x-markdown"] {
            assert_eq!(standard(&text, t).unwrap(), ["test.editor.desktop"], "{t}");
        }
        assert_eq!(id(&aktuell(te).unwrap()), "test.editor.desktop");
        assert_eq!(
            id(&gio::AppInfo::default_for_type("text/markdown", false).unwrap()),
            "test.editor.desktop"
        );
        // Kategorie ohne Anwendung: keine Auswahl, keine Zuständige.
        let musik = sm::kategorie("musik").unwrap();
        assert!(kandidaten(musik).is_empty());
        assert!(aktuell(musik).is_none());
        println!("innen_ohne_datei ok");
    }

    const VORHER: &str = "[Default Applications]\n\
        text/plain=test.vim.desktop\n\
        application/json=test.vim.desktop\n\
        image/png=test.web.desktop\n\
        x-scheme-handler/http=test.web.desktop\n\
        \n\
        [Added Associations]\n\
        application/x-fremd=fremd.desktop;\n";

    #[test]
    fn rundlauf_vorhandene_datei() {
        kind("vorhandene_datei", Some(VORHER));
    }

    #[test]
    #[ignore = "nur als Kindprozess von rundlauf_vorhandene_datei"]
    fn innen_vorhandene_datei() {
        let w = umgelenkt();
        let datei = w.join("konfig/mimeapps.list");
        let bi = sm::kategorie("bilder").unwrap();
        let te = sm::kategorie("texteditor").unwrap();
        let web = sm::kategorie("webbrowser").unwrap();
        // Lesen: die Datei bestimmt.
        assert_eq!(id(&aktuell(bi).unwrap()), "test.web.desktop");
        assert_eq!(id(&aktuell(te).unwrap()), "test.vim.desktop");
        assert_eq!(id(&aktuell(web).unwrap()), "test.web.desktop");
        let ids: Vec<String> = kandidaten(bi).iter().map(id).collect();
        assert_eq!(ids, ["test.bilder.desktop", "test.web.desktop"]);
        // Bilder ändern: image/png und image/jpeg, sonst nichts.
        let b = kandidaten(bi)
            .into_iter()
            .find(|a| id(a) == "test.bilder.desktop")
            .unwrap();
        assert_eq!(festlegen(bi, &b).unwrap(), ["image/png", "image/jpeg"]);
        let text = std::fs::read_to_string(&datei).unwrap();
        assert_eq!(
            standard(&text, "image/png").unwrap(),
            ["test.bilder.desktop"]
        );
        assert_eq!(
            standard(&text, "image/jpeg").unwrap(),
            ["test.bilder.desktop"]
        );
        // Fremde Einträge bleiben.
        assert_eq!(standard(&text, "text/plain").unwrap(), ["test.vim.desktop"]);
        assert_eq!(
            standard(&text, "application/json").unwrap(),
            ["test.vim.desktop"]
        );
        assert_eq!(
            standard(&text, "x-scheme-handler/http").unwrap(),
            ["test.web.desktop"]
        );
        assert!(
            sm::gruppe_lesen(&text, "Added Associations")
                .iter()
                .any(|(t, ids)| t == "application/x-fremd" && ids == &["fremd.desktop"])
        );
        assert_eq!(id(&aktuell(bi).unwrap()), "test.bilder.desktop");
        assert_eq!(id(&aktuell(te).unwrap()), "test.vim.desktop");
        // Webbrowser auf denselben Stand setzen ändert die Zuordnung nicht.
        let wa = aktuell(web).unwrap();
        festlegen(web, &wa).unwrap();
        assert_eq!(id(&aktuell(web).unwrap()), "test.web.desktop");
        let text = std::fs::read_to_string(&datei).unwrap();
        assert_eq!(standard(&text, "text/html").unwrap(), ["test.web.desktop"]);
        println!("innen_vorhandene_datei ok");
    }
}
