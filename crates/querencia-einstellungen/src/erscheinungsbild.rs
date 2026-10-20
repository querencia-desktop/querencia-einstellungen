//! Bereich „Erscheinungsbild“ (G2-AJ, QD-046 ④): Symbolthema, GTK-Thema,
//! Farbschema und Mauszeiger. Geschrieben wird `org.gnome.desktop.interface`
//! (wirkt sofort über das Settings-Portal, auch in Panel und Desktop, die
//! seit G2-AD dem Symbolthema folgen) und für labwc `XCURSOR_THEME`/
//! `XCURSOR_SIZE` in der Umgebung der Benutzerkopie (labwc lädt den Zeiger
//! beim `--reconfigure` neu; Programme außerhalb von GTK ab der nächsten
//! Anmeldung).
//!
//! Angeboten werden nur installierte Themen: Symbolthemen mit `index.theme`
//! und Verzeichnissen (ohne `Hidden=true`, außer Adwaita), GTK-Themen mit
//! `gtk-4.0/gtk.css` und das eingebaute Thema von GTK, Zeigerthemen mit
//! `cursors/`.
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, glib};

use crate::gsettings::{
    self as gs, FARBSCHEMA, GTK_THEMA, SYMBOLTHEMA, ZEIGERGROESSE, ZEIGERTHEMA,
};
use crate::labwc_konfig::{Stand, Wirkung};
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::Fehler;

/// Das in GTK eingebaute Thema (keine Datei nötig).
pub const GTK_EINGEBAUT: &str = "Adwaita";
/// Panel und Desktop zeigen bei diesem Wert (wie ungesetzt) Querencia (G2-AD).
const SYMBOLE_WIE_QUERENCIA: &str = "Adwaita";
/// Zeigergrößen zur Auswahl (Pixel).
pub const ZEIGERGROESSEN: &[(u32, &str)] = &[
    (24, "Normal (24)"),
    (32, "Groß (32)"),
    (48, "Sehr groß (48)"),
];
/// Symbole der Vorschau (gibt es in allen angebotenen Themen).
const VORSCHAU: &[&str] = &[
    "folder",
    "user-home",
    "computer",
    "text-x-generic",
    "user-trash",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thema {
    /// Ordnername = Wert in gsettings
    pub name: String,
    pub anzeige: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    pub symbolthema: String,
    pub gtk_thema: String,
    pub zeigerthema: String,
    pub zeigergroesse: u32,
    /// `color-scheme` = `prefer-dark`
    pub dunkel: bool,
    /// `color-scheme` beim Lesen (bleibt, solange `dunkel` gleich bleibt)
    pub farbschema: String,
}

impl Werte {
    fn gs_liste(&self) -> Vec<(&'static str, String)> {
        vec![
            (SYMBOLTHEMA, self.symbolthema.clone()),
            (GTK_THEMA, self.gtk_thema.clone()),
            (ZEIGERTHEMA, self.zeigerthema.clone()),
            (ZEIGERGROESSE, self.zeigergroesse.to_string()),
            (FARBSCHEMA, self.farbschema.clone()),
        ]
    }
}

fn ordner(unterordner: &str) -> Vec<PathBuf> {
    let mut liste = vec![
        glib::user_data_dir().join(unterordner),
        glib::home_dir().join(format!(".{unterordner}")),
    ];
    liste.extend(
        glib::system_data_dirs()
            .into_iter()
            .map(|d| d.join(unterordner)),
    );
    liste
}

/// `index.theme`: Anzeigename (übersetzt, falls vorhanden), Verzeichnisse, versteckt.
fn index(pfad: &std::path::Path) -> Option<(Option<String>, bool, bool)> {
    let kf = glib::KeyFile::new();
    kf.load_from_file(pfad.join("index.theme"), glib::KeyFileFlags::NONE)
        .ok()?;
    let gruppe = "Icon Theme";
    let name = kf
        .locale_string(gruppe, "Name", None)
        .ok()
        .map(|s| s.to_string());
    let verzeichnisse = kf
        .string(gruppe, "Directories")
        .is_ok_and(|s| !s.trim().is_empty());
    let versteckt = kf.boolean(gruppe, "Hidden").unwrap_or(false);
    Some((name, verzeichnisse, versteckt))
}

fn sortieren(mut liste: Vec<Thema>) -> Vec<Thema> {
    liste.sort_by_key(|t| t.anzeige.to_lowercase());
    liste
}

/// Installierte Symbolthemen (der erste Fund eines Namens gilt).
pub fn symbolthemen() -> Vec<Thema> {
    let mut liste: Vec<Thema> = Vec::new();
    for d in ordner("icons") {
        let Ok(eintraege) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in eintraege.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name == "default" || name == "hicolor" || liste.iter().any(|t| t.name == name) {
                continue;
            }
            let Some((anzeige, verzeichnisse, versteckt)) = index(&e.path()) else {
                continue;
            };
            if !verzeichnisse || (versteckt && name != SYMBOLE_WIE_QUERENCIA) {
                continue;
            }
            liste.push(Thema {
                anzeige: anzeige.unwrap_or_else(|| name.clone()),
                name,
            });
        }
    }
    sortieren(liste)
}

