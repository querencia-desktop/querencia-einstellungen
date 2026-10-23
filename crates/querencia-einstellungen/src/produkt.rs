//! Produktversion und Adressen von Querencia Desktop für die Seite
//! „Informationen über Querencia“.
//!
//! Quelle sind die Dateien des Metapakets `querencia-desktop` (ab Alpha 2.1):
//! `/usr/share/querencia/release` im Format von os-release(5) (`NAME`,
//! `VERSION`, `HOME_URL`, `BUG_REPORT_URL`, `DOCUMENTATION_URL`; weitere
//! Schlüssel werden übergangen) und `/usr/share/querencia/version` (eine
//! Zeile, der Anzeigename der Ausgabe). Reihenfolge: zuerst `release`, fehlt
//! dort `VERSION`, die erste Zeile von `version`. Fehlen beide Dateien (kein
//! Metapaket, Bau aus dem Quellbaum), zeigt die Seite keine Produktversion
//! und nimmt die Adressen aus den Konstanten unten. Die Pfade sind fest;
//! Tests lesen über [`Produkt::lesen_aus`] aus einem eigenen Ordner.
use std::path::Path;

use crate::i18n::gettext;

/// Ordner der beiden Dateien im installierten System.
pub const ORDNER: &str = "/usr/share/querencia";

