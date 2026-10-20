//! Bereich „Sprache und Region“ (QD-056, Einstellungen 0.1.0): Anzeigesprache
//! und Formate der Sitzung. Geschrieben wird wie bei Tastatur und Zeiger die
//! eigene Umgebungsdatei der labwc-Benutzerkopie
//! (`environment.d/zz-querencia-einstellungen.env`, Baustein
//! [`crate::labwc_konfig`]); der Sitzungsstarter gibt die Werte bei der
//! Anmeldung an labwc und an die D-Bus-/systemd-Aktivierung weiter (QD-064).
//! Laufende Programme behalten ihre Umgebung, deshalb gilt alles erst ab der
//! nächsten Anmeldung; ein `labwc --reconfigure` bringt hier nichts.
//!
//! Aufteilung wie in GNOME: die Sprache wählt `LANGUAGE` (und `LANG`, damit
//! glibc `LANGUAGE` überhaupt beachtet – in der C-Locale ignoriert es sie),
//! die Formate `LC_TIME`, `LC_NUMERIC`, `LC_MONETARY`, `LC_PAPER` und
//! `LC_MEASUREMENT`. Die Querencia-Vorgabe setzt heute `LANG=en_US.UTF-8` und
//! `LANGUAGE=de`: Texte deutsch, Datum und Zahlen im US-Format (Befund
//! QD-056); die Seite zeigt das so an. „Standard wiederherstellen“ entfernt
//! die eigenen Angaben wieder.
//!
//! Angeboten werden Sprachen, für die dieses Programm einen Katalog hat und
//! eine Locale installiert ist, und als Formate alle installierten
//! UTF-8-Locales (`locale -a`). Die Beispiele entstehen aus `locale -k` der
//! jeweiligen Locale (das `date` der VM ist uutils und folgt der Locale nicht).
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use gtk4::prelude::*;

use crate::i18n::{self, gettext, gettext_f, gettext_noop};
use crate::labwc_konfig::{Stand, Wirkung};
use crate::labwc_seite::{self as ls, Bereich};
use crate::labwc_xml::Fehler;
use crate::xkb;

/// Variablen der Formate (gleiche Liste wie der Sitzungsstarter, QD-064).
pub const FORMAT_VARIABLEN: &[&str] = &[
    "LC_TIME",
    "LC_NUMERIC",
    "LC_MONETARY",
    "LC_PAPER",
    "LC_MEASUREMENT",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Werte {
    /// Anzeigesprache, z. B. `de`, `en`
    pub sprache: String,
    /// Locale der Formate, z. B. `de_DE.UTF-8`
    pub formate: String,
}

/// `de_DE.utf8` → `de_DE.UTF-8` (Schreibweise der Querencia-Vorgabe).
pub fn kanonisch(locale: &str) -> String {
    let l = locale.trim();
    match l.split_once('.') {
        Some((basis, cs)) => {
            let (cs, modifier) = cs.split_once('@').map_or((cs, ""), |(c, m)| (c, m));
            let cs = if cs.eq_ignore_ascii_case("utf8") || cs.eq_ignore_ascii_case("utf-8") {
                "UTF-8".to_owned()
            } else {
                cs.to_owned()
            };
            if modifier.is_empty() {
                format!("{basis}.{cs}")
            } else {
                format!("{basis}.{cs}@{modifier}")
            }
        }
        None => l.to_owned(),
    }
}

/// Sprachteil einer Locale oder einer `LANGUAGE`-Angabe (`de_AT.UTF-8` → `de`).
pub fn sprache_von(locale: &str) -> &str {
    locale.split(['_', '.', '@']).next().unwrap_or("").trim()
}

/// C-Locale (glibc übersetzt dann nichts, auch nicht über `LANGUAGE`).
fn ist_c(locale: &str) -> bool {
    matches!(sprache_von(locale), "" | "C" | "POSIX")
}

/// UTF-8-Locales mit Land aus der Ausgabe von `locale -a`, kanonisch, sortiert.
pub fn locales_aus(text: &str) -> Vec<String> {
    let mut l: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|z| {
            z.contains('_')
                && z.split_once('.').is_some_and(|(_, cs)| {
                    cs.eq_ignore_ascii_case("utf8") || cs.eq_ignore_ascii_case("utf-8")
                })
        })
        .map(kanonisch)
        .collect();
    l.sort();
    l.dedup();
    l
}

