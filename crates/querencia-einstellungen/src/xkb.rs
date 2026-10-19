//! Tastaturbelegungen (XKB) mit deutschen Namen für den Bereich „Tastatur“
//! (G2-AG, QD-057).
//!
//! Quelle ist `/usr/share/X11/xkb/rules/evdev.xml` (xkb-data). Deutsche
//! Übersetzungen von xkeyboard-config sind auf dem System nicht installiert;
//! die Namen entstehen deshalb aus Sprache und Land der Belegung über die
//! iso-codes-Übersetzungen (gettext-Domänen `iso_639-2`, `iso_3166-1`, z. B.
//! „Deutsch (Österreich)“). Varianten bekommen eine kleine eigene Tabelle
//! für die häufigen Zusätze („ohne Akzenttasten“), sonst bleibt der englische
//! Zusatz stehen.
use std::collections::HashMap;

pub const REGELN: &str = "/usr/share/X11/xkb/rules/evdev.xml";
const SPRACHEN: &str = "/usr/share/iso-codes/json/iso_639-2.json";
const LAENDER: &str = "/usr/share/iso-codes/json/iso_3166-1.json";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Belegung {
    pub layout: String,
    /// leer = Grundbelegung
    pub variante: String,
    pub name: String,
}

/// Übersetzung (Domäne, englischer Text) → Text; in der App gettext.
pub type Uebersetzer<'a> = &'a dyn Fn(&str, &str) -> String;

pub fn gettext(domaene: &str, text: &str) -> String {
    gtk4::glib::dgettext(Some(domaene), text).to_string()
}

/// Objekte einer iso-codes-JSON-Datei (eine Angabe je Zeile) als Schlüssel→Wert.
fn iso_json(text: &str) -> Vec<HashMap<String, String>> {
    let mut liste = Vec::new();
    let mut akt = HashMap::new();
    for zeile in text.lines() {
        let z = zeile.trim().trim_end_matches(',');
        if z.starts_with('}') {
            if !akt.is_empty() {
                liste.push(std::mem::take(&mut akt));
            }
        } else if let Some((k, v)) = z.split_once("\": \"") {
            let k = k.trim_start_matches('"');
            let v = v.trim_end_matches('"').replace("\\\"", "\"");
            akt.insert(k.to_owned(), v);
        }
    }
    liste
}

/// Zusätze der Varianten (durch Komma getrennte Teile in Klammern).
const ZUSAETZE: &[(&str, &str)] = &[
    ("no dead keys", "ohne Akzenttasten"),
    ("with dead keys", "mit Akzenttasten"),
    ("intl.", "international"),
    ("alt. intl.", "international, alternativ"),
    ("with AltGr dead keys", "Akzente über AltGr"),
    ("alt.", "alternativ"),
    ("legacy", "veraltet"),
    ("extended", "erweitert"),
    ("dead acute", "Akzenttaste Akut"),
    ("dead grave acute", "Akzenttasten Gravis und Akut"),
    ("dead tilde", "Akzenttaste Tilde"),
    ("euro on 5", "Euro auf 5"),
    ("phonetic", "phonetisch"),
    ("one-handed", "einhändig"),
    ("left", "links"),
    ("right", "rechts"),
    ("Latin-9 only", "nur Latin-9"),
    ("programmer Dvorak", "Dvorak für Programmierer"),
    ("classic Dvorak", "klassisches Dvorak"),
    ("US", "USA"),
    ("UK", "Vereinigtes Königreich"),
    ("Germany", "Deutschland"),
    ("Austria", "Österreich"),
    ("Switzerland", "Schweiz"),
    ("France", "Frankreich"),
];

fn zusatz_deutsch(teil: &str) -> String {
    ZUSAETZE
        .iter()
        .find(|(en, _)| en.eq_ignore_ascii_case(teil.trim()))
        .map_or_else(|| teil.trim().to_owned(), |(_, de)| (*de).to_owned())
}

fn klammer(beschreibung: &str) -> Option<&str> {
    let a = beschreibung.find('(')?;
    let e = beschreibung.rfind(')')?;
    (e > a).then(|| &beschreibung[a + 1..e])
}

struct Namen<'a> {
    sprachen: HashMap<String, String>,
    laender: HashMap<String, String>,
    ue: Uebersetzer<'a>,
}