/// Installierte GTK-4-Themen und das eingebaute.
pub fn gtk_themen() -> Vec<Thema> {
    let mut liste = vec![Thema {
        name: GTK_EINGEBAUT.into(),
        anzeige: "GTK-Standard (eingebaut)".into(),
    }];
    for d in ordner("themes") {
        let Ok(eintraege) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in eintraege.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if e.path().join("gtk-4.0").join("gtk.css").is_file()
                && !liste.iter().any(|t| t.name == name)
            {
                liste.push(Thema {
                    anzeige: name.clone(),
                    name,
                });
            }
        }
    }
    liste
}

/// Installierte Zeigerthemen (Ordner mit `cursors/`), Anzeigename aus `index.theme`.
pub fn zeigerthemen() -> Vec<Thema> {
    let mut liste: Vec<Thema> = Vec::new();
    for name in crate::maus::zeigerthemen() {
        let anzeige = ordner("icons")
            .iter()
            .find_map(|d| index(&d.join(&name)).and_then(|(n, _, _)| n))
            .unwrap_or_else(|| name.clone());
        liste.push(Thema { name, anzeige });
    }
    sortieren(liste)
}

/// Optionsliste mit dem aktuellen Wert, auch wenn er nicht installiert ist.
fn mit_aktuellem(themen: &[Thema], aktuell: &str) -> Vec<(String, String)> {
    let mut liste: Vec<(String, String)> = themen
        .iter()
        .map(|t| (t.name.clone(), t.anzeige.clone()))
        .collect();
    if !aktuell.is_empty() && !themen.iter().any(|t| t.name == aktuell) {
        liste.push((aktuell.to_owned(), format!("{aktuell} (nicht installiert)")));
    }
    liste
}

/// Was der gewählten Option folgt: Vorschau der Symbole, Hinweise.
struct Neben {
    vorschau: gtk4::Box,
    vorschau_cache: RefCell<HashMap<String, Vec<gdk::Paintable>>>,
    symbol_hinweis: gtk4::Label,
}

pub struct Erscheinungsbild {
    symbole_kasten: gtk4::Grid,
    symbole: RefCell<Vec<(String, gtk4::CheckButton)>>,
    gtk_kasten: gtk4::Grid,
    gtk: RefCell<Vec<(String, gtk4::CheckButton)>>,
    dunkel: gtk4::CheckButton,
    neben: Rc<Neben>,
    zeiger_kasten: gtk4::Box,
    zeiger: RefCell<Vec<(String, gtk4::CheckButton)>>,
    groessen: Vec<gtk4::CheckButton>,
    eigene_groesse: std::cell::Cell<u32>,
    farbschema: RefCell<String>,
    geaendert: Rc<dyn Fn()>,
}

fn gewaehlt(l: &RefCell<Vec<(String, gtk4::CheckButton)>>) -> String {
    l.borrow()
        .iter()
        .find(|(_, k)| k.is_active())
        .map(|(t, _)| t.clone())
        .unwrap_or_default()
}