/// Die installierten UTF-8-Locales (`locale -a`).
pub fn installierte_locales() -> Vec<String> {
    std::process::Command::new("locale")
        .arg("-a")
        .env("LC_ALL", "C")
        .output()
        .map(|o| locales_aus(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default()
}

/// Die wirksamen Werte einer Umgebung (Reihenfolge wie glibc).
pub fn wirksam(env: &BTreeMap<String, String>) -> Werte {
    let g = |k: &str| env.get(k).map(String::as_str).filter(|v| !v.is_empty());
    let lang = g("LANG").unwrap_or("C");
    let lc_all = g("LC_ALL");
    let nachrichten = lc_all.or(g("LC_MESSAGES")).unwrap_or(lang);
    let sprache = if ist_c(nachrichten) {
        "en".to_owned()
    } else {
        g("LANGUAGE")
            .and_then(|l| l.split(':').map(sprache_von).find(|s| !s.is_empty()))
            .unwrap_or_else(|| sprache_von(nachrichten))
            .to_owned()
    };
    let formate = kanonisch(lc_all.or(g("LC_TIME")).unwrap_or(lang));
    Werte { sprache, formate }
}

/// `LANG` für eine Sprache: die Formate-Locale, wenn sie zur Sprache passt,
/// sonst die Hauptlocale der Sprache (`de_DE`, `en_US`, `fr_FR` …), sonst
/// irgendeine installierte, zuletzt die Formate selbst.
pub fn lang_fuer(sprache: &str, formate: &str, installiert: &[String]) -> String {
    if sprache_von(formate) == sprache {
        return formate.to_owned();
    }
    let haupt = match sprache {
        "en" => "en_US.UTF-8".to_owned(),
        s => format!("{s}_{}.UTF-8", s.to_ascii_uppercase()),
    };
    if installiert.contains(&haupt) {
        return haupt;
    }
    installiert
        .iter()
        .find(|l| sprache_von(l) == sprache)
        .cloned()
        .unwrap_or_else(|| formate.to_owned())
}

/// Die Umgebungsänderungen für `neu` (rein, für Tests): alle sieben
/// Variablen; gleich der Vorgabe → eigene Angabe entfernen.
pub fn aenderungen(
    neu: &Werte,
    st: &Stand,
    installiert: &[String],
) -> Vec<(&'static str, Option<String>)> {
    let mut alle = vec!["LANG", "LANGUAGE"];
    alle.extend_from_slice(FORMAT_VARIABLEN);
    if *neu == wirksam(&st.env_ohne_eigene) {
        return alle.into_iter().map(|k| (k, None)).collect();
    }
    let lang = lang_fuer(&neu.sprache, &neu.formate, installiert);
    let lc = if neu.formate == lang {
        ""
    } else {
        neu.formate.as_str()
    };
    let mut v = vec![
        ls::env_wert(st, "LANG", &lang),
        ls::env_wert(st, "LANGUAGE", &neu.sprache),
    ];
    v.extend(FORMAT_VARIABLEN.iter().map(|k| ls::env_wert(st, k, lc)));
    v
}

// ------------------------------------------------------------------ Namen

/// Namen aus den iso-codes-Tabellen (englisch) für Sprache und Land.
struct IsoNamen {
    sprachen: HashMap<String, String>,
    laender: HashMap<String, String>,
}

impl IsoNamen {
    fn laden() -> IsoNamen {
        let lesen = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
        IsoNamen::aus(
            &lesen("/usr/share/iso-codes/json/iso_639-2.json"),
            &lesen("/usr/share/iso-codes/json/iso_3166-1.json"),
        )
    }

    fn aus(sprachen: &str, laender: &str) -> IsoNamen {
        let mut sp = HashMap::new();
        for o in xkb::iso_json(sprachen) {
            if let (Some(c), Some(n)) = (o.get("alpha_2"), o.get("name")) {
                sp.insert(c.clone(), n.clone());
            }
        }
        let mut la = HashMap::new();
        for o in xkb::iso_json(laender) {
            if let (Some(c), Some(n)) = (o.get("alpha_2"), o.get("name")) {
                la.insert(c.clone(), n.clone());
            }
        }
        IsoNamen {
            sprachen: sp,
            laender: la,
        }
    }

    /// „Deutsch (Deutschland)“ in der Oberflächensprache; unbekannt → Locale.
    fn locale(&self, locale: &str, ue: xkb::Uebersetzer) -> String {
        let basis = locale.split(['.', '@']).next().unwrap_or(locale);
        let (s, l) = basis.split_once('_').unwrap_or((basis, ""));
        let sprache = self.sprachen.get(s).map(|en| {
            let t = ue("iso_639-2", en);
            t.split(';').next().unwrap_or(&t).trim().to_owned()
        });
        let land = self.laender.get(l).map(|en| ue("iso_3166-1", en));
        match (sprache, land) {
            (Some(s), Some(l)) => format!("{s} ({l})"),
            (Some(s), None) => s,
            _ => locale.to_owned(),
        }
    }
}

/// Name einer Anzeigesprache in ihr selbst (wie im Anmeldebildschirm), dazu
/// der Name in der Oberflächensprache, wenn er anders lautet.
pub fn sprachname(code: &str) -> String {
    let eigen = match code {
        "de" => Some("Deutsch"),
        "en" => Some("English"),
        "fr" => Some("Français"),
        "es" => Some("Español"),
        "it" => Some("Italiano"),
        "nl" => Some("Nederlands"),
        _ => None,
    };
    let iso = IsoNamen::laden();
    let hier = iso.sprachen.get(code).map(|en| {
        let t = xkb::dgettext("iso_639-2", en);
        t.split(';').next().unwrap_or(&t).trim().to_owned()
    });
    match (eigen, hier) {
        (Some(e), Some(h)) if e != h => format!("{e} – {h}"),
        (Some(e), _) => e.to_owned(),
        (None, Some(h)) => h,
        (None, None) => code.to_owned(),
    }
}

// --------------------------------------------------------------- Beispiele

/// Werte aus `locale -k` (Schlüssel → Wert ohne Anführungszeichen).
pub fn kategorien_aus(text: &str) -> HashMap<String, String> {
    text.lines()
        .filter_map(|z| z.split_once('='))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().trim_matches('"').to_owned()))
        .collect()
}

