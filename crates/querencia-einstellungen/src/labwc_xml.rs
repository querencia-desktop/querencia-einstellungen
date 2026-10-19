//! Gezielte Änderungen an labwcs `rc.xml`, ohne GTK (G2-AG, QD-046 ②).
//!
//! Gelesen wird mit roxmltree, geändert wird im **Text**: nur der Wert eines
//! Elements oder Attributs wird ersetzt bzw. ein neues Element eingefügt;
//! Kommentare, Einrückung, Reihenfolge und alles Fremde bleiben byte-gleich.
//! Jedes Ergebnis wird erneut geparst, ungültiges XML wird nie geliefert.
//!
//! labwc liest rc.xml „element/attribute agnostic“ (labwc-config(5)):
//! `<keyboard><repeatRate>30</repeatRate></keyboard>` gleicht
//! `<keyboard repeatRate="30"/>`, Namen ohne Rücksicht auf Groß-/Kleinschreibung,
//! und bei mehrfachen Angaben gilt die letzte. Genau so liest und ändert
//! dieses Modul: Wert = letzte Fundstelle (Attribut oder Kindelement) in allen
//! passenden Behältern; geändert wird diese letzte Fundstelle, sonst wird im
//! letzten passenden Behälter ein Kindelement angelegt (fehlende Behälter
//! samt Filterattribut, z. B. `<device category="default">`, mit).
use std::ops::Range;

/// Welche Elemente ein Pfadschritt trifft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    Keiner,
    /// `<libinput><device>`: ohne `category` oder `category="default"` – das
    /// Profil für alle Geräte ohne eigenes (labwc `get_device_type`).
    Standardgeraet,
    /// `<theme><font place="…">`
    Ort(&'static str),
    /// `<theme><font>` ohne `place` (gilt für alle Orte; nur zum Lesen).
    OhneOrt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Schritt {
    pub name: &'static str,
    pub filter: Filter,
}

const fn s(name: &'static str) -> Schritt {
    Schritt {
        name,
        filter: Filter::Keiner,
    }
}

pub type Pfad = &'static [Schritt];

pub const TASTATUR: Pfad = &[s("keyboard")];
pub const MAUS: Pfad = &[s("mouse")];
pub const GERAET: Pfad = &[
    s("libinput"),
    Schritt {
        name: "device",
        filter: Filter::Standardgeraet,
    },
];
pub const THEMA: Pfad = &[s("theme")];
pub const TITELLEISTE: Pfad = &[s("theme"), s("titlebar")];
pub const SCHRIFT_AKTIV: Pfad = &[
    s("theme"),
    Schritt {
        name: "font",
        filter: Filter::Ort("ActiveWindow"),
    },
];
pub const SCHRIFT_INAKTIV: Pfad = &[
    s("theme"),
    Schritt {
        name: "font",
        filter: Filter::Ort("InactiveWindow"),
    },
];
pub const SCHRIFT_ALLE: Pfad = &[
    s("theme"),
    Schritt {
        name: "font",
        filter: Filter::OhneOrt,
    },
];
pub const ARBEITSFLAECHEN: Pfad = &[s("desktops")];

/// Alle Werte, die Querencia Einstellungen in rc.xml verwaltet (für den
/// Dreiwegevergleich mit einer neuen Vorgabe); dazu kommen die Namen der
/// Arbeitsflächen ([`namen`]).
pub const VERWALTET: &[(Pfad, &str)] = &[
    (TASTATUR, "repeatRate"),
    (TASTATUR, "repeatDelay"),
    (TASTATUR, "numlock"),
    (MAUS, "doubleClickTime"),
    (GERAET, "pointerSpeed"),
    (GERAET, "accelProfile"),
    (GERAET, "leftHanded"),
    (GERAET, "naturalScroll"),
    (THEMA, "name"),
    (THEMA, "cornerRadius"),
    (THEMA, "dropShadows"),
    (TITELLEISTE, "layout"),
    (SCHRIFT_AKTIV, "name"),
    (SCHRIFT_AKTIV, "size"),
    (SCHRIFT_INAKTIV, "name"),
    (SCHRIFT_INAKTIV, "size"),
    (ARBEITSFLAECHEN, "number"),
    (ARBEITSFLAECHEN, "popupTime"),
];

/// Grundgerüst, wenn es noch gar keine rc.xml gibt.
pub const LEER: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<labwc_config>\n</labwc_config>\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fehler(pub String);

impl std::fmt::Display for Fehler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn parsen(text: &str) -> Result<roxmltree::Document<'_>, Fehler> {
    let doc = roxmltree::Document::parse(text)
        .map_err(|e| Fehler(format!("rc.xml ist kein gültiges XML ({e})")))?;
    if !doc
        .root_element()
        .tag_name()
        .name()
        .eq_ignore_ascii_case("labwc_config")
    {
        return Err(Fehler(format!(
            "rc.xml: Wurzel ist <{}>, nicht <labwc_config>",
            doc.root_element().tag_name().name()
        )));
    }
    Ok(doc)
}