impl Bereich for Erscheinungsbild {
    type Werte = Werte;
    const NAME: &'static str = "Erscheinungsbild";
    const QUELLE: &'static str = "gsettings org.gnome.desktop.interface (icon-theme, gtk-theme, color-scheme, cursor-theme, cursor-size); für labwc ~/.config/querencia/labwc/environment.d/zz-querencia-einstellungen.env (XCURSOR_THEME, XCURSOR_SIZE). Themen aus ~/.local/share/icons, ~/.local/share/themes, /usr/share/icons und /usr/share/themes.";
    const GSETTINGS: &'static [&'static str] = &[
        SYMBOLTHEMA,
        GTK_THEMA,
        ZEIGERTHEMA,
        ZEIGERGROESSE,
        FARBSCHEMA,
    ];

    fn lesen(st: &Stand) -> Werte {
        let wert = |k: &str| {
            st.gs.get(k).cloned().unwrap_or_else(|| {
                gs::STANDARD
                    .iter()
                    .find(|(s, _)| *s == k)
                    .map_or_else(String::new, |(_, v)| (*v).to_owned())
            })
        };
        let farbschema = wert(FARBSCHEMA);
        Werte {
            symbolthema: wert(SYMBOLTHEMA),
            gtk_thema: wert(GTK_THEMA),
            zeigerthema: wert(ZEIGERTHEMA),
            zeigergroesse: wert(ZEIGERGROESSE).parse().unwrap_or(24),
            dunkel: farbschema == "prefer-dark",
            farbschema,
        }
    }

    fn rc_aendern(_neu: &Werte, _alt: &Werte, rc: &str) -> Result<String, Fehler> {
        Ok(rc.to_owned())
    }

    fn env_aenderungen(
        neu: &Werte,
        alt: &Werte,
        st: &Stand,
    ) -> Vec<(&'static str, Option<String>)> {
        let mut v = Vec::new();
        if neu.zeigerthema != alt.zeigerthema {
            v.push(ls::env_wert(st, "XCURSOR_THEME", &neu.zeigerthema));
        }
        if neu.zeigergroesse != alt.zeigergroesse {
            v.push(ls::env_wert(
                st,
                "XCURSOR_SIZE",
                &neu.zeigergroesse.to_string(),
            ));
        }
        v
    }

    fn gs_aenderungen(neu: &Werte, alt: &Werte) -> Vec<(&'static str, String)> {
        gs::unterschiede(&neu.gs_liste(), &alt.gs_liste())
    }

    fn hinweis(w: &Wirkung, _st: &Stand) -> Option<String> {
        let labwc = match w {
            Wirkung::Sofort { .. } => "sofort nach „Übernehmen“",
            Wirkung::NaechsteAnmeldung { .. } => "ab der nächsten Anmeldung",
        };
        Some(format!(
            "Symbole, Thema, Farbschema und Zeiger wirken nach „Übernehmen“ sofort in allen GTK-Programmen, im Panel und auf dem Desktop. Über Fensterrahmen und freier Arbeitsfläche (labwc) wechselt der Zeiger {labwc}, in anderen Programmen (z. B. X11) ab der nächsten Anmeldung."
        ))
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

        let (f, b) = ls::rahmen("Symbole");
        let symbole_kasten = gtk4::Grid::new();
        symbole_kasten.set_row_spacing(2);
        symbole_kasten.set_column_spacing(12);
        b.append(&symbole_kasten);
        let vorschau = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        vorschau.set_margin_top(4);
        b.append(&ls::zeile("Vorschau:", &vorschau));
        let symbol_hinweis = ls::grau("");
        b.append(&symbol_hinweis);
        inhalt.append(&f);

        let (f, b) = ls::rahmen("Steuerelemente (GTK-Thema)");
        let gtk_kasten = gtk4::Grid::new();
        gtk_kasten.set_row_spacing(2);
        gtk_kasten.set_column_spacing(12);
        b.append(&gtk_kasten);
        let dunkel = gtk4::CheckButton::with_label("Dunkle Darstellung bevorzugen");
        b.append(&dunkel);
        b.append(&ls::grau(
            "Dunkel werden Programme, die diese Vorliebe auswerten (z. B. Firefox, Webseiten, GNOME-Programme); Querencia-Programme behalten ihr Thema.",
        ));
        inhalt.append(&f);

        let (f, b) = ls::rahmen("Mauszeiger");
        let zeiger_kasten = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        b.append(&ls::zeile("Thema:", &zeiger_kasten));
        let gr = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let mut groessen: Vec<gtk4::CheckButton> = Vec::new();
        for (_, text) in ZEIGERGROESSEN {
            let k = gtk4::CheckButton::with_label(text);
            if let Some(erste) = groessen.first() {
                k.set_group(Some(erste));
            }
            gr.append(&k);
            groessen.push(k);
        }
        b.append(&ls::zeile("Größe:", &gr));
        b.append(&ls::grau(
            "Derselbe Zeiger steht im Bereich „Maus“; beide Bereiche schreiben dieselben Werte.",
        ));
        inhalt.append(&f);

        let e = Erscheinungsbild {
            symbole_kasten,
            symbole: RefCell::new(Vec::new()),
            gtk_kasten,
            gtk: RefCell::new(Vec::new()),
            dunkel,
            neben: Rc::new(Neben {
                vorschau,
                vorschau_cache: RefCell::new(HashMap::new()),
                symbol_hinweis,
            }),
            zeiger_kasten,
            zeiger: RefCell::new(Vec::new()),
            groessen,
            eigene_groesse: std::cell::Cell::new(24),
            farbschema: RefCell::new("default".into()),
            geaendert: geaendert.clone(),
        };
        let g = geaendert.clone();
        e.dunkel.connect_toggled(move |_| g());
        for k in &e.groessen {
            let g = geaendert.clone();
            k.connect_toggled(move |k| {
                if k.is_active() {
                    g();
                }
            });
        }
        (e, inhalt)
    }

    fn zeigen(&self, w: &Werte) {
        ls::optionen_raster(
            &self.symbole_kasten,
            &self.symbole,
            &mit_aktuellem(&symbolthemen(), &w.symbolthema),
            &w.symbolthema,
            &self.geaendert,
            3,
        );
        ls::optionen_raster(
            &self.gtk_kasten,
            &self.gtk,
            &mit_aktuellem(&gtk_themen(), &w.gtk_thema),
            &w.gtk_thema,
            &self.geaendert,
            2,
        );
        ls::optionen(
            &self.zeiger_kasten,
            &self.zeiger,
            &mit_aktuellem(&zeigerthemen(), &w.zeigerthema),
            &w.zeigerthema,
            &self.geaendert,
        );
        self.dunkel.set_active(w.dunkel);
        match ZEIGERGROESSEN
            .iter()
            .position(|(g, _)| *g == w.zeigergroesse)
        {
            Some(i) => self.groessen[i].set_active(true),
            None => {
                for k in &self.groessen {
                    k.set_active(false);
                }
            }
        }
        self.eigene_groesse.set(w.zeigergroesse);
        *self.farbschema.borrow_mut() = w.farbschema.clone();
        self.neben.symbole(&w.symbolthema);
        // Vorschau und Hinweise folgen jeder Wahl
        for (name, k) in self.symbole.borrow().iter() {
            let (n, name) = (self.neben.clone(), name.clone());
            k.connect_toggled(move |k| {
                if k.is_active() {
                    n.symbole(&name);
                }
            });
        }
    }

    fn aktuell(&self) -> Werte {
        let dunkel = self.dunkel.is_active();
        let gelesen = self.farbschema.borrow().clone();
        let farbschema = if dunkel == (gelesen == "prefer-dark") {
            gelesen
        } else if dunkel {
            "prefer-dark".into()
        } else {
            "default".into()
        };
        Werte {
            symbolthema: gewaehlt(&self.symbole),
            gtk_thema: gewaehlt(&self.gtk),
            zeigerthema: gewaehlt(&self.zeiger),
            zeigergroesse: self
                .groessen
                .iter()
                .position(|k| k.is_active())
                .map_or(self.eigene_groesse.get(), |i| ZEIGERGROESSEN[i].0),
            dunkel,
            farbschema,
        }
    }
}

