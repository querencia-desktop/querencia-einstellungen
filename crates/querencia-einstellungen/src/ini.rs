//! Gruppen in `desktop.ini` einzeln ersetzen, ohne GTK (G2-X).
//!
//! Mehrere Stellen schreiben dieselbe Datei: der Hintergrund-Dialog
//! `[Background]`, der Bereich „Arbeitsfläche“ `[Icons]` und `[Volumes]`.
//! Jede ersetzt nur ihre eigenen Gruppen; alles andere bleibt byte-gleich
//! (Spezifikation `docs/desktop-ini-arbeitsflaeche.md` §5).
//!
//! Zum Block einer Gruppe gehören die Kopfzeile und alle Zeilen bis zur
//! letzten Schlüsselzeile vor der nächsten Kopfzeile; Kommentar- und
//! Leerzeilen direkt davor gehören schon zur nächsten Gruppe.
use std::path::{Path, PathBuf};

/// Zeilen (mit Zeilenende) eines Textes; eine letzte Zeile ohne `\n` bekommt keins.
fn zeilen(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

fn kopf(zeile: &str) -> Option<&str> {
    let z = zeile.trim();
    z.strip_prefix('[')?.strip_suffix(']')
}

fn inhaltlich(zeile: &str) -> bool {
    let z = zeile.trim();
    !z.is_empty() && !z.starts_with('#')
}

/// Bereiche `[anfang, ende)` (Zeilenindizes) aller Blöcke der Gruppe `name`.
fn bloecke(z: &[&str], name: &str) -> Vec<(usize, usize)> {
    let mut liste = Vec::new();
    let mut i = 0;
    while i < z.len() {
        if kopf(z[i]) == Some(name) {
            let anfang = i;
            let mut ende = i + 1;
            let mut j = i + 1;
            while j < z.len() && kopf(z[j]).is_none() {
                if inhaltlich(z[j]) {
                    ende = j + 1;
                }
                j += 1;
            }
            liste.push((anfang, ende));
            i = j;
        } else {
            i += 1;
        }
    }
    liste
}

/// Der Text der Gruppe `name` (alle Blöcke hintereinander, wie gelesen);
/// leer, wenn sie fehlt. Dient dem Vergleich „hat jemand anderes meine
/// Gruppe geändert?“.
pub fn gruppe(text: &str, name: &str) -> String {
    let z = zeilen(text);
    bloecke(&z, name)
        .into_iter()
        .map(|(a, e)| z[a..e].concat())
        .collect()
}

/// Ersetzt die Gruppe `name` durch `block` (beginnt mit `[name]`, endet mit
/// `\n`). Doppelte Blöcke werden zu einem zusammengefasst (an der Stelle des
/// ersten); fehlt die Gruppe, wird sie mit einer Leerzeile Abstand angehängt.
pub fn ersetzen(text: &str, name: &str, block: &str) -> String {
    let z = zeilen(text);
    let b = bloecke(&z, name);
    let Some(&(erster, _)) = b.first() else {
        if text.trim().is_empty() {
            return block.to_owned();
        }
        let mut neu = text.to_owned();
        if !neu.ends_with('\n') {
            neu.push('\n');
        }
        if !neu.ends_with("\n\n") {
            neu.push('\n');
        }
        neu.push_str(block);
        return neu;
    };
    let mut neu = String::with_capacity(text.len() + block.len());
    let mut i = 0;
    for &(a, e) in &b {
        neu.push_str(&z[i..a].concat());
        if a == erster {
            neu.push_str(block);
        }
        i = e;
    }
    let rest = z[i..].concat();
    if !rest.is_empty() && !neu.ends_with('\n') {
        neu.push('\n');
    }
    neu.push_str(&rest);
    neu
}

/// Schreibt `neu` atomar (`.ini.neu` + `rename`); eine vorhandene, nicht leere
/// Datei wird vorher als `desktop.ini.vorher-<Zeit>` gesichert (`zeit` z. B.
/// `20261019-101500`). Rückgabe: die Sicherung, falls angelegt.
pub fn schreiben(
    pfad: &Path,
    neu: &str,
    bisher: &str,
    zeit: &str,
) -> std::io::Result<Option<PathBuf>> {
    if let Some(eltern) = pfad.parent() {
        std::fs::create_dir_all(eltern)?;
    }
    let mut sicherung = None;
    if !bisher.is_empty() && pfad.exists() {
        let name = pfad
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "desktop.ini".into());
        let ziel = pfad.with_file_name(format!("{name}.vorher-{zeit}"));
        if !ziel.exists() {
            std::fs::copy(pfad, &ziel)?;
            sicherung = Some(ziel);
        }
    }
    let temporaer = pfad.with_extension("ini.neu");
    std::fs::write(&temporaer, neu)?;
    std::fs::rename(&temporaer, pfad)?;
    Ok(sicherung)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HG: &str = "[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n";
    const VOL: &str = "[Volumes]\nRemovable=true\nOptical=true\nNetwork=true\nInternal=false\n";

    #[test]
    fn leere_datei_ergibt_nur_den_block() {
        assert_eq!(ersetzen("", "Background", HG), HG);
        assert_eq!(ersetzen("\n\n", "Volumes", VOL), VOL);
    }

    #[test]
    fn fehlende_gruppe_wird_angehaengt() {
        assert_eq!(ersetzen(HG, "Volumes", VOL), format!("{HG}\n{VOL}"));
        // ohne Zeilenende am Schluss
        assert_eq!(
            ersetzen("[Background]\nMode=zoom", "Volumes", VOL),
            format!("[Background]\nMode=zoom\n\n{VOL}")
        );
        // schon mit Leerzeile am Schluss: keine zweite
        assert_eq!(
            ersetzen(&format!("{HG}\n"), "Volumes", VOL),
            format!("{HG}\n{VOL}")
        );
    }

    #[test]
    fn andere_gruppen_bleiben_byte_gleich() {
        let alt = format!(
            "# Kopf-Kommentar\n{HG}\n# zu Volumes\n[Volumes]\nRemovable=false\n# innen\nNetwork=false\n\n[Fremd]\nX=1\n"
        );
        let neu = ersetzen(&alt, "Volumes", VOL);
        assert_eq!(
            neu,
            format!("# Kopf-Kommentar\n{HG}\n# zu Volumes\n{VOL}\n[Fremd]\nX=1\n")
        );
        assert_eq!(gruppe(&neu, "Background"), HG);
        assert_eq!(gruppe(&neu, "Fremd"), "[Fremd]\nX=1\n");
        // der Hintergrund-Dialog ersetzt danach nur seine Gruppe
        let hg2 = "[Background]\nType=color\nFile=\nMode=zoom\nColor=#242424\n";
        let neu2 = ersetzen(&neu, "Background", hg2);
        assert_eq!(gruppe(&neu2, "Volumes"), VOL);
        assert_eq!(gruppe(&neu2, "Background"), hg2);
        assert!(neu2.starts_with("# Kopf-Kommentar\n[Background]\nType=color\n"));
    }

    #[test]
    fn doppelte_gruppen_werden_zusammengefasst() {
        let alt = "[Volumes]\nRemovable=false\n[Icons]\nSize=32\n[Volumes]\nNetwork=false\n";
        let neu = ersetzen(alt, "Volumes", VOL);
        assert_eq!(neu, format!("{VOL}[Icons]\nSize=32\n"));
        assert_eq!(
            gruppe(alt, "Volumes"),
            "[Volumes]\nRemovable=false\n[Volumes]\nNetwork=false\n"
        );
    }

    #[test]
    fn letzter_block_ohne_zeilenende() {
        let alt = "[Volumes]\nRemovable=false";
        assert_eq!(ersetzen(alt, "Volumes", VOL), VOL);
        let alt = "[Icons]\nSize=32\n[Volumes]\nRemovable=false";
        assert_eq!(
            ersetzen(alt, "Icons", "[Icons]\nSize=48\n"),
            "[Icons]\nSize=48\n[Volumes]\nRemovable=false"
        );
    }

    #[test]
    fn gruppe_ohne_inhalt_und_kopf_mit_leerraum() {
        assert_eq!(gruppe("[Fremd]\n", "Volumes"), "");
        let alt = "  [Volumes]  \nRemovable=false\n";
        assert_eq!(ersetzen(alt, "Volumes", VOL), VOL);
    }

    #[test]
    fn schreiben_sichert_und_ersetzt_atomar() {
        let ordner = std::env::temp_dir().join(format!("qe-ini-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&ordner);
        let pfad = ordner.join("desktop.ini");
        assert_eq!(schreiben(&pfad, HG, "", "1").unwrap(), None);
        assert_eq!(std::fs::read_to_string(&pfad).unwrap(), HG);
        let neu = ersetzen(HG, "Volumes", VOL);
        let s = schreiben(&pfad, &neu, HG, "2").unwrap().unwrap();
        assert_eq!(std::fs::read_to_string(&s).unwrap(), HG);
        assert_eq!(std::fs::read_to_string(&pfad).unwrap(), neu);
        assert!(!pfad.with_extension("ini.neu").exists());
        std::fs::remove_dir_all(&ordner).unwrap();
    }
}