fn kategorien(locale: &str) -> HashMap<String, String> {
    std::process::Command::new("locale")
        .args([
            "-k",
            "LC_TIME",
            "LC_NUMERIC",
            "LC_MONETARY",
            "LC_PAPER",
            "LC_MEASUREMENT",
        ])
        .env("LC_ALL", locale)
        .env_remove("LANGUAGE")
        .output()
        .map(|o| kategorien_aus(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default()
}

/// Ganzzahl mit Tausendertrennung (Dreiergruppen).
fn gruppiert(n: u64, trenner: &str) -> String {
    let s = n.to_string();
    let mut aus = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            aus.push_str(trenner);
        }
        aus.push(c);
    }
    aus
}

/// Kleines strftime für die numerischen Felder, die `d_fmt`/`t_fmt` der
/// Locales benutzen (`%d %m %Y %y %H %M %S %I %p %T %R %r %D %F %e`).
pub fn zeit_formatieren(
    format: &str,
    (jahr, monat, tag, stunde, minute, sekunde): (i32, u32, u32, u32, u32, u32),
    am_pm: (&str, &str),
) -> String {
    let mut aus = String::new();
    let mut zeichen = format.chars();
    while let Some(c) = zeichen.next() {
        if c != '%' {
            aus.push(c);
            continue;
        }
        let h12 = if stunde % 12 == 0 { 12 } else { stunde % 12 };
        let pm = if stunde < 12 { am_pm.0 } else { am_pm.1 };
        match zeichen.next() {
            Some('d') => aus.push_str(&format!("{tag:02}")),
            Some('e') => aus.push_str(&format!("{tag:2}")),
            Some('m') => aus.push_str(&format!("{monat:02}")),
            Some('Y') => aus.push_str(&jahr.to_string()),
            Some('y') => aus.push_str(&format!("{:02}", jahr % 100)),
            Some('H') => aus.push_str(&format!("{stunde:02}")),
            Some('M') => aus.push_str(&format!("{minute:02}")),
            Some('S') => aus.push_str(&format!("{sekunde:02}")),
            Some('I') => aus.push_str(&format!("{h12:02}")),
            Some('p') => aus.push_str(pm),
            Some('T') => aus.push_str(&format!("{stunde:02}:{minute:02}:{sekunde:02}")),
            Some('R') => aus.push_str(&format!("{stunde:02}:{minute:02}")),
            Some('r') => aus.push_str(&format!("{h12:02}:{minute:02}:{sekunde:02} {pm}")),
            Some('D') => aus.push_str(&format!("{monat:02}/{tag:02}/{:02}", jahr % 100)),
            Some('F') => aus.push_str(&format!("{jahr}-{monat:02}-{tag:02}")),
            Some('%') => aus.push('%'),
            Some(x) => {
                aus.push('%');
                aus.push(x);
            }
            None => aus.push('%'),
        }
    }
    aus.trim().to_owned()
}