/// Prüft einen Text so, wie die Änderungen ihn verlangen (leer zählt als [`LEER`]).
pub fn gueltig(text: &str) -> Result<(), Fehler> {
    if text.trim().is_empty() {
        return Ok(());
    }
    parsen(text).map(|_| ())
}

fn attribut<'a>(n: roxmltree::Node<'a, '_>, name: &str) -> Option<&'a str> {
    n.attributes()
        .find(|a| a.name().eq_ignore_ascii_case(name))
        .map(|a| a.value())
}

fn passt(n: roxmltree::Node, schritt: &Schritt) -> bool {
    n.is_element()
        && n.tag_name().name().eq_ignore_ascii_case(schritt.name)
        && match schritt.filter {
            Filter::Keiner => true,
            Filter::Standardgeraet => {
                attribut(n, "category").is_none_or(|c| c.eq_ignore_ascii_case("default"))
            }
            Filter::Ort(ort) => attribut(n, "place").is_some_and(|p| p.eq_ignore_ascii_case(ort)),
            Filter::OhneOrt => attribut(n, "place").is_none(),
        }
}

/// Alle Elemente, die auf den Pfad passen, in Dokumentreihenfolge.
fn behaelter<'a, 'i>(doc: &'a roxmltree::Document<'i>, pfad: &[Schritt]) -> Vec<roxmltree::Node<'a, 'i>> {
    let mut ebene = vec![doc.root_element()];
    for schritt in pfad {
        ebene = ebene
            .iter()
            .flat_map(|n| n.children().filter(|c| passt(*c, schritt)))
            .collect();
    }
    ebene
}

enum Fund<'a, 'i> {
    Attribut {
        wert: Range<usize>,
        anfuehrung: u8,
        text: String,
    },
    Element(roxmltree::Node<'a, 'i>),
}

fn element_text(n: roxmltree::Node) -> String {
    n.children()
        .filter(|c| c.is_text())
        .filter_map(|c| c.text())
        .collect::<String>()
        .trim()
        .to_owned()
}

fn funde<'a, 'i>(doc: &'a roxmltree::Document<'i>, pfad: &[Schritt], schluessel: &str) -> Vec<Fund<'a, 'i>> {
    let mut liste = Vec::new();
    for b in behaelter(doc, pfad) {
        for a in b.attributes() {
            if a.name().eq_ignore_ascii_case(schluessel) {
                let r = a.range_value();
                let anfuehrung = doc.input_text().as_bytes()[r.start - 1];
                liste.push(Fund::Attribut {
                    wert: r,
                    anfuehrung,
                    text: a.value().trim().to_owned(),
                });
            }
        }
        for c in b.children() {
            if c.is_element() && c.tag_name().name().eq_ignore_ascii_case(schluessel) {
                liste.push(Fund::Element(c));
            }
        }
    }
    liste
}

/// Der wirksame Wert (letzte Fundstelle), getrimmt; `None` = nicht gesetzt.
pub fn wert(text: &str, pfad: &[Schritt], schluessel: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(text).ok()?;
    match funde(&doc, pfad, schluessel).pop()? {
        Fund::Attribut { text, .. } => Some(text),
        Fund::Element(n) => Some(element_text(n)),
    }
}

/// Namen der Arbeitsflächen (`<desktops><names><name>`), wie labwc sie anhängt.
pub fn namen(text: &str) -> Vec<String> {
    let Ok(doc) = roxmltree::Document::parse(text) else {
        return Vec::new();
    };
    behaelter(&doc, &[s("desktops"), s("names"), s("name")])
        .into_iter()
        .map(element_text)
        .collect()
}