impl Namen<'_> {
    fn sprache(&self, code: &str) -> Option<String> {
        self.sprachen.get(code).map(|en| {
            // „Spanish; Castilian“ → erster Name
            let de = (self.ue)("iso_639-2", en);
            de.split(';').next().unwrap_or(&de).trim().to_owned()
        })
    }
    fn land(&self, code: &str) -> Option<String> {
        self.laender.get(code).map(|en| (self.ue)("iso_3166-1", en))
    }
}

fn kind_text(n: roxmltree::Node, name: &str) -> Option<String> {
    n.children()
        .find(|c| c.has_tag_name("configItem"))?
        .children()
        .find(|c| c.has_tag_name(name))?
        .text()
        .map(str::to_owned)
}

fn codes(n: roxmltree::Node, liste: &str, eintrag: &str) -> Vec<String> {
    n.children()
        .find(|c| c.has_tag_name("configItem"))
        .and_then(|ci| ci.children().find(|c| c.has_tag_name(liste)))
        .map(|l| {
            l.children()
                .filter(|c| c.has_tag_name(eintrag))
                .filter_map(|c| c.text().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Alle Belegungen und Varianten, nach deutschem Namen sortiert.
pub fn liste_aus(regeln: &str, sprachen: &str, laender: &str, ue: Uebersetzer) -> Vec<Belegung> {
    let mut sp = HashMap::new();
    for o in iso_json(sprachen) {
        if let Some(name) = o.get("name") {
            for k in ["alpha_3", "bibliographic"] {
                if let Some(c) = o.get(k) {
                    sp.insert(c.clone(), name.clone());
                }
            }
        }
    }
    let mut la = HashMap::new();
    for o in iso_json(laender) {
        if let (Some(c), Some(name)) = (o.get("alpha_2"), o.get("name")) {
            la.insert(c.clone(), name.clone());
        }
    }
    let namen = Namen {
        sprachen: sp,
        laender: la,
        ue,
    };
    // evdev.xml beginnt mit <!DOCTYPE xkbConfigRegistry SYSTEM "xkb.dtd">
    let optionen = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let Ok(doc) = roxmltree::Document::parse_with_options(regeln, optionen) else {
        return Vec::new();
    };
    let mut liste = Vec::new();
    let Some(layouts) = doc
        .root_element()
        .children()
        .find(|c| c.has_tag_name("layoutList"))
    else {
        return liste;
    };
    for l in layouts.children().filter(|c| c.has_tag_name("layout")) {
        let (Some(code), Some(beschr)) = (kind_text(l, "name"), kind_text(l, "description")) else {
            continue;
        };
        let sprache = codes(l, "languageList", "iso639Id")
            .first()
            .and_then(|c| namen.sprache(c));
        let land = codes(l, "countryList", "iso3166Id")
            .first()
            .and_then(|c| namen.land(c));
        // Sprachwort der englischen Beschreibung, z. B. „German“
        let wort = beschr.split(" (").next().unwrap_or(&beschr).to_owned();
        // Beschreibung nennt die Sprache („German“) oder ein Land („Belgian“)?
        let sprache_en = codes(l, "languageList", "iso639Id")
            .first()
            .and_then(|c| namen.sprachen.get(c))
            .map(|en| en.split(';').next().unwrap_or(en).trim().to_owned());
        let nennt_sprache = sprache_en.as_deref() == Some(wort.as_str());
        let grundname = match (&sprache, klammer(&beschr), &land) {
            (Some(s), Some(_), Some(l)) if nennt_sprache => format!("{s} ({l})"),
            (Some(s), None, _) if nennt_sprache => s.clone(),
            (Some(s), _, Some(l)) => format!("{l} ({s})"),
            _ => beschr.clone(),
        };
        liste.push(Belegung {
            layout: code.clone(),
            variante: String::new(),
            name: grundname.clone(),
        });
        let Some(varianten) = l.children().find(|c| c.has_tag_name("variantList")) else {
            continue;
        };
        for v in varianten.children().filter(|c| c.has_tag_name("variant")) {
            let (Some(vcode), Some(vbeschr)) = (kind_text(v, "name"), kind_text(v, "description")) else {
                continue;
            };
            let name = if vbeschr.starts_with(&wort) {
                match klammer(&vbeschr) {
                    Some(k) => {
                        // die Teile der Grundbelegung (z. B. „UK“) nicht wiederholen
                        let grund: Vec<&str> = klammer(&beschr).map(|g| g.split(',').map(str::trim).collect()).unwrap_or_default();
                        let teile: Vec<String> = k
                            .split(", ")
                            .filter(|t| !grund.contains(&t.trim()))
                            .map(zusatz_deutsch)
                            .collect();
                        if teile.is_empty() {
                            grundname.clone()
                        } else {
                            format!("{grundname} – {}", teile.join(", "))
                        }
                    }
                    None => format!("{grundname} – {vcode}"),
                }
            } else {
                // eigene Sprache der Variante (z. B. Niedersorbisch auf deutscher Tastatur)
                let eigene = codes(v, "languageList", "iso639Id")
                    .first()
                    .and_then(|c| namen.sprache(c));
                match eigene {
                    Some(s) if Some(&s) != sprache.as_ref() => format!("{s} – {grundname}"),
                    _ => format!("{grundname} – {vbeschr}"),
                }
            };
            liste.push(Belegung {
                layout: code.clone(),
                variante: vcode,
                name,
            });
        }
    }
    // Doppelte Namen (z. B. zwei Varianten ohne übersetzbaren Zusatz)
    // bekommen das XKB-Kürzel dahinter.
    let mut zahl: HashMap<String, usize> = HashMap::new();
    for b in &liste {
        *zahl.entry(b.name.clone()).or_default() += 1;
    }
    for b in &mut liste {
        if zahl[&b.name] > 1 {
            b.name = if b.variante.is_empty() {
                format!("{} [{}]", b.name, b.layout)
            } else {
                format!("{} [{}, {}]", b.name, b.layout, b.variante)
            };
        }
    }
    liste.sort_by_key(|b| sortierschluessel(&b.name));
    liste
}

fn sortierschluessel(s: &str) -> String {
    s.to_lowercase()
        .replace('ä', "a")
        .replace('ö', "o")
        .replace('ü', "u")
        .replace('ß', "ss")
}

/// Die Liste des Systems (mit gettext).
pub fn liste() -> Vec<Belegung> {
    let r = std::fs::read_to_string(REGELN).unwrap_or_default();
    let s = std::fs::read_to_string(SPRACHEN).unwrap_or_default();
    let l = std::fs::read_to_string(LAENDER).unwrap_or_default();
    liste_aus(&r, &s, &l, &gettext)
}

/// Anzeigename für `XKB_DEFAULT_LAYOUT`/`_VARIANT` (mehrere Belegungen mit Komma).
pub fn anzeigename(liste: &[Belegung], layout: &str, variante: &str) -> String {
    if layout.is_empty() {
        return "Englisch (USA) – labwc-Standard".into();
    }
    let layouts: Vec<&str> = layout.split(',').collect();
    let varianten: Vec<&str> = variante.split(',').collect();
    layouts
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let v = varianten.get(i).copied().unwrap_or("").trim();
            liste
                .iter()
                .find(|b| b.layout == l.trim() && b.variante == v)
                .map_or_else(
                    || if v.is_empty() { l.trim().to_owned() } else { format!("{} ({v})", l.trim()) },
                    |b| b.name.clone(),
                )
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deutsch(domaene: &str, t: &str) -> String {
        match (domaene, t) {
            ("iso_639-2", "German") => "Deutsch".into(),
            ("iso_639-2", "English") => "Englisch".into(),
            ("iso_639-2", "Lower Sorbian") => "Niedersorbisch".into(),
            ("iso_3166-1", "Austria") => "Österreich".into(),
            ("iso_3166-1", "Belgium") => "Belgien".into(),
            ("iso_3166-1", "United States") => "Vereinigte Staaten".into(),
            _ => t.into(),
        }
    }

    const REGELN_TEST: &str = r#"<?xml version="1.0"?>
<!DOCTYPE xkbConfigRegistry SYSTEM "xkb.dtd">
<xkbConfigRegistry version="1.1"><modelList/><layoutList>
<layout><configItem><name>de</name><shortDescription>de</shortDescription><description>German</description>
<countryList><iso3166Id>DE</iso3166Id></countryList><languageList><iso639Id>deu</iso639Id></languageList></configItem>
<variantList>
<variant><configItem><name>nodeadkeys</name><description>German (no dead keys)</description></configItem></variant>
<variant><configItem><name>dsb</name><description>Lower Sorbian</description><languageList><iso639Id>dsb</iso639Id></languageList></configItem></variant>
</variantList></layout>
<layout><configItem><name>at</name><description>German (Austria)</description>
<countryList><iso3166Id>AT</iso3166Id></countryList><languageList><iso639Id>deu</iso639Id></languageList></configItem>
<variantList><variant><configItem><name>mac</name><description>German (Austria, Macintosh)</description></configItem></variant></variantList></layout>
<layout><configItem><name>us</name><description>English (US)</description>
<countryList><iso3166Id>US</iso3166Id></countryList><languageList><iso639Id>eng</iso639Id></languageList></configItem>
<variantList><variant><configItem><name>intl</name><description>English (US, intl., with dead keys)</description></configItem></variant></variantList></layout>
<layout><configItem><name>be</name><description>Belgian</description>
<countryList><iso3166Id>BE</iso3166Id></countryList><languageList><iso639Id>deu</iso639Id></languageList></configItem></layout>
<layout><configItem><name>brai</name><description>Braille</description></configItem></layout>
</layoutList></xkbConfigRegistry>"#;

    const SPRACHEN_TEST: &str = "{\n  \"639-2\": [\n    {\n      \"alpha_2\": \"de\",\n      \"alpha_3\": \"deu\",\n      \"bibliographic\": \"ger\",\n      \"name\": \"German\"\n    },\n    {\n      \"alpha_3\": \"eng\",\n      \"name\": \"English\"\n    },\n    {\n      \"alpha_3\": \"dsb\",\n      \"name\": \"Lower Sorbian\"\n    }\n  ]\n}\n";
    const LAENDER_TEST: &str = "{\n  \"3166-1\": [\n    {\n      \"alpha_2\": \"AT\",\n      \"name\": \"Austria\"\n    },\n    {\n      \"alpha_2\": \"BE\",\n      \"name\": \"Belgium\"\n    },\n    {\n      \"alpha_2\": \"DE\",\n      \"name\": \"Germany\"\n    },\n    {\n      \"alpha_2\": \"US\",\n      \"name\": \"United States\"\n    }\n  ]\n}\n";

    #[test]
    fn deutsche_namen() {
        let l = liste_aus(REGELN_TEST, SPRACHEN_TEST, LAENDER_TEST, &deutsch);
        let n = |lay: &str, var: &str| l.iter().find(|b| b.layout == lay && b.variante == var).unwrap().name.clone();
        assert_eq!(n("de", ""), "Deutsch");
        assert_eq!(n("de", "nodeadkeys"), "Deutsch – ohne Akzenttasten");
        assert_eq!(n("de", "dsb"), "Niedersorbisch – Deutsch");
        assert_eq!(n("at", ""), "Deutsch (Österreich)");
        assert_eq!(n("at", "mac"), "Deutsch (Österreich) – Macintosh");
        assert_eq!(n("us", ""), "Englisch (Vereinigte Staaten)");
        assert_eq!(n("us", "intl"), "Englisch (Vereinigte Staaten) – international, mit Akzenttasten");
        assert_eq!(n("brai", ""), "Braille");
        assert_eq!(n("be", ""), "Belgien (Deutsch)");
        // sortiert, Umlaute wie Grundbuchstaben
        assert_eq!(l.first().unwrap().name, "Braille");
        assert_eq!(anzeigename(&l, "de", ""), "Deutsch");
        assert_eq!(anzeigename(&l, "de,us", ",intl"), "Deutsch + Englisch (Vereinigte Staaten) – international, mit Akzenttasten");
        assert_eq!(anzeigename(&l, "xx", "y"), "xx (y)");
    }

    #[test]
    fn systemliste_vollstaendig() {
        // xkb-data liegt auf jedem Querencia-System; ohne gettext englisch/deutsch gemischt
        if std::path::Path::new(REGELN).exists() {
            let l = liste();
            assert!(l.len() > 400, "{} Einträge", l.len());
            assert!(l.iter().any(|b| b.layout == "de" && b.variante.is_empty()));
            assert!(l.iter().any(|b| b.layout == "de" && b.variante == "nodeadkeys"));
            let mut namen: Vec<&str> = l.iter().map(|b| b.name.as_str()).collect();
            namen.sort();
            let vorher = namen.len();
            namen.dedup();
            assert_eq!(namen.len(), vorher, "Namen eindeutig");
        }
    }
}
