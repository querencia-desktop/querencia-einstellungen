//! Werte aus `org.gnome.desktop.interface` (G2-AJ, QD-046 ④): Themen,
//! Zeiger, Farbschema und Schriften. GTK 4 liest sie über das Settings-Portal,
//! eine Änderung wirkt deshalb sofort in allen laufenden GTK-4-Programmen
//! (auch Panel und Desktop). Werte werden als Text geführt; ganze Zahlen
//! (`cursor-size`) werden beim Lesen und Schreiben umgewandelt.
use std::collections::BTreeMap;

use crate::i18n::gettext_f;
use gtk4::gio;
use gtk4::prelude::*;

pub const SCHEMA: &str = "org.gnome.desktop.interface";

pub const SYMBOLTHEMA: &str = "icon-theme";
pub const GTK_THEMA: &str = "gtk-theme";
pub const ZEIGERTHEMA: &str = "cursor-theme";
pub const ZEIGERGROESSE: &str = "cursor-size";
pub const FARBSCHEMA: &str = "color-scheme";
pub const SCHRIFT: &str = "font-name";
pub const DOKUMENTSCHRIFT: &str = "document-font-name";
pub const DICKTENGLEICH: &str = "monospace-font-name";

/// Querencias Standard (Thema-1/2/3, Entscheidungen 20.10.: Querencia-Klassisch,
/// Symbolthema Querencia, Variante A wie Ubuntu 10.04: DejaVu 10, DMZ-White 24).
/// „Standard wiederherstellen“ zeigt diese Werte, nicht die des Schemas.
pub const STANDARD: &[(&str, &str)] = &[
    (SYMBOLTHEMA, "Querencia"),
    (GTK_THEMA, "Querencia-Klassisch"),
    (ZEIGERTHEMA, "DMZ-White"),
    (ZEIGERGROESSE, "24"),
    (FARBSCHEMA, "default"),
    (SCHRIFT, "DejaVu Sans 10"),
    (DOKUMENTSCHRIFT, "DejaVu Sans 10"),
    (DICKTENGLEICH, "DejaVu Sans Mono 10"),
];

pub fn standard(schluessel: &[&str]) -> BTreeMap<String, String> {
    STANDARD
        .iter()
        .filter(|(k, _)| schluessel.contains(k))
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn einstellungen() -> Option<gio::Settings> {
    let quelle = gio::SettingsSchemaSource::default()?;
    quelle.lookup(SCHEMA, true)?;
    Some(gio::Settings::new(SCHEMA))
}

/// Die gesetzten (oder vorgegebenen) Werte; fehlt das Schema oder ein
/// Schlüssel, fehlt er auch in der Liste.
pub fn lesen(schluessel: &[&str]) -> BTreeMap<String, String> {
    let mut werte = BTreeMap::new();
    if schluessel.is_empty() {
        return werte;
    }
    let Some(s) = einstellungen() else {
        return werte;
    };
    let Some(schema) = s.settings_schema() else {
        return werte;
    };
    for k in schluessel {
        if !schema.has_key(k) {
            continue;
        }
        let v = s.value(k);
        let text = v
            .str()
            .map(str::to_owned)
            .or_else(|| v.get::<i32>().map(|i| i.to_string()))
            .unwrap_or_else(|| v.print(false).to_string());
        werte.insert((*k).to_owned(), text);
    }
    werte
}

/// Schreibt die Werte und wartet, bis dconf sie hat.
pub fn schreiben(aenderungen: &[(&str, String)]) -> Result<(), String> {
    if aenderungen.is_empty() {
        return Ok(());
    }
    let s = einstellungen()
        .ok_or_else(|| gettext_f("Schema {schema} is missing", &[("schema", SCHEMA)]))?;
    for (k, wert) in aenderungen {
        let ergebnis = if s.value(k).type_() == gtk4::glib::VariantTy::INT32 {
            let zahl: i32 = wert.parse().map_err(|_| {
                gettext_f(
                    "{key}: “{value}” is not a number",
                    &[("key", k), ("value", wert)],
                )
            })?;
            s.set_int(k, zahl)
        } else {
            s.set_string(k, wert)
        };
        ergebnis.map_err(|e| format!("{k}: {e}"))?;
    }
    gio::Settings::sync();
    Ok(())
}

/// Für die Bereiche: die geänderten Schlüssel aus zwei Wertlisten.
pub fn unterschiede(
    neu: &[(&'static str, String)],
    alt: &[(&'static str, String)],
) -> Vec<(&'static str, String)> {
    neu.iter()
        .filter(|(k, v)| !alt.iter().any(|(ak, av)| ak == k && av == v))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_nur_gefragte() {
        let s = standard(&[SYMBOLTHEMA, ZEIGERGROESSE]);
        assert_eq!(s.len(), 2);
        assert_eq!(s[SYMBOLTHEMA], "Querencia");
        assert_eq!(s[ZEIGERGROESSE], "24");
    }

    #[test]
    fn unterschiede_nur_geaenderte() {
        let alt = vec![
            (GTK_THEMA, "A".to_owned()),
            (FARBSCHEMA, "default".to_owned()),
        ];
        let neu = vec![
            (GTK_THEMA, "B".to_owned()),
            (FARBSCHEMA, "default".to_owned()),
        ];
        assert_eq!(unterschiede(&neu, &alt), vec![(GTK_THEMA, "B".to_owned())]);
        assert!(unterschiede(&alt, &alt).is_empty());
    }
}