fn text_maskieren(wert: &str) -> String {
    wert.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn attribut_maskieren(wert: &str, anfuehrung: u8) -> String {
    let w = text_maskieren(wert);
    if anfuehrung == b'\'' {
        w.replace('\'', "&apos;")
    } else {
        w.replace('"', "&quot;")
    }
}

fn ersetzen(text: &str, bereich: Range<usize>, neu: &str) -> String {
    let mut t = String::with_capacity(text.len() + neu.len());
    t.push_str(&text[..bereich.start]);
    t.push_str(neu);
    t.push_str(&text[bereich.end..]);
    t
}

/// Ende des Start-Tags (Index des `>`), Anführungszeichen beachtet.
fn starttag_ende(element: &str) -> usize {
    let mut in_wert: Option<u8> = None;
    for (i, b) in element.bytes().enumerate().skip(1) {
        match (in_wert, b) {
            (Some(q), _) if b == q => in_wert = None,
            (Some(_), _) => {}
            (None, b'"' | b'\'') => in_wert = Some(b),
            (None, b'>') => return i,
            _ => {}
        }
    }
    element.len() - 1
}

fn ist_leer_element(element: &str) -> bool {
    element.as_bytes()[starttag_ende(element) - 1] == b'/'
}

/// Qualifizierter Name, wie er im Start-Tag steht.
fn tagname(element: &str) -> &str {
    let rest = &element[1..];
    let ende = rest
        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .unwrap_or(rest.len());
    &rest[..ende]
}

/// Einrückung der Zeile, in der `pos` steht, falls davor nur Leerraum steht.
fn einzug(text: &str, pos: usize) -> String {
    let zeilenanfang = text[..pos].rfind('\n').map_or(0, |i| i + 1);
    let davor = &text[zeilenanfang..pos];
    if davor.bytes().all(|b| b == b' ' || b == b'\t') {
        davor.to_owned()
    } else {
        String::new()
    }
}

fn inhalt_setzen(text: &str, n: roxmltree::Node, neu: &str) -> String {
    let r = n.range();
    let e = &text[r.clone()];
    if ist_leer_element(e) {
        let ende = starttag_ende(e);
        let kopf = e[..ende - 1].trim_end();
        let name = tagname(e);
        ersetzen(text, r, &format!("{kopf}>{neu}</{name}>"))
    } else {
        let anfang = r.start + starttag_ende(e) + 1;
        let schluss = r.start + e.rfind("</").unwrap_or(e.len());
        ersetzen(text, anfang..schluss, neu)
    }
}

/// Fügt `zeilen` (relativ eingerückt, ohne Zeilenende) als letzte Kinder in
/// das Element `eltern` ein; ein leeres Element `<x/>` wird aufgeklappt.
fn einfuegen(text: &str, eltern: roxmltree::Node, zeilen: &[String]) -> String {
    let r = eltern.range();
    let e = &text[r.clone()];
    let aussen = einzug(text, r.start);
    let innen = format!("{aussen}  ");
    let block: String = zeilen.iter().map(|z| format!("{innen}{z}\n")).collect();
    if ist_leer_element(e) {
        let ende = r.start + starttag_ende(e);
        let mut schraeg = ende - 1;
        while schraeg > r.start && text.as_bytes()[schraeg - 1].is_ascii_whitespace() {
            schraeg -= 1;
        }
        let name = tagname(e);
        return ersetzen(text, schraeg..ende + 1, &format!(">\n{block}{aussen}</{name}>"));
    }
    let schluss = r.start + e.rfind("</").unwrap_or(e.len());
    let mut s = schluss;
    while s > r.start && matches!(text.as_bytes()[s - 1], b' ' | b'\t') {
        s -= 1;
    }
    if text.as_bytes()[s - 1] == b'\n' {
        ersetzen(text, s..s, &block)
    } else {
        // alles in einer Zeile: ohne Zeilenumbrüche anhängen
        let einzeilig: String = zeilen.iter().map(|z| z.trim()).collect();
        ersetzen(text, schluss..schluss, &einzeilig)
    }
}

fn oeffnen(schritt: &Schritt) -> String {
    match schritt.filter {
        Filter::Standardgeraet => format!("<{} category=\"default\">", schritt.name),
        Filter::Ort(ort) => format!("<{} place=\"{ort}\">", schritt.name),
        Filter::Keiner | Filter::OhneOrt => format!("<{}>", schritt.name),
    }
}

/// Legt `inhalt` (relative Zeilen) im letzten Behälter des Pfads an; fehlende
/// Behälter werden mit angelegt.
fn anlegen(text: &str, pfad: &[Schritt], inhalt: &[String]) -> Result<String, Fehler> {
    let basis = if text.trim().is_empty() { LEER } else { text };
    let doc = parsen(basis)?;
    for k in (0..=pfad.len()).rev() {
        if let Some(eltern) = behaelter(&doc, &pfad[..k]).pop() {
            let rest = &pfad[k..];
            let mut zeilen = Vec::new();
            for (i, schritt) in rest.iter().enumerate() {
                zeilen.push(format!("{}{}", "  ".repeat(i), oeffnen(schritt)));
            }
            let tiefe = "  ".repeat(rest.len());
            zeilen.extend(inhalt.iter().map(|z| format!("{tiefe}{z}")));
            for (i, schritt) in rest.iter().enumerate().rev() {
                zeilen.push(format!("{}</{}>", "  ".repeat(i), schritt.name));
            }
            return pruefen(einfuegen(basis, eltern, &zeilen));
        }
    }
    unreachable!("die Wurzel passt immer auf den leeren Pfad")
}

fn pruefen(neu: String) -> Result<String, Fehler> {
    parsen(&neu)?;
    Ok(neu)
}

/// Setzt den Wert `schluessel` unter `pfad` (siehe Moduldoku).
pub fn setzen(text: &str, pfad: &[Schritt], schluessel: &str, wert: &str) -> Result<String, Fehler> {
    if !text.trim().is_empty() {
        let doc = parsen(text)?;
        if let Some(fund) = funde(&doc, pfad, schluessel).pop() {
            let neu = match fund {
                Fund::Attribut {
                    wert: bereich,
                    anfuehrung,
                    ..
                } => ersetzen(text, bereich, &attribut_maskieren(wert, anfuehrung)),
                Fund::Element(n) => inhalt_setzen(text, n, &text_maskieren(wert)),
            };
            return pruefen(neu);
        }
    }
    anlegen(
        text,
        pfad,
        &[format!("<{schluessel}>{}</{schluessel}>", text_maskieren(wert))],
    )
}

/// Entfernt ein Element samt eigener Zeile (Einrückung und Zeilenende),
/// wenn es allein in seiner Zeile steht.
fn entfernen_bereich(text: &str, r: Range<usize>) -> Range<usize> {
    let b = text.as_bytes();
    let mut anfang = r.start;
    while anfang > 0 && matches!(b[anfang - 1], b' ' | b'\t') {
        anfang -= 1;
    }
    let mut ende = r.end;
    while ende < b.len() && matches!(b[ende], b' ' | b'\t' | b'\r') {
        ende += 1;
    }
    if (anfang == 0 || b[anfang - 1] == b'\n') && (ende == b.len() || b[ende] == b'\n') {
        anfang..(ende + 1).min(b.len())
    } else {
        r
    }
}

/// Ersetzt alle `<names>` der Arbeitsflächen durch eine Liste (leer: keine).
pub fn namen_setzen(text: &str, liste: &[String]) -> Result<String, Fehler> {
    let mut neu = if text.trim().is_empty() {
        LEER.to_owned()
    } else {
        text.to_owned()
    };
    // alle vorhandenen <names> entfernen, von hinten nach vorn
    let bereiche: Vec<Range<usize>> = {
        let doc = parsen(&neu)?;
        behaelter(&doc, &[s("desktops"), s("names")])
            .into_iter()
            .map(|n| entfernen_bereich(&neu, n.range()))
            .collect()
    };
    for r in bereiche.into_iter().rev() {
        neu = ersetzen(&neu, r, "");
    }
    neu = pruefen(neu)?;
    if liste.is_empty() {
        return Ok(neu);
    }
    let mut zeilen = vec!["<names>".to_owned()];
    zeilen.extend(
        liste
            .iter()
            .map(|n| format!("  <name>{}</name>", text_maskieren(n))),
    );
    zeilen.push("</names>".to_owned());
    anlegen(&neu, ARBEITSFLAECHEN, &zeilen)
}

// ------------------------------------------------------------ Vergleich

/// Schlüssel, die an diesem Behälter verwaltet sind.
fn verwaltet_hier(n: roxmltree::Node) -> Vec<&'static str> {
    let mut liste = Vec::new();
    for (pfad, schluessel) in VERWALTET {
        if pfad_trifft(n, pfad) {
            liste.push(*schluessel);
        }
    }
    if pfad_trifft(n, ARBEITSFLAECHEN) {
        liste.push("names");
    }
    liste
}