impl Neben {
    /// Vorschau der Symbole und Hinweis zum Symbolthema.
    fn symbole(&self, symbolthema: &str) {
        while let Some(c) = self.vorschau.first_child() {
            self.vorschau.remove(&c);
        }
        let bilder = self
            .vorschau_cache
            .borrow_mut()
            .entry(symbolthema.to_owned())
            .or_insert_with(|| symbole_von(symbolthema))
            .clone();
        for p in bilder {
            let bild = gtk4::Image::from_paintable(Some(&p));
            bild.set_pixel_size(32);
            self.vorschau.append(&bild);
        }
        self.symbol_hinweis.set_text(if symbolthema == SYMBOLE_WIE_QUERENCIA {
            "Panel und Desktop zeigen bei „Adwaita“ weiter die Querencia-Symbole; Programme zeigen Adwaita."
        } else {
            "Panel, Desktop und Programme zeigen dieses Thema; fehlende Symbole kommen aus den Themen, auf die es aufbaut."
        });
    }
}

/// Die Vorschausymbole eines Themas (32 px), gezeichnet mit einem eigenen
/// Symbolthema-Objekt, damit das Fenster selbst sein Thema behält.
fn symbole_von(thema: &str) -> Vec<gdk::Paintable> {
    let it = gtk4::IconTheme::new();
    it.set_theme_name(Some(thema));
    VORSCHAU
        .iter()
        .map(|n| {
            it.lookup_icon(
                n,
                &[],
                32,
                1,
                gtk4::TextDirection::Ltr,
                gtk4::IconLookupFlags::empty(),
            )
            .upcast()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labwc_konfig::Quelle;
    use std::collections::BTreeMap;

    fn stand(gs: &[(&str, &str)], env: &[(&str, &str)]) -> Stand {
        let m = |l: &[(&str, &str)]| {
            l.iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect::<BTreeMap<_, _>>()
        };
        Stand {
            rc: String::new(),
            quelle: Quelle::Benutzer,
            rc_vorgabe: String::new(),
            env: m(env),
            env_ohne_eigene: m(env),
            vorgabe_neu: false,
            fehler: None,
            gs: m(gs),
        }
    }

    const LIVE: &[(&str, &str)] = &[
        (SYMBOLTHEMA, "Querencia"),
        (GTK_THEMA, "Querencia-Klassisch"),
        (ZEIGERTHEMA, "DMZ-White"),
        (ZEIGERGROESSE, "24"),
        (FARBSCHEMA, "default"),
    ];

    #[test]
    fn lesen_und_aendern() {
        let st = stand(
            LIVE,
            &[("XCURSOR_THEME", "DMZ-White"), ("XCURSOR_SIZE", "24")],
        );
        let alt = Erscheinungsbild::lesen(&st);
        assert_eq!(alt.symbolthema, "Querencia");
        assert!(!alt.dunkel);
        assert!(Erscheinungsbild::gs_aenderungen(&alt, &alt).is_empty());
        let neu = Werte {
            symbolthema: "gnome-human".into(),
            zeigerthema: "DMZ-Black".into(),
            zeigergroesse: 32,
            dunkel: true,
            farbschema: "prefer-dark".into(),
            ..alt.clone()
        };
        assert_eq!(
            Erscheinungsbild::gs_aenderungen(&neu, &alt),
            vec![
                (SYMBOLTHEMA, "gnome-human".to_owned()),
                (ZEIGERTHEMA, "DMZ-Black".to_owned()),
                (ZEIGERGROESSE, "32".to_owned()),
                (FARBSCHEMA, "prefer-dark".to_owned()),
            ]
        );
        assert_eq!(
            Erscheinungsbild::env_aenderungen(&neu, &alt, &st),
            vec![
                ("XCURSOR_THEME", Some("DMZ-Black".to_owned())),
                ("XCURSOR_SIZE", Some("32".to_owned())),
            ]
        );
        // zurück auf die Vorgabe: eigene Angabe entfernen
        assert_eq!(
            Erscheinungsbild::env_aenderungen(&alt, &neu, &st),
            vec![("XCURSOR_THEME", None), ("XCURSOR_SIZE", None)]
        );
        // rc.xml bleibt unberührt
        assert_eq!(
            Erscheinungsbild::rc_aendern(&neu, &alt, "<labwc_config/>").unwrap(),
            "<labwc_config/>"
        );
    }

    #[test]
    fn standard_ist_querencia() {
        let mut st = stand(&[], &[]);
        st.gs = gs::standard(Erscheinungsbild::GSETTINGS);
        let w = Erscheinungsbild::lesen(&st);
        assert_eq!(
            (
                w.symbolthema.as_str(),
                w.gtk_thema.as_str(),
                w.zeigerthema.as_str(),
                w.zeigergroesse
            ),
            ("Querencia", "Querencia-Klassisch", "DMZ-White", 24)
        );
    }

    #[test]
    fn helles_schema_bleibt_ohne_aenderung() {
        let mut l = LIVE.to_vec();
        l[4] = (FARBSCHEMA, "prefer-light");
        let w = Erscheinungsbild::lesen(&stand(&l, &[]));
        assert!(!w.dunkel);
        assert_eq!(w.farbschema, "prefer-light");
    }

    #[test]
    fn nicht_installiert_bleibt_waehlbar() {
        let t = vec![Thema {
            name: "A".into(),
            anzeige: "A".into(),
        }];
        assert_eq!(mit_aktuellem(&t, "A").len(), 1);
        let l = mit_aktuellem(&t, "B");
        assert_eq!(l[1], ("B".to_owned(), "B (nicht installiert)".to_owned()));
    }
}