/// Beispielzeile für eine Locale aus ihren `locale -k`-Werten.
pub fn beispiel_aus(k: &HashMap<String, String>, jetzt: (i32, u32, u32, u32, u32, u32)) -> String {
    let w = |s: &str| k.get(s).map(String::as_str).unwrap_or("");
    let (am, pm) = w("am_pm").split_once(';').unwrap_or(("", ""));
    let datum = zeit_formatieren(w("d_fmt"), jetzt, (am, pm));
    let zeit = zeit_formatieren(w("t_fmt"), jetzt, (am, pm));
    let zahl = format!(
        "{}{}{}",
        gruppiert(1_234_567, w("thousands_sep")),
        w("decimal_point"),
        "89"
    );
    let betrag = format!(
        "{}{}{}",
        gruppiert(1_234, w("mon_thousands_sep")),
        w("mon_decimal_point"),
        "56"
    );
    let symbol = w("currency_symbol");
    let abstand = if w("p_sep_by_space") == "1" { " " } else { "" };
    let waehrung = if w("p_cs_precedes") == "1" {
        format!("{symbol}{abstand}{betrag}")
    } else {
        format!("{betrag}{abstand}{symbol}")
    };
    let papier = match (w("width"), w("height")) {
        ("210", "297") => "A4".to_owned(),
        ("216", "279") => "Letter".to_owned(),
        (b, h) => format!("{b} × {h} mm"),
    };
    let masse = if w("measurement") == "2" {
        gettext("US units")
    } else {
        gettext("metric")
    };
    gettext_f(
        "Date {date} · Time {time} · Number {number} · Currency {currency} · Paper {paper} · Units {units}",
        &[
            ("date", &datum),
            ("time", &zeit),
            ("number", &zahl),
            ("currency", &waehrung),
            ("paper", &papier),
            ("units", &masse),
        ],
    )
}

fn jetzt() -> (i32, u32, u32, u32, u32, u32) {
    gtk4::glib::DateTime::now_local().map_or((2026, 10, 20, 14, 30, 0), |d| {
        (
            d.year(),
            d.month() as u32,
            d.day_of_month() as u32,
            d.hour() as u32,
            d.minute() as u32,
            d.second() as u32,
        )
    })
}

