//! Bereich „Fensterdesign“ (G2-AG, QD-046 ⑤): labwc-Thema, Eckenradius,
//! Schatten, Titelleistenknöpfe und – weil es nur rc.xml betrifft – die
//! Schrift der Titelleisten (`<theme><font place="ActiveWindow">` und
//! `InactiveWindow`). Alles wirkt mit `labwc --reconfigure` sofort.
use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{glib, pango};

use crate::i18n::{gettext, gettext_f, gettext_noop};
use crate::labwc_konfig::{Stand, Wirkung};
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::{self as lx, Fehler};
use crate::tastatur::ja;

/// Titelleistenknöpfe (labwc `<theme><titlebar><layout>`), Standard zuerst.
pub const KNOEPFE: &[(&str, &str)] = &[
    (
        "icon:iconify,max,close",
        gettext_noop("Program icon on the left; minimize, maximize, close on the right (default)"),
    ),
    (
        "menu:iconify,max,close",
        gettext_noop("Window menu on the left; minimize, maximize, close on the right"),
    ),
    (
        "close,iconify,max:",
        gettext_noop("Close, minimize, maximize on the left"),
    ),
    ("icon:close", gettext_noop("Close only")),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    /// leer = labwcs eingebautes Thema
    pub thema: String,
    pub eckenradius: u32,
    pub schatten: bool,
    pub knoepfe: String,
    pub schrift: String,
    /// Punkte
    pub schriftgroesse: u32,
}

/// Installierte labwc-Themen (`themes/<name>/labwc/themerc`).
pub fn themen() -> Vec<String> {
    let mut ordner = vec![
        glib::user_data_dir().join("themes"),
        glib::home_dir().join(".themes"),
    ];
    ordner.extend(
        glib::system_data_dirs()
            .into_iter()
            .map(|d| d.join("themes")),
    );
    let mut liste = Vec::new();
    for d in ordner {
        if let Ok(e) = std::fs::read_dir(&d) {
            for t in e.flatten() {
                if t.path().join("labwc").join("themerc").is_file() {
                    liste.push(t.file_name().to_string_lossy().into_owned());
                }
            }
        }
    }
    liste.sort();
    liste.dedup();
    liste
}

/// Der Querencia-Rahmen folgt dem Farbschema (0.1.6, Entscheidung der
/// Startsession): Schaltet „Erscheinungsbild“ die dunkle Darstellung um,
/// wechselt `<theme><name>` zwischen diesen beiden Namen – aber nur, wenn
/// rc.xml gerade einen davon nennt. Einen anderen Rahmen lässt die
/// Umschaltung stehen; ein Kennzeichen „vom Benutzer gewählt“ gibt es nicht.
pub const RAHMEN_HELL: &str = "Querencia-Klassisch";
pub const RAHMEN_DUNKEL: &str = "Querencia-Klassisch-Dunkel";

/// Neuer Rahmenname für `ist` (Wert von `<theme><name>`) nach dem
/// Umschalten auf `dunkel`; `None` = nichts ändern. Ohne installiertes
/// dunkles Thema bleibt der helle Name (labwc fiele sonst auf sein
/// eingebautes Thema zurück).
pub fn rahmen_fuer(
    ist: Option<&str>,
    dunkel: bool,
    dunkel_vorhanden: bool,
) -> Option<&'static str> {
    match ist.map(str::trim) {
        Some(RAHMEN_HELL) if dunkel && dunkel_vorhanden => Some(RAHMEN_DUNKEL),
        Some(RAHMEN_DUNKEL) if !dunkel => Some(RAHMEN_HELL),
        _ => None,
    }
}

/// rc.xml nach dem Umschalten auf `dunkel` (nur der Rahmenname, siehe
/// [`rahmen_fuer`]); das Zurückstellen gleicht danach
/// [`lx::angleichen`] byte-genau an die Basis an.
pub fn rahmen_umstellen(rc: &str, dunkel: bool, dunkel_vorhanden: bool) -> Result<String, Fehler> {
    match rahmen_fuer(
        lx::wert(rc, lx::THEMA, "name").as_deref(),
        dunkel,
        dunkel_vorhanden,
    ) {
        Some(name) => lx::setzen(rc, lx::THEMA, "name", name),
        None => Ok(rc.to_owned()),
    }
}