pub const NAME: &str = "Querencia Desktop";
pub const WEBSITE: &str = "https://querencialinux.org";
pub const FEHLER_MELDEN: &str = "https://github.com/querencia-desktop/querencia/issues/new/choose";
/// Online-Hilfe ohne `release`, in der Sprache der Oberfläche (der Katalog
/// übersetzt die Adresse; ohne Katalog die englische Seite).
pub fn hilfe_vorgabe() -> String {
    // TRANSLATORS: address of the online help in your language; only change “/en/” if querencialinux.org has docs in your language
    gettext("https://querencialinux.org/en/docs/")
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Produkt {
    pub name: Option<String>,
    pub version: Option<String>,
    pub website: Option<String>,
    pub fehler_melden: Option<String>,
    pub hilfe: Option<String>,
}

impl Produkt {
    pub fn lesen() -> Self {
        Self::lesen_aus(Path::new(ORDNER))
    }

    pub fn lesen_aus(ordner: &Path) -> Self {
        let mut p = std::fs::read_to_string(ordner.join("release"))
            .map(|t| Self::aus_release(&t))
            .unwrap_or_default();
        if p.version.is_none() {
            p.version = std::fs::read_to_string(ordner.join("version"))
                .ok()
                .and_then(|t| t.lines().next().map(|z| z.trim().to_string()))
                .filter(|v| !v.is_empty());
        }
        p
    }

    fn aus_release(text: &str) -> Self {
        let mut p = Self::default();
        for zeile in text.lines() {
            let zeile = zeile.trim();
            if zeile.is_empty() || zeile.starts_with('#') {
                continue;
            }
            let Some((schluessel, wert)) = zeile.split_once('=') else {
                continue;
            };
            let wert = wert_lesen(wert.trim());
            if wert.is_empty() {
                continue;
            }
            let ziel = match schluessel.trim() {
                "NAME" => &mut p.name,
                "VERSION" => &mut p.version,
                "HOME_URL" => &mut p.website,
                "BUG_REPORT_URL" => &mut p.fehler_melden,
                "DOCUMENTATION_URL" => &mut p.hilfe,
                _ => continue,
            };
            *ziel = Some(wert);
        }
        p
    }

    pub fn name(&self) -> &str {
        self.name.as_deref().unwrap_or(NAME)
    }

    pub fn website(&self) -> &str {
        self.website.as_deref().unwrap_or(WEBSITE)
    }

    pub fn fehler_melden(&self) -> &str {
        self.fehler_melden.as_deref().unwrap_or(FEHLER_MELDEN)
    }

    pub fn hilfe(&self) -> String {
        self.hilfe.clone().unwrap_or_else(hilfe_vorgabe)
    }
}

/// Wert nach os-release(5): in doppelten oder einfachen Anführungszeichen
/// oder ohne; Escapes `\"`, `\\`, `` \` ``, `\$`. Keine Shell-Erweiterung.
fn wert_lesen(roh: &str) -> String {
    let innen = ['"', '\'']
        .iter()
        .find_map(|q| {
            roh.strip_prefix(*q)
                .and_then(|r| r.strip_suffix(*q))
                .filter(|_| roh.len() >= 2)
        })
        .unwrap_or(roh);
    let mut aus = String::with_capacity(innen.len());
    let mut zeichen = innen.chars();
    while let Some(c) = zeichen.next() {
        if c == '\\' {
            match zeichen.next() {
                Some(n @ ('"' | '\\' | '`' | '$')) => aus.push(n),
                Some(n) => {
                    aus.push('\\');
                    aus.push(n);
                }
                None => aus.push('\\'),
            }
        } else {
            aus.push(c);
        }
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALPHA21: &str = "NAME=\"Querencia Desktop\"\nVERSION=\"Alpha 2.1\"\nVERSION_ID=\"0.1.0~alpha2.1\"\nHOME_URL=\"https://querencialinux.org\"\nBUG_REPORT_URL=\"https://github.com/querencia-desktop/querencia/issues\"\nDOCUMENTATION_URL=\"https://querencialinux.org/de/docs/\"\n";

    fn ordner(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "querencia-einstellungen-produkt-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn release_alpha21() {
        let d = ordner("alpha21");
        std::fs::write(d.join("release"), ALPHA21).unwrap();
        std::fs::write(d.join("version"), "Anders\n").unwrap();
        let p = Produkt::lesen_aus(&d);
        assert_eq!(p.name(), "Querencia Desktop");
        assert_eq!(p.version.as_deref(), Some("Alpha 2.1"));
        assert_eq!(p.website(), "https://querencialinux.org");
        assert_eq!(
            p.fehler_melden(),
            "https://github.com/querencia-desktop/querencia/issues"
        );
        assert_eq!(p.hilfe(), "https://querencialinux.org/de/docs/");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn ohne_dateien_rueckfall() {
        let d = ordner("leer");
        let p = Produkt::lesen_aus(&d);
        assert_eq!(p, Produkt::default());
        assert_eq!(p.name(), "Querencia Desktop");
        assert_eq!(p.version, None);
        assert_eq!(p.website(), "https://querencialinux.org");
        assert_eq!(
            p.fehler_melden(),
            "https://github.com/querencia-desktop/querencia/issues/new/choose"
        );
        // ohne gebundenen Katalog die englische Seite
        assert_eq!(p.hilfe(), "https://querencialinux.org/en/docs/");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn nur_version() {
        let d = ordner("version");
        std::fs::write(d.join("version"), "  Alpha 2.1  \nzweite Zeile\n").unwrap();
        let p = Produkt::lesen_aus(&d);
        assert_eq!(p.version.as_deref(), Some("Alpha 2.1"));
        assert_eq!(p.website(), WEBSITE);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn release_ohne_version_nimmt_version() {
        let d = ordner("ohne-version");
        std::fs::write(d.join("release"), "HOME_URL=https://example.org\n").unwrap();
        std::fs::write(d.join("version"), "Alpha 3\n").unwrap();
        let p = Produkt::lesen_aus(&d);
        assert_eq!(p.version.as_deref(), Some("Alpha 3"));
        assert_eq!(p.website(), "https://example.org");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn syntax() {
        let p = Produkt::aus_release(
            "# Kommentar\n\n  VERSION_CODENAME=neu\nNAME='Q D'\nVERSION=\"a \\\"b\\\" \\$c \\\\ \\`d\\` \\n\"\nKAPUTT\nHOME_URL=\"\"\n",
        );
        assert_eq!(p.name.as_deref(), Some("Q D"));
        assert_eq!(p.version.as_deref(), Some("a \"b\" $c \\ `d` \\n"));
        assert_eq!(p.website, None);
        assert_eq!(wert_lesen("\""), "\"");
    }
}