// ------------------------------------------------------------------ Bereich

pub struct Sprache {
    sprachen_kasten: gtk4::Box,
    sprachen: Rc<RefCell<Vec<(String, gtk4::CheckButton)>>>,
    formate_raster: gtk4::Grid,
    formate: Rc<RefCell<Vec<(String, gtk4::CheckButton)>>>,
    beispiel: gtk4::Label,
    installiert: Vec<String>,
    geaendert: Rc<dyn Fn()>,
}

fn gewaehlt(k: &RefCell<Vec<(String, gtk4::CheckButton)>>) -> String {
    k.borrow()
        .iter()
        .find(|(_, k)| k.is_active())
        .map(|(s, _)| s.clone())
        .unwrap_or_default()
}

fn beispiel_zeigen(label: &gtk4::Label, locale: &str) {
    let k = kategorien(locale);
    if k.is_empty() || locale.is_empty() {
        label.set_text("");
    } else {
        label.set_text(&beispiel_aus(&k, jetzt()));
    }
}

/// Text zur laufenden Sitzung (Umgebung dieses Prozesses).
fn sitzung_text() -> String {
    let env: BTreeMap<String, String> = ["LANG", "LANGUAGE", "LC_ALL", "LC_MESSAGES", "LC_TIME"]
        .iter()
        .filter_map(|k| std::env::var(k).ok().map(|v| ((*k).to_owned(), v)))
        .collect();
    let w = wirksam(&env);
    let roh: Vec<String> = env.iter().map(|(k, v)| format!("{k}={v}")).collect();
    gettext_f(
        "This session: texts {language}, formats {formats} ({variables}).",
        &[
            ("language", &sprachname(&w.sprache)),
            (
                "formats",
                &IsoNamen::laden().locale(&w.formate, &xkb::dgettext),
            ),
            ("variables", &roh.join(", ")),
        ],
    )
}