#[cfg(test)]
thread_local! {
    /// Prüfungen: ob [`dunkler_rahmen_vorhanden`] das dunkle Thema findet.
    pub static DUNKEL_VORHANDEN: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// Ist [`RAHMEN_DUNKEL`] als labwc-Thema installiert (dieselben Ordner wie
/// [`themen`]: ~/.local/share/themes, ~/.themes, die Systemordner)?
pub fn dunkler_rahmen_vorhanden() -> bool {
    #[cfg(test)]
    if let Some(v) = DUNKEL_VORHANDEN.with(std::cell::Cell::get) {
        return v;
    }
    themen().iter().any(|t| t == RAHMEN_DUNKEL)
}

pub struct Fensterdesign {
    themen_kasten: gtk4::Box,
    themen: RefCell<Vec<(String, gtk4::CheckButton)>>,
    eckenradius: gtk4::SpinButton,
    schatten: gtk4::CheckButton,
    knoepfe_kasten: gtk4::Box,
    knoepfe: RefCell<Vec<(String, gtk4::CheckButton)>>,
    schrift: gtk4::FontDialogButton,
    geaendert: Rc<dyn Fn()>,
}

impl Bereich for Fensterdesign {
    type Werte = Werte;
    const NAME: &'static str = gettext_noop("Window Design");
    const PROTOKOLL: &'static str = "Fensterdesign";
    const QUELLE: &'static str = gettext_noop(
        "~/.config/querencia/labwc/rc.xml <theme> (name, cornerRadius, dropShadows, titlebar/layout, font place=\"ActiveWindow\"/\"InactiveWindow\"); themes in ~/.local/share/themes/*/labwc/themerc and /usr/share/themes.",
    );

    const GSETTINGS: &'static [&'static str] = &[crate::gsettings::FARBSCHEMA];

    fn hinweis(w: &Wirkung, st: &Stand) -> Option<String> {
        let mut t = crate::labwc_konfig::wirkung_text(w, st);
        if st.gs.get(crate::gsettings::FARBSCHEMA).map(String::as_str) == Some("prefer-dark") {
            t.push(' ');
            t.push_str(&gettext(
                "Dark appearance is on: the Querencia window frame follows the color scheme (Querencia-Klassisch-Dunkel when dark, Querencia-Klassisch when light); other frames stay as chosen.",
            ));
        }
        Some(t)
    }

    fn lesen(st: &Stand) -> Werte {
        let t = |k: &str| lx::wert(&st.rc, lx::THEMA, k);
        let schrift = |k: &str| {
            lx::wert(&st.rc, lx::SCHRIFT_AKTIV, k).or_else(|| lx::wert(&st.rc, lx::SCHRIFT_ALLE, k))
        };
        Werte {
            thema: t("name").unwrap_or_default(),
            eckenradius: t("cornerRadius").and_then(|v| v.parse().ok()).unwrap_or(8),
            schatten: ja(t("dropShadows")),
            knoepfe: lx::wert(&st.rc, lx::TITELLEISTE, "layout")
                .unwrap_or_else(|| KNOEPFE[0].0.to_owned()),
            schrift: schrift("name").unwrap_or_else(|| "sans".into()),
            schriftgroesse: schrift("size").and_then(|v| v.parse().ok()).unwrap_or(10),
        }
    }

    fn rc_aendern(neu: &Werte, alt: &Werte, rc: &str) -> Result<String, Fehler> {
        let mut t = rc.to_owned();
        if neu.thema != alt.thema && !neu.thema.is_empty() {
            t = lx::setzen(&t, lx::THEMA, "name", &neu.thema)?;
        }
        if neu.eckenradius != alt.eckenradius {
            t = lx::setzen(&t, lx::THEMA, "cornerRadius", &neu.eckenradius.to_string())?;
        }
        if neu.schatten != alt.schatten {
            t = lx::setzen(
                &t,
                lx::THEMA,
                "dropShadows",
                if neu.schatten { "yes" } else { "no" },
            )?;
        }
        if neu.knoepfe != alt.knoepfe {
            t = lx::setzen(&t, lx::TITELLEISTE, "layout", &neu.knoepfe)?;
        }
        if (&neu.schrift, neu.schriftgroesse) != (&alt.schrift, alt.schriftgroesse) {
            for ort in [lx::SCHRIFT_AKTIV, lx::SCHRIFT_INAKTIV] {
                t = lx::setzen(&t, ort, "name", &neu.schrift)?;
                t = lx::setzen(&t, ort, "size", &neu.schriftgroesse.to_string())?;
            }
        }
        Ok(t)
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        let (f, b) = ls::rahmen(&gettext("Window Frame Theme"));
        let themen_kasten = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        b.append(&themen_kasten);
        inhalt.append(&f);

        let (f, b) = ls::rahmen(&gettext("Shape"));
        let eckenradius = gtk4::SpinButton::with_range(0.0, 20.0, 1.0);
        b.append(&ls::zeile(
            &gettext("Top corner radius (pixels):"),
            &eckenradius,
        ));
        let schatten = gtk4::CheckButton::with_label(&gettext("Shadows behind windows"));
        b.append(&schatten);
        inhalt.append(&f);

        let (f, b) = ls::rahmen(&gettext("Title Bar Buttons"));
        let knoepfe_kasten = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        b.append(&knoepfe_kasten);
        inhalt.append(&f);

        let (f, b) = ls::rahmen(&gettext("Title Bar Font"));
        let schrift = gtk4::FontDialogButton::new(Some(
            gtk4::FontDialog::builder()
                .title(gettext("Title Bar Font"))
                .modal(true)
                .build(),
        ));
        schrift.set_halign(gtk4::Align::Start);
        b.append(&schrift);
        b.append(&ls::grau(&gettext(
            "For active and inactive windows. The other fonts are in the “Fonts” section.",
        )));
        inhalt.append(&f);

        let d = Fensterdesign {
            themen_kasten,
            themen: RefCell::new(Vec::new()),
            eckenradius,
            schatten,
            knoepfe_kasten,
            knoepfe: RefCell::new(Vec::new()),
            schrift,
            geaendert: geaendert.clone(),
        };
        let g = geaendert.clone();
        d.eckenradius.connect_value_changed(move |_| g());
        let g = geaendert.clone();
        d.schatten.connect_toggled(move |_| g());
        let g = geaendert.clone();
        d.schrift.connect_font_desc_notify(move |_| g());
        (d, inhalt)
    }

    fn zeigen(&self, w: &Werte) {
        let mut themen = themen();
        if !themen.contains(&w.thema) {
            themen.push(w.thema.clone());
        }
        let beschriftung = |t: &String| {
            if t.is_empty() {
                gettext("Built-in labwc theme")
            } else if t == &w.thema && !self::themen().contains(t) {
                gettext_f("{theme} (not installed)", &[("theme", t)])
            } else {
                t.clone()
            }
        };
        let liste: Vec<(String, String)> = themen
            .iter()
            .map(|t| (t.clone(), beschriftung(t)))
            .collect();
        ls::optionen(
            &self.themen_kasten,
            &self.themen,
            &liste,
            &w.thema,
            &self.geaendert,
        );
        self.eckenradius.set_value(w.eckenradius as f64);
        self.schatten.set_active(w.schatten);
        let mut knoepfe: Vec<(String, String)> = KNOEPFE
            .iter()
            .map(|(k, t)| ((*k).to_owned(), gettext(*t)))
            .collect();
        if !knoepfe.iter().any(|(k, _)| k == &w.knoepfe) {
            knoepfe.push((
                w.knoepfe.clone(),
                gettext_f("Custom arrangement: {layout}", &[("layout", &w.knoepfe)]),
            ));
        }
        ls::optionen(
            &self.knoepfe_kasten,
            &self.knoepfe,
            &knoepfe,
            &w.knoepfe,
            &self.geaendert,
        );
        let mut fd = pango::FontDescription::from_string(&w.schrift);
        fd.set_size(w.schriftgroesse as i32 * pango::SCALE);
        self.schrift.set_font_desc(&fd);
    }

    fn aktuell(&self) -> Werte {
        let gewaehlt = |l: &RefCell<Vec<(String, gtk4::CheckButton)>>| {
            l.borrow()
                .iter()
                .find(|(_, k)| k.is_active())
                .map(|(t, _)| t.clone())
                .unwrap_or_default()
        };
        let fd = self.schrift.font_desc();
        Werte {
            thema: gewaehlt(&self.themen),
            eckenradius: self.eckenradius.value_as_int().max(0) as u32,
            schatten: self.schatten.is_active(),
            knoepfe: gewaehlt(&self.knoepfe),
            schrift: fd
                .as_ref()
                .and_then(|f| f.family())
                .map_or_else(|| "sans".into(), |f| f.to_string()),
            schriftgroesse: fd
                .as_ref()
                .map(|f| f.size() / pango::SCALE)
                .filter(|s| *s > 0)
                .unwrap_or(10) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labwc_konfig::Quelle;
    use std::collections::BTreeMap;

    fn stand(rc: &str) -> Stand {
        Stand {
            rc: rc.into(),
            quelle: Quelle::Benutzer,
            rc_vorgabe: rc.into(),
            rc_basis: rc.into(),
            env: BTreeMap::new(),
            env_ohne_eigene: BTreeMap::new(),
            vorgabe_neu: false,
            fehler: None,
            gs: BTreeMap::new(),
        }
    }

    const VORGABE: &str = "<labwc_config>\n  <theme>\n    <name>Querencia-Klassisch</name>\n    <font place=\"OnScreenDisplay\">\n      <name>Adwaita Sans</name>\n      <size>11</size>\n    </font>\n  </theme>\n</labwc_config>\n";

    #[test]
    fn lesen_vorgabe() {
        let w = Fensterdesign::lesen(&stand(VORGABE));
        assert_eq!(
            w,
            Werte {
                thema: "Querencia-Klassisch".into(),
                eckenradius: 8,
                schatten: false,
                knoepfe: "icon:iconify,max,close".into(),
                schrift: "sans".into(),
                schriftgroesse: 10
            }
        );
    }

    #[test]
    fn schreiben_rundlauf() {
        let st = stand(VORGABE);
        let alt = Fensterdesign::lesen(&st);
        let neu = Werte {
            thema: "Andere".into(),
            eckenradius: 0,
            schatten: true,
            knoepfe: "icon:close".into(),
            schrift: "DejaVu Sans".into(),
            schriftgroesse: 12,
        };
        let rc = Fensterdesign::rc_aendern(&neu, &alt, &st.rc).unwrap();
        assert_eq!(Fensterdesign::lesen(&stand(&rc)), neu);
        assert_eq!(
            lx::wert(&rc, lx::SCHRIFT_INAKTIV, "size").as_deref(),
            Some("12")
        );
        // OSD-Schrift bleibt
        assert!(rc.contains("<font place=\"OnScreenDisplay\">\n      <name>Adwaita Sans</name>\n      <size>11</size>"));
        // leeres Thema wird nie geschrieben
        let rc2 = Fensterdesign::rc_aendern(
            &Werte {
                thema: String::new(),
                ..alt.clone()
            },
            &alt,
            &st.rc,
        )
        .unwrap();
        assert_eq!(rc2, st.rc);
    }
}

#[cfg(test)]
mod rahmen_tests {
    use super::*;

    #[test]
    fn rahmen_folgt_nur_den_querencia_namen() {
        // hell → dunkel nur mit installiertem dunklem Thema
        assert_eq!(
            rahmen_fuer(Some(RAHMEN_HELL), true, true),
            Some(RAHMEN_DUNKEL)
        );
        assert_eq!(rahmen_fuer(Some(RAHMEN_HELL), true, false), None);
        assert_eq!(rahmen_fuer(Some(RAHMEN_HELL), false, true), None);
        // dunkel → hell immer (auch ohne dunkles Thema)
        assert_eq!(
            rahmen_fuer(Some(RAHMEN_DUNKEL), false, false),
            Some(RAHMEN_HELL)
        );
        assert_eq!(rahmen_fuer(Some(RAHMEN_DUNKEL), true, true), None);
        assert_eq!(
            rahmen_fuer(Some(" Querencia-Klassisch "), true, true),
            Some(RAHMEN_DUNKEL)
        );
        // fremde Rahmen und labwcs eingebautes Thema bleiben
        for ist in [Some("Anders"), Some("querencia-klassisch"), None] {
            for (d, v) in [(true, true), (false, true), (true, false)] {
                assert_eq!(rahmen_fuer(ist, d, v), None, "{ist:?} {d} {v}");
            }
        }
    }
}