/// Trifft der Pfad (von der Wurzel aus) genau dieses Element?
fn pfad_trifft(n: roxmltree::Node, pfad: &[Schritt]) -> bool {
    let mut knoten = Some(n);
    for schritt in pfad.iter().rev() {
        match knoten {
            Some(k) if passt(k, schritt) => knoten = k.parent_element(),
            _ => return false,
        }
    }
    knoten.is_some_and(|k| k == k.document().root_element())
}

fn behaelter_name(n: roxmltree::Node) -> bool {
    [
        "keyboard", "mouse", "libinput", "device", "theme", "titlebar", "font", "desktops",
    ]
    .iter()
    .any(|b| n.tag_name().name().eq_ignore_ascii_case(b))
}

/// Kanonische Form ohne die verwalteten Werte: Elementnamen (klein),
/// Attribute (sortiert), Text ohne Leerraum, Kinder; Kommentare zählen nicht.
/// Behälter, die dadurch leer werden, fallen weg.
fn kanonisch(n: roxmltree::Node, aus: &mut String) {
    let verwaltet = verwaltet_hier(n);
    let mut attr: Vec<String> = n
        .attributes()
        .filter(|a| !verwaltet.iter().any(|v| a.name().eq_ignore_ascii_case(v)))
        .map(|a| format!("{}={}", a.name().to_ascii_lowercase(), a.value().trim()))
        .collect();
    attr.sort();
    let mut innen = String::new();
    for c in n.children() {
        if c.is_element() {
            if verwaltet
                .iter()
                .any(|v| c.tag_name().name().eq_ignore_ascii_case(v))
            {
                continue;
            }
            kanonisch(c, &mut innen);
        } else if c.is_text() {
            let t = c.text().unwrap_or("").trim();
            if !t.is_empty() {
                innen.push_str(&format!("\"{t}\""));
            }
        }
    }
    let nur_filter = attr
        .iter()
        .all(|a| a == "category=default" || a.starts_with("place="));
    if behaelter_name(n) && innen.is_empty() && nur_filter && n.parent_element().is_some() {
        return;
    }
    aus.push_str(&format!(
        "<{} {}>{}</>",
        n.tag_name().name().to_ascii_lowercase(),
        attr.join(" "),
        innen
    ));
}