impl Bereich for Sprache {
    type Werte = Werte;
    const NAME: &'static str = gettext_noop("Language and Region");
    const PROTOKOLL: &'static str = "Sprache und Region";
    const QUELLE: &'static str = gettext_noop(
        "~/.config/querencia/labwc/environment.d/zz-querencia-einstellungen.env (LANG, LANGUAGE, LC_TIME, LC_NUMERIC, LC_MONETARY, LC_PAPER, LC_MEASUREMENT; default in environment); languages from the catalogs of this program, formats from “locale -a”.",
    );
    const NEU_LADEN: bool = false;

    fn lesen(st: &Stand) -> Werte {
        wirksam(&st.env)
    }

    fn rc_aendern(_neu: &Werte, _alt: &Werte, rc: &str) -> Result<String, Fehler> {
        Ok(rc.to_owned())
    }

    fn env_aenderungen(
        neu: &Werte,
        alt: &Werte,
        st: &Stand,
    ) -> Vec<(&'static str, Option<String>)> {
        if neu == alt {
            return Vec::new();
        }
        aenderungen(neu, st, &installierte_locales())
    }

    fn hinweis(_w: &Wirkung, st: &Stand) -> Option<String> {
        let v = wirksam(&st.env_ohne_eigene);
        let roh = |k: &str| {
            st.env_ohne_eigene
                .get(k)
                .map_or_else(|| "-".to_owned(), String::clone)
        };
        Some(gettext_f(
            "Effective from the next login. Programs receive language and formats when they start; programs that are already running keep theirs. Querencia’s default: texts {language}, formats {formats} (LANGUAGE={language_variable}, LANG={lang_variable}).",
            &[
                ("language", &sprachname(&v.sprache)),
                (
                    "formats",
                    &IsoNamen::laden().locale(&v.formate, &xkb::dgettext),
                ),
                ("language_variable", &roh("LANGUAGE")),
                ("lang_variable", &roh("LANG")),
            ],
        ))
    }

    fn bauen(geaendert: Rc<dyn Fn()>) -> (Self, gtk4::Box) {
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

        let (f, b) = ls::rahmen(&gettext("Display Language"));
        let sprachen_kasten = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        b.append(&sprachen_kasten);
        b.append(&ls::grau(&gettext(
            "Language of menus and texts (LANGUAGE). Offered are languages with a translation for which a locale is installed; programs without a translation show English.",
        )));
        inhalt.append(&f);

        let (f, b) = ls::rahmen(&gettext("Formats"));
        let formate_raster = gtk4::Grid::new();
        formate_raster.set_column_spacing(12);
        formate_raster.set_row_spacing(2);
        b.append(&formate_raster);
        let beispiel = gtk4::Label::new(None);
        beispiel.set_xalign(0.0);
        beispiel.set_wrap(true);
        beispiel.set_max_width_chars(70);
        beispiel.set_selectable(true);
        beispiel.set_margin_top(4);
        b.append(&beispiel);
        b.append(&ls::grau(&gettext(
            "Date and time, numbers, currency, paper and units (LC_TIME, LC_NUMERIC, LC_MONETARY, LC_PAPER, LC_MEASUREMENT).",
        )));
        inhalt.append(&f);

        inhalt.append(&ls::grau(&sitzung_text()));

        let formate: Rc<RefCell<Vec<(String, gtk4::CheckButton)>>> =
            Rc::new(RefCell::new(Vec::new()));
        // Beim Wählen eines Formats das Beispiel auffrischen, dann melden.
        let mit_beispiel: Rc<dyn Fn()> = {
            let (f, l, g) = (formate.clone(), beispiel.clone(), geaendert.clone());
            Rc::new(move || {
                beispiel_zeigen(&l, &gewaehlt(&f));
                g();
            })
        };
        let s = Sprache {
            sprachen_kasten,
            sprachen: Rc::new(RefCell::new(Vec::new())),
            formate_raster,
            formate,
            beispiel,
            installiert: installierte_locales(),
            geaendert: mit_beispiel,
        };
        (s, inhalt)
    }

    fn zeigen(&self, w: &Werte) {
        // Sprachen: Kataloge dieses Programms mit installierter Locale
        let mut sprachen: Vec<(String, String)> = i18n::kataloge()
            .into_iter()
            .filter(|s| self.installiert.iter().any(|l| sprache_von(l) == s))
            .map(|s| {
                let n = sprachname(&s);
                (s, n)
            })
            .collect();
        if !w.sprache.is_empty() && !sprachen.iter().any(|(s, _)| *s == w.sprache) {
            sprachen.push((
                w.sprache.clone(),
                gettext_f(
                    "{language} (no translation)",
                    &[("language", &sprachname(&w.sprache))],
                ),
            ));
        }
        ls::optionen(
            &self.sprachen_kasten,
            &self.sprachen,
            &sprachen,
            &w.sprache,
            &self.geaendert,
        );
        let iso = IsoNamen::laden();
        let mut formate: Vec<(String, String)> = self
            .installiert
            .iter()
            .map(|l| (l.clone(), iso.locale(l, &xkb::dgettext)))
            .collect();
        formate.sort_by_cached_key(|(_, n)| n.to_lowercase());
        if !formate.iter().any(|(l, _)| *l == w.formate) {
            formate.push((
                w.formate.clone(),
                gettext_f("{locale} (not installed)", &[("locale", &w.formate)]),
            ));
        }
        ls::optionen_raster(
            &self.formate_raster,
            &self.formate,
            &formate,
            &w.formate,
            &self.geaendert,
            2,
        );
        beispiel_zeigen(&self.beispiel, &w.formate);
    }

    fn aktuell(&self) -> Werte {
        Werte {
            sprache: gewaehlt(&self.sprachen),
            formate: gewaehlt(&self.formate),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labwc_konfig::Quelle;

    fn env(l: &[(&str, &str)]) -> BTreeMap<String, String> {
        l.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn stand(eigen: &[(&str, &str)], vorgabe: &[(&str, &str)]) -> Stand {
        let mut e = env(vorgabe);
        e.extend(env(eigen));
        Stand {
            rc: String::new(),
            quelle: Quelle::Benutzer,
            rc_vorgabe: String::new(),
            env: e,
            env_ohne_eigene: env(vorgabe),
            vorgabe_neu: false,
            fehler: None,
            gs: BTreeMap::new(),
        }
    }

    const VORGABE: &[(&str, &str)] = &[("LANG", "en_US.UTF-8"), ("LANGUAGE", "de")];

    fn installiert() -> Vec<String> {
        locales_aus("C\nC.utf8\nde_AT.utf8\nde_CH.utf8\nde_DE.utf8\nen_US.utf8\nPOSIX\n")
    }

    #[test]
    fn locales_kanonisch_und_ohne_c() {
        assert_eq!(
            installiert(),
            vec!["de_AT.UTF-8", "de_CH.UTF-8", "de_DE.UTF-8", "en_US.UTF-8"]
        );
        assert_eq!(kanonisch("sr_RS.utf8@latin"), "sr_RS.UTF-8@latin");
        assert_eq!(kanonisch("C"), "C");
    }

    #[test]
    fn befund_der_vorgabe_texte_deutsch_formate_us() {
        let w = wirksam(&env(VORGABE));
        assert_eq!(
            w,
            Werte {
                sprache: "de".into(),
                formate: "en_US.UTF-8".into()
            }
        );
        // C-Locale: glibc beachtet LANGUAGE nicht
        let c = wirksam(&env(&[("LANG", "C.UTF-8"), ("LANGUAGE", "de")]));
        assert_eq!(c.sprache, "en");
        // LC_TIME vor LANG, LC_ALL vor allem
        let f = wirksam(&env(&[("LANG", "en_US.UTF-8"), ("LC_TIME", "de_DE.utf8")]));
        assert_eq!(f.formate, "de_DE.UTF-8");
        assert_eq!(f.sprache, "en");
        let a = wirksam(&env(&[("LANG", "en_US.UTF-8"), ("LC_ALL", "de_AT.UTF-8")]));
        assert_eq!(
            (a.sprache.as_str(), a.formate.as_str()),
            ("de", "de_AT.UTF-8")
        );
    }

    #[test]
    fn deutsch_mit_deutschen_formaten() {
        let st = stand(&[], VORGABE);
        let neu = Werte {
            sprache: "de".into(),
            formate: "de_DE.UTF-8".into(),
        };
        let a = aenderungen(&neu, &st, &installiert());
        let gesetzt: Vec<_> = a.iter().filter(|(_, v)| v.is_some()).collect();
        assert_eq!(gesetzt, vec![&("LANG", Some("de_DE.UTF-8".into()))]);
        // LANGUAGE gleich der Vorgabe, keine LC_*: alles entfernt
        assert!(a.contains(&("LANGUAGE", None)));
        assert!(a.contains(&("LC_TIME", None)));
    }

    #[test]
    fn englisch_mit_deutschen_formaten() {
        let st = stand(&[], VORGABE);
        let neu = Werte {
            sprache: "en".into(),
            formate: "de_CH.UTF-8".into(),
        };
        let a = aenderungen(&neu, &st, &installiert());
        assert!(a.contains(&("LANG", None)), "{a:?}");
        assert!(a.contains(&("LANGUAGE", Some("en".into()))));
        for k in FORMAT_VARIABLEN {
            assert!(a.contains(&(k, Some("de_CH.UTF-8".into()))), "{k}");
        }
        // geschrieben und wieder gelesen ergibt dieselbe Wahl
        let mut e = env(VORGABE);
        for (k, v) in &a {
            if let Some(v) = v {
                e.insert((*k).to_owned(), v.clone());
            }
        }
        assert_eq!(wirksam(&e), neu);
    }

    #[test]
    fn standard_entfernt_alles() {
        let st = stand(
            &[("LANG", "de_DE.UTF-8"), ("LC_TIME", "de_AT.UTF-8")],
            VORGABE,
        );
        let neu = wirksam(&st.env_ohne_eigene);
        let a = aenderungen(&neu, &st, &installiert());
        assert_eq!(a.len(), 7);
        assert!(a.iter().all(|(_, v)| v.is_none()));
    }

    #[test]
    fn lang_passend_zur_sprache() {
        let i = installiert();
        assert_eq!(lang_fuer("de", "de_AT.UTF-8", &i), "de_AT.UTF-8");
        assert_eq!(lang_fuer("de", "en_US.UTF-8", &i), "de_DE.UTF-8");
        assert_eq!(lang_fuer("en", "de_DE.UTF-8", &i), "en_US.UTF-8");
        assert_eq!(lang_fuer("fr", "de_DE.UTF-8", &i), "de_DE.UTF-8");
    }

    #[test]
    fn beispiele_aus_locale_k() {
        let de = kategorien_aus(
            "d_fmt=\"%d.%m.%Y\"\nt_fmt=\"%T\"\nam_pm=\";\"\ndecimal_point=\",\"\nthousands_sep=\".\"\ncurrency_symbol=\"€\"\np_cs_precedes=0\np_sep_by_space=1\nmon_decimal_point=\",\"\nmon_thousands_sep=\".\"\nheight=297\nwidth=210\nmeasurement=1\n",
        );
        let t = (2026, 10, 20, 14, 5, 9);
        assert_eq!(
            beispiel_aus(&de, t),
            "Date 20.10.2026 · Time 14:05:09 · Number 1.234.567,89 · Currency 1.234,56 € · Paper A4 · Units metric"
        );
        let us = kategorien_aus(
            "d_fmt=\"%m/%d/%Y\"\nt_fmt=\"%r\"\nam_pm=\"AM;PM\"\ndecimal_point=\".\"\nthousands_sep=\",\"\ncurrency_symbol=\"$\"\np_cs_precedes=1\np_sep_by_space=0\nmon_decimal_point=\".\"\nmon_thousands_sep=\",\"\nheight=279\nwidth=216\nmeasurement=2\n",
        );
        assert_eq!(
            beispiel_aus(&us, t),
            "Date 10/20/2026 · Time 02:05:09 PM · Number 1,234,567.89 · Currency $1,234.56 · Paper Letter · Units US units"
        );
    }

    #[test]
    fn localenamen_aus_iso_codes() {
        let iso = IsoNamen::aus(
            "{\n  \"639-2\": [\n    {\n      \"alpha_2\": \"de\",\n      \"alpha_3\": \"deu\",\n      \"name\": \"German\"\n    }\n  ]\n}\n",
            "{\n  \"3166-1\": [\n    {\n      \"alpha_2\": \"CH\",\n      \"name\": \"Switzerland\"\n    }\n  ]\n}\n",
        );
        let ue = |d: &str, t: &str| match (d, t) {
            ("iso_639-2", "German") => "Deutsch".to_owned(),
            ("iso_3166-1", "Switzerland") => "Schweiz".to_owned(),
            _ => t.to_owned(),
        };
        assert_eq!(iso.locale("de_CH.UTF-8", &ue), "Deutsch (Schweiz)");
        assert_eq!(iso.locale("de_XX.UTF-8", &ue), "Deutsch");
        assert_eq!(iso.locale("xx_YY.UTF-8", &ue), "xx_YY.UTF-8");
    }

    #[test]
    fn systemlocales_haben_beispiele() {
        // auf der VM installiert: de_DE und en_US (nur prüfen, wenn vorhanden)
        let l = installierte_locales();
        if l.iter().any(|x| x == "de_DE.UTF-8") {
            let k = kategorien("de_DE.UTF-8");
            assert_eq!(k.get("decimal_point").map(String::as_str), Some(","));
            assert!(beispiel_aus(&k, (2026, 1, 2, 3, 4, 5)).contains("02.01.2026"));
        }
    }
}