/// Unterscheiden sich zwei rc.xml nur in den verwalteten Werten?
pub fn nur_verwaltete_unterschiede(a: &str, b: &str) -> bool {
    let (Ok(da), Ok(db)) = (
        roxmltree::Document::parse(if a.trim().is_empty() { LEER } else { a }),
        roxmltree::Document::parse(if b.trim().is_empty() { LEER } else { b }),
    ) else {
        return false;
    };
    let (mut ka, mut kb) = (String::new(), String::new());
    kanonisch(da.root_element(), &mut ka);
    kanonisch(db.root_element(), &mut kb);
    ka == kb
}

/// Überträgt die eigenen Werte (`ours` ≠ `basis`) auf eine neue Vorgabe.
/// Nur, wenn `ours` sich sonst nicht von `basis` unterscheidet; sonst `Err`.
pub fn auf_neue_vorgabe(ours: &str, basis: &str, neue: &str) -> Result<String, Fehler> {
    if !nur_verwaltete_unterschiede(ours, basis) {
        return Err(Fehler(
            "rc.xml enthält Änderungen außerhalb der Einstellungen – bitte von Hand zusammenführen".into(),
        ));
    }
    let mut neu = neue.to_owned();
    for (pfad, schluessel) in VERWALTET {
        let o = wert(ours, pfad, schluessel);
        if o != wert(basis, pfad, schluessel)
            && let Some(v) = o
        {
            neu = setzen(&neu, pfad, schluessel, &v)?;
        }
    }
    let n = namen(ours);
    if n != namen(basis) {
        neu = namen_setzen(&neu, &n)?;
    }
    Ok(neu)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Vorgabe des Runtime-Repos vom 19.10. (gekürzt um die Kürzel).
    const VORGABE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- Kopf -->
<labwc_config>
  <!-- vier Arbeitsflächen -->
  <desktops number="4" popupTime="0" prefix="Arbeitsfläche" />

  <keyboard>
    <default />
    <keybind key="C-A-Left">
      <action name="GoToDesktop" to="left" wrap="no" />
    </keybind>
  </keyboard>
  <theme>
    <name>Querencia-Klassisch</name>
    <icon>gnome-human</icon>
    <font place="OnScreenDisplay">
      <name>Adwaita Sans</name>
      <size>11</size>
    </font>
  </theme>
</labwc_config>
"#;

    #[test]
    fn lesen_element_und_attribut() {
        assert_eq!(wert(VORGABE, THEMA, "name").as_deref(), Some("Querencia-Klassisch"));
        assert_eq!(wert(VORGABE, ARBEITSFLAECHEN, "number").as_deref(), Some("4"));
        assert_eq!(wert(VORGABE, ARBEITSFLAECHEN, "popuptime").as_deref(), Some("0"));
        assert_eq!(wert(VORGABE, TASTATUR, "repeatRate"), None);
        assert_eq!(wert(VORGABE, SCHRIFT_AKTIV, "name"), None);
        assert!(namen(VORGABE).is_empty());
        // letzte Angabe gilt, Groß-/Kleinschreibung egal
        let t = "<labwc_config><keyboard repeatRate=\"20\"><RepeatRate>30</RepeatRate></keyboard><keyboard><repeatrate> 40 </repeatrate></keyboard></labwc_config>";
        assert_eq!(wert(t, TASTATUR, "repeatRate").as_deref(), Some("40"));
    }

    #[test]
    fn attribut_aendern_laesst_alles_andere_stehen() {
        let neu = setzen(VORGABE, ARBEITSFLAECHEN, "number", "6").unwrap();
        assert_eq!(neu, VORGABE.replace("number=\"4\"", "number=\"6\""));
        let neu = setzen(VORGABE, ARBEITSFLAECHEN, "popupTime", "1000").unwrap();
        assert_eq!(neu, VORGABE.replace("popupTime=\"0\"", "popupTime=\"1000\""));
    }

    #[test]
    fn element_aendern() {
        let neu = setzen(VORGABE, THEMA, "name", "Andere & Co").unwrap();
        assert_eq!(
            neu,
            VORGABE.replace(
                "<name>Querencia-Klassisch</name>",
                "<name>Andere &amp; Co</name>"
            )
        );
        assert_eq!(wert(&neu, THEMA, "name").as_deref(), Some("Andere & Co"));
        // die Schrift des OSD (gleicher Elementname „name“, tiefer) bleibt
        assert!(neu.contains("<name>Adwaita Sans</name>"));
    }

    #[test]
    fn kind_anlegen_mit_einrueckung() {
        let neu = setzen(VORGABE, TASTATUR, "repeatRate", "30").unwrap();
        assert_eq!(
            neu,
            VORGABE.replace(
                "    </keybind>\n  </keyboard>",
                "    </keybind>\n    <repeatRate>30</repeatRate>\n  </keyboard>"
            )
        );
        assert_eq!(wert(&neu, TASTATUR, "repeatRate").as_deref(), Some("30"));
    }

    #[test]
    fn behaelter_anlegen() {
        let neu = setzen(VORGABE, GERAET, "pointerSpeed", "0.5").unwrap();
        assert!(neu.ends_with(
            "  </theme>\n  <libinput>\n    <device category=\"default\">\n      <pointerSpeed>0.5</pointerSpeed>\n    </device>\n  </libinput>\n</labwc_config>\n"
        ));
        assert_eq!(wert(&neu, GERAET, "pointerSpeed").as_deref(), Some("0.5"));
        // zweiter Wert landet im selben Gerät
        let neu2 = setzen(&neu, GERAET, "leftHanded", "yes").unwrap();
        assert!(neu2.contains(
            "      <pointerSpeed>0.5</pointerSpeed>\n      <leftHanded>yes</leftHanded>\n    </device>"
        ));
        let neu3 = setzen(&neu2, SCHRIFT_AKTIV, "size", "12").unwrap();
        assert!(neu3.contains(
            "    </font>\n    <font place=\"ActiveWindow\">\n      <size>12</size>\n    </font>\n  </theme>"
        ));
        assert_eq!(wert(&neu3, SCHRIFT_AKTIV, "size").as_deref(), Some("12"));
        // OSD-Schrift unberührt
        assert_eq!(wert(&neu3, SCHRIFT_ALLE, "size"), None);
    }

    #[test]
    fn geraet_mit_anderer_kategorie_zaehlt_nicht() {
        let t = "<labwc_config>\n  <libinput>\n    <device category=\"touchpad\">\n      <naturalScroll>yes</naturalScroll>\n    </device>\n  </libinput>\n</labwc_config>\n";
        assert_eq!(wert(t, GERAET, "naturalScroll"), None);
        let neu = setzen(t, GERAET, "naturalScroll", "no").unwrap();
        assert!(neu.contains("<device category=\"touchpad\">\n      <naturalScroll>yes</naturalScroll>\n    </device>\n    <device category=\"default\">\n      <naturalScroll>no</naturalScroll>\n    </device>\n  </libinput>"));
        // Gerät ohne category gilt als Standard
        let t2 = "<labwc_config><libinput><device><pointerSpeed>0.1</pointerSpeed></device></libinput></labwc_config>";
        assert_eq!(wert(t2, GERAET, "pointerSpeed").as_deref(), Some("0.1"));
        let neu2 = setzen(t2, GERAET, "pointerSpeed", "-0.3").unwrap();
        assert_eq!(neu2, t2.replace("0.1", "-0.3"));
    }

    #[test]
    fn leeres_element_und_einzeilig() {
        let t = "<labwc_config><mouse/></labwc_config>";
        let neu = setzen(t, MAUS, "doubleClickTime", "400").unwrap();
        assert_eq!(
            neu,
            "<labwc_config><mouse>\n  <doubleClickTime>400</doubleClickTime>\n</mouse></labwc_config>"
        );
        let t = "<labwc_config><mouse><x/></mouse></labwc_config>";
        let neu = setzen(t, MAUS, "doubleClickTime", "400").unwrap();
        assert_eq!(
            neu,
            "<labwc_config><mouse><x/><doubleClickTime>400</doubleClickTime></mouse></labwc_config>"
        );
        let t = "<labwc_config><theme><cornerRadius/></theme></labwc_config>";
        assert_eq!(
            setzen(t, THEMA, "cornerRadius", "4").unwrap(),
            "<labwc_config><theme><cornerRadius>4</cornerRadius></theme></labwc_config>"
        );
    }

    #[test]
    fn leere_datei_bekommt_geruest() {
        let neu = setzen("", TASTATUR, "numlock", "on").unwrap();
        assert_eq!(
            neu,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<labwc_config>\n  <keyboard>\n    <numlock>on</numlock>\n  </keyboard>\n</labwc_config>\n"
        );
    }

    #[test]
    fn idempotent() {
        let a = setzen(VORGABE, TASTATUR, "repeatDelay", "500").unwrap();
        let b = setzen(&a, TASTATUR, "repeatDelay", "500").unwrap();
        assert_eq!(a, b);
        let c = namen_setzen(&a, &["Eins".into(), "Zwei".into()]).unwrap();
        let d = namen_setzen(&c, &["Eins".into(), "Zwei".into()]).unwrap();
        assert_eq!(c, d);
        assert_eq!(namen(&d), vec!["Eins", "Zwei"]);
    }

    #[test]
    fn namen_setzen_und_entfernen() {
        let neu = namen_setzen(VORGABE, &["Büro".into(), "<Web>".into()]).unwrap();
        assert!(neu.contains(
            "  <desktops number=\"4\" popupTime=\"0\" prefix=\"Arbeitsfläche\">\n    <names>\n      <name>Büro</name>\n      <name>&lt;Web&gt;</name>\n    </names>\n  </desktops>\n"
        ));
        assert_eq!(namen(&neu), vec!["Büro", "<Web>"]);
        assert_eq!(wert(&neu, ARBEITSFLAECHEN, "number").as_deref(), Some("4"));
        let leer = namen_setzen(&neu, &[]).unwrap();
        assert!(namen(&leer).is_empty());
        assert!(leer.contains("  <desktops number=\"4\" popupTime=\"0\" prefix=\"Arbeitsfläche\">\n  </desktops>\n"));
        // zweimal <names>: beide ersetzt
        let t = "<labwc_config>\n  <desktops>\n    <names><name>a</name></names>\n    <names><name>b</name></names>\n  </desktops>\n</labwc_config>\n";
        assert_eq!(namen(t), vec!["a", "b"]);
        let neu = namen_setzen(t, &["c".into()]).unwrap();
        assert_eq!(
            neu,
            "<labwc_config>\n  <desktops>\n    <names>\n      <name>c</name>\n    </names>\n  </desktops>\n</labwc_config>\n"
        );
    }

    #[test]
    fn attribut_mit_einfachen_anfuehrungszeichen() {
        let t = "<labwc_config><theme name='a'/></labwc_config>";
        assert_eq!(
            setzen(t, THEMA, "name", "it's").unwrap(),
            "<labwc_config><theme name='it&apos;s'/></labwc_config>"
        );
        let t = "<labwc_config><theme name=\"a\" x='>'/></labwc_config>";
        assert_eq!(
            setzen(t, THEMA, "cornerRadius", "3").unwrap(),
            "<labwc_config><theme name=\"a\" x='>'>\n  <cornerRadius>3</cornerRadius>\n</theme></labwc_config>"
        );
    }

    #[test]
    fn ungueltig_wird_abgelehnt() {
        assert!(setzen("<labwc_config><keyboard>", TASTATUR, "numlock", "on").is_err());
        assert!(setzen("<anderes/>", TASTATUR, "numlock", "on").is_err());
        assert!(namen_setzen("<labwc_config>", &[]).is_err());
        assert!(gueltig("").is_ok());
        assert!(gueltig(VORGABE).is_ok());
    }

    #[test]
    fn dreiweg_nur_verwaltete_werte() {
        let ours = setzen(VORGABE, TASTATUR, "repeatRate", "33").unwrap();
        let ours = namen_setzen(&ours, &["A".into()]).unwrap();
        let ours = setzen(&ours, GERAET, "naturalScroll", "yes").unwrap();
        assert!(nur_verwaltete_unterschiede(&ours, VORGABE));
        // neue Vorgabe: zusätzliches Kürzel, andere Anzahl
        let neue = VORGABE
            .replace(
                "    </keybind>\n  </keyboard>",
                "    </keybind>\n    <keybind key=\"W-e\"><action name=\"Execute\" command=\"x\"/></keybind>\n  </keyboard>",
            )
            .replace("number=\"4\"", "number=\"5\"");
        let m = auf_neue_vorgabe(&ours, VORGABE, &neue).unwrap();
        assert!(m.contains("<keybind key=\"W-e\">"));
        assert_eq!(wert(&m, TASTATUR, "repeatRate").as_deref(), Some("33"));
        assert_eq!(wert(&m, GERAET, "naturalScroll").as_deref(), Some("yes"));
        assert_eq!(namen(&m), vec!["A"]);
        // Anzahl hat der Benutzer nicht geändert: neue Vorgabe gilt
        assert_eq!(wert(&m, ARBEITSFLAECHEN, "number").as_deref(), Some("5"));
        // Handänderung außerhalb: abgelehnt
        let hand = ours.replace("<icon>gnome-human</icon>", "<icon>Adwaita</icon>");
        assert!(!nur_verwaltete_unterschiede(&hand, VORGABE));
        assert!(auf_neue_vorgabe(&hand, VORGABE, &neue).is_err());
        // Kommentare zählen nicht
        let kommentar = ours.replace("<!-- Kopf -->", "<!-- anders -->");
        assert!(nur_verwaltete_unterschiede(&kommentar, VORGABE));
    }
}
