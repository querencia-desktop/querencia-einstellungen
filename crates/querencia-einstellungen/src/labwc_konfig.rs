//! Baustein „labwc-Konfiguration“ (G2-AG, QD-046 ②): wo die Einstellungen
//! labwcs Konfiguration lesen und schreiben, ohne GTK-Oberfläche.
//!
//! Vertrag mit dem Sitzungsstarter: `docs/labwc-benutzerkonfiguration.md`
//! (Runtime-Repo, gleichlautend in `docs/` dieses Repos). Kurz:
//! - geschrieben wird nur die Benutzerkopie `$XDG_CONFIG_HOME/querencia/labwc/`
//!   (`rc.xml` gezielt über [`crate::labwc_xml`], Umgebung nur in
//!   `environment.d/zz-querencia-einstellungen.env`), nie die Vorgabe im
//!   Runtime-Repo (labwc las sie bis Teil A live);
//! - fehlt die Benutzerkopie, wird die Vorgabe gelesen; das erste Schreiben
//!   legt die Benutzerkopie an und notiert die Vorgabe in `.vorgabe/rc.xml`,
//!   damit der Starter sie als Ausgangspunkt kennt;
//! - vor jedem Ersetzen eine Sicherung `<datei>.vorher-<Zeit>` (die zehn
//!   jüngsten bleiben), Schreiben atomar über eine temporäre Datei;
//! - wirksam wird es mit `labwc --reconfigure`, aber nur, wenn der laufende
//!   labwc (`LABWC_PID`) wirklich die Benutzerkopie liest; sonst ab der
//!   nächsten Anmeldung.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::labwc_xml;

/// Vorgaben im Runtime-Repo (nur lesen). Für Prüfungen: `QE_LABWC_VORGABE`.
pub const VORGABE_STANDARD: &str =
    "~/projects/querencia-runtime/session/querencia/labwc";
/// Eigene Umgebungsdatei. labwc liest `environment.d/*.env` nach
/// `environment`, alphabetisch (scandir/alphasort), die letzte Angabe gilt;
/// „zz-“ hält die Wahl des Benutzers hinter den Vorgaben anderer
/// Querencia-Teile (z. B. `querencia-erscheinungsbild.env`, Thema-3).
pub const ENV_DATEI: &str = "zz-querencia-einstellungen.env";
/// So viele Sicherungen je Datei bleiben liegen.
const SICHERUNGEN: usize = 10;

#[derive(Clone, Debug)]
pub struct Orte {
    pub benutzer: PathBuf,
    pub vorgabe: PathBuf,
}

impl Orte {
    pub fn standard() -> Orte {
        Orte {
            benutzer: gtk4::glib::user_config_dir()
                .join("querencia")
                .join("labwc"),
            vorgabe: std::env::var_os("QE_LABWC_VORGABE")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(VORGABE_STANDARD)),
        }
    }
    pub fn rc(&self) -> PathBuf {
        self.benutzer.join("rc.xml")
    }
    pub fn env_datei(&self) -> PathBuf {
        self.benutzer.join("environment.d").join(ENV_DATEI)
    }
    pub fn vorgabe_neu(&self) -> PathBuf {
        self.benutzer.join("rc.xml.vorgabe-neu")
    }
    pub fn vorgabe_basis(&self) -> PathBuf {
        self.benutzer.join(".vorgabe").join("rc.xml")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quelle {
    /// `~/.config/querencia/labwc/rc.xml`
    Benutzer,
    /// noch keine Benutzerkopie: die Vorgabe des Runtime-Repos
    Vorgabe,
    /// weder noch: labwcs eingebaute Werte
    Keine,
}

/// Was die Bereiche zum Anzeigen und Vergleichen brauchen.
#[derive(Clone, Debug)]
pub struct Stand {
    /// wirksame rc.xml (Benutzerkopie, sonst Vorgabe, sonst leer)
    pub rc: String,
    pub quelle: Quelle,
    /// rc.xml der Vorgabe (für „Standard wiederherstellen“)
    pub rc_vorgabe: String,
    /// wirksame Umgebung: environment (Benutzer, sonst Vorgabe), dann
    /// environment.d/*.env in alphabetischer Reihenfolge
    pub env: BTreeMap<String, String>,
    /// dasselbe ohne die eigene Datei (= „Standard“ für Tastatur/Zeiger)
    pub env_ohne_eigene: BTreeMap<String, String>,
    /// eine neuere Vorgabe liegt bereit, die der Starter nicht übernommen hat
    pub vorgabe_neu: bool,
}

/// Zeilen `NAME=WERT` wie labwc (`#` am Zeilenanfang = Kommentar).
pub fn env_parsen(text: &str, ziel: &mut BTreeMap<String, String>) {
    for zeile in text.lines() {
        let z = zeile.trim();
        if z.is_empty() || z.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = z.split_once('=') {
            let k = k.trim();
            if !k.is_empty() {
                ziel.insert(k.to_owned(), v.trim().to_owned());
            }
        }
    }
}

fn lesen_oder_leer(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

pub fn lesen(o: &Orte) -> Stand {
    let rc_vorgabe = lesen_oder_leer(&o.vorgabe.join("rc.xml"));
    let (rc, quelle) = match std::fs::read_to_string(o.rc()) {
        Ok(t) => (t, Quelle::Benutzer),
        Err(_) if !rc_vorgabe.is_empty() => (rc_vorgabe.clone(), Quelle::Vorgabe),
        Err(_) => (String::new(), Quelle::Keine),
    };
    let mut basis = BTreeMap::new();
    let env_benutzer = o.benutzer.join("environment");
    if env_benutzer.is_file() {
        env_parsen(&lesen_oder_leer(&env_benutzer), &mut basis);
    } else {
        env_parsen(&lesen_oder_leer(&o.vorgabe.join("environment")), &mut basis);
    }
    let mut env_ohne_eigene = basis.clone();
    let mut env = basis;
    if let Ok(e) = std::fs::read_dir(o.benutzer.join("environment.d")) {
        let mut dateien: Vec<PathBuf> = e
            .flatten()
            .map(|d| d.path())
            .filter(|p| p.extension().is_some_and(|x| x == "env") && p.is_file())
            .collect();
        dateien.sort();
        for d in dateien {
            let t = lesen_oder_leer(&d);
            env_parsen(&t, &mut env);
            if d.file_name().is_some_and(|n| n != ENV_DATEI) {
                env_parsen(&t, &mut env_ohne_eigene);
            }
        }
    }
    Stand {
        rc,
        quelle,
        rc_vorgabe,
        env,
        env_ohne_eigene,
        vorgabe_neu: o.vorgabe_neu().is_file(),
    }
}

/// Atomar schreiben: temporäre Datei im selben Ordner, dann `rename`.
fn atomar(pfad: &Path, inhalt: &str) -> std::io::Result<()> {
    let ordner = pfad.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(ordner)?;
    let name = pfad
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = ordner.join(format!(".{name}.neu-{}", std::process::id()));
    if let Err(e) = std::fs::write(&tmp, inhalt).and_then(|()| std::fs::rename(&tmp, pfad)) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

/// Sichert eine vorhandene Datei als `<name>.vorher-<zeit>` und räumt ältere
/// Sicherungen über die zehn jüngsten hinaus weg (nur dieses Muster).
fn sichern(pfad: &Path, zeit: &str) -> std::io::Result<Option<PathBuf>> {
    if !pfad.is_file() {
        return Ok(None);
    }
    let name = pfad
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ziel = pfad.with_file_name(format!("{name}.vorher-{zeit}"));
    if !ziel.exists() {
        std::fs::copy(pfad, &ziel)?;
    }
    if let Some(ordner) = pfad.parent()
        && let Ok(e) = std::fs::read_dir(ordner)
    {
        let praefix = format!("{name}.vorher-");
        let mut alte: Vec<PathBuf> = e
            .flatten()
            .map(|d| d.path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with(&praefix))
            })
            .collect();
        alte.sort();
        let zuviel = alte.len().saturating_sub(SICHERUNGEN);
        for p in alte.into_iter().take(zuviel) {
            let _ = std::fs::remove_file(p);
        }
    }
    Ok(Some(ziel))
}

/// Schreibt eine neue rc.xml in die Benutzerkopie (nur gültiges XML).
/// `stand` ist der Stand, auf dem `neu` beruht: kam er aus der Vorgabe,
/// wird die Vorgabe als Ausgangspunkt in `.vorgabe/rc.xml` notiert (falls
/// dort noch nichts steht). Rückgabe: die Sicherung, falls angelegt.
pub fn rc_schreiben(
    o: &Orte,
    neu: &str,
    stand: &Stand,
    zeit: &str,
) -> Result<Option<PathBuf>, String> {
    labwc_xml::gueltig(neu).map_err(|e| e.to_string())?;
    if neu.trim().is_empty() {
        return Err("leere rc.xml wird nicht geschrieben".into());
    }
    let io = |e: std::io::Error| e.to_string();
    std::fs::create_dir_all(&o.benutzer).map_err(io)?;
    if stand.quelle == Quelle::Vorgabe && !o.vorgabe_basis().exists() {
        atomar(&o.vorgabe_basis(), &stand.rc_vorgabe).map_err(io)?;
    }
    let sicherung = sichern(&o.rc(), zeit).map_err(io)?;
    atomar(&o.rc(), neu).map_err(io)?;
    Ok(sicherung)
}

/// Ändert die eigene Umgebungsdatei: `Some(wert)` setzt, `None` entfernt.
/// Andere Zeilen bleiben; ein Schlüssel steht danach höchstens einmal da.
pub fn env_text(alt: &str, aenderungen: &[(&str, Option<String>)]) -> String {
    let mut zeilen: Vec<String> = if alt.is_empty() {
        vec![
            "# Querencia Einstellungen (Tastatur, Maus). labwc liest diese Datei nach".into(),
            "# ../environment; Werte hier haben Vorrang. Wirksam mit labwc --reconfigure,".into(),
            "# für schon laufende Programme ab der nächsten Anmeldung.".into(),
        ]
    } else {
        alt.lines().map(str::to_owned).collect()
    };
    for (schluessel, wert) in aenderungen {
        let ist = |z: &String| {
            let z = z.trim();
            !z.starts_with('#')
                && z.split_once('=')
                    .is_some_and(|(k, _)| k.trim() == *schluessel)
        };
        // erste Zeile des Schlüssels ersetzen, weitere entfernen
        let mut gesetzt = false;
        zeilen = zeilen
            .into_iter()
            .filter_map(|z| match wert {
                _ if !ist(&z) => Some(z),
                Some(w) if !gesetzt => {
                    gesetzt = true;
                    Some(format!("{schluessel}={w}"))
                }
                _ => None,
            })
            .collect();
        if let (Some(w), false) = (wert, gesetzt) {
            zeilen.push(format!("{schluessel}={w}"));
        }
    }
    let mut t = zeilen.join("\n");
    t.push('\n');
    t
}

pub fn env_schreiben(
    o: &Orte,
    aenderungen: &[(&str, Option<String>)],
    zeit: &str,
) -> Result<Option<PathBuf>, String> {
    let pfad = o.env_datei();
    let alt = lesen_oder_leer(&pfad);
    let neu = env_text(&alt, aenderungen);
    if neu == alt {
        return Ok(None);
    }
    let io = |e: std::io::Error| e.to_string();
    let sicherung = sichern(&pfad, zeit).map_err(io)?;
    atomar(&pfad, &neu).map_err(io)?;
    Ok(sicherung)
}

/// Wann eine geschriebene Änderung gilt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Wirkung {
    /// Der laufende labwc liest die Benutzerkopie: `labwc --reconfigure` wirkt.
    Sofort { pid: u32, programm: PathBuf },
    /// labwc liest einen anderen Ordner (Teil A noch nicht aktiv, alter Weg)
    /// oder es gibt keinen labwc dieser Sitzung.
    NaechsteAnmeldung { ordner: Option<PathBuf> },
}

/// Der `-C`-Ordner und das Programm des labwc mit dieser PID (nur eigene Prozesse).
pub fn labwc_von(pid: u32) -> Option<(Option<PathBuf>, PathBuf)> {
    use std::os::unix::fs::MetadataExt;
    let proc = PathBuf::from(format!("/proc/{pid}"));
    let eigen = std::fs::metadata(&proc).ok()?.uid() == std::fs::metadata("/proc/self").ok()?.uid();
    if !eigen {
        return None;
    }
    let programm = std::fs::read_link(proc.join("exe")).ok()?;
    if programm.file_name().is_none_or(|n| n != "labwc") {
        return None;
    }
    let cmd = std::fs::read(proc.join("cmdline")).ok()?;
    let teile: Vec<&[u8]> = cmd.split(|b| *b == 0).collect();
    let ordner = teile
        .iter()
        .position(|t| *t == b"-C" || *t == b"--config-dir")
        .and_then(|i| teile.get(i + 1))
        .map(|d| PathBuf::from(String::from_utf8_lossy(d).into_owned()));
    Some((ordner, programm))
}

pub fn wirkung(o: &Orte) -> Wirkung {
    let Some(pid) = std::env::var("LABWC_PID")
        .ok()
        .and_then(|p| p.trim().parse::<u32>().ok())
    else {
        return Wirkung::NaechsteAnmeldung { ordner: None };
    };
    let Some((ordner, programm)) = labwc_von(pid) else {
        return Wirkung::NaechsteAnmeldung { ordner: None };
    };
    let gleich = ordner.as_ref().is_some_and(|d| {
        match (std::fs::canonicalize(d), std::fs::canonicalize(&o.benutzer)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    });
    if gleich {
        Wirkung::Sofort { pid, programm }
    } else {
        Wirkung::NaechsteAnmeldung { ordner }
    }
}

/// `labwc --reconfigure` für den laufenden labwc (SIGHUP über `LABWC_PID`).
pub fn neu_laden(w: &Wirkung) -> Result<(), String> {
    let Wirkung::Sofort { pid, programm } = w else {
        return Err("labwc liest die Benutzerkopie nicht".into());
    };
    let status = std::process::Command::new(programm)
        .arg("--reconfigure")
        .env("LABWC_PID", pid.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("{} nicht startbar: {e}", programm.display()))?;
    if status.status.success() {
        Ok(())
    } else {
        Err(format!(
            "labwc --reconfigure: {} {}",
            status.status,
            String::from_utf8_lossy(&status.stderr).trim()
        ))
    }
}

/// Neuere Vorgabe übernehmen und die eigenen Werte darauf übertragen
/// (Dreiwegevergleich, [`labwc_xml::auf_neue_vorgabe`]). Danach gilt die
/// neue Vorgabe als Ausgangspunkt, `rc.xml.vorgabe-neu` ist weg.
pub fn vorgabe_uebernehmen(o: &Orte, zeit: &str) -> Result<Option<PathBuf>, String> {
    let ours = std::fs::read_to_string(o.rc()).map_err(|e| format!("rc.xml: {e}"))?;
    let basis =
        std::fs::read_to_string(o.vorgabe_basis()).map_err(|e| format!(".vorgabe/rc.xml: {e}"))?;
    let neue =
        std::fs::read_to_string(o.vorgabe_neu()).map_err(|e| format!("rc.xml.vorgabe-neu: {e}"))?;
    let zusammen = labwc_xml::auf_neue_vorgabe(&ours, &basis, &neue).map_err(|e| e.to_string())?;
    labwc_xml::gueltig(&zusammen).map_err(|e| e.to_string())?;
    let io = |e: std::io::Error| e.to_string();
    let sicherung = sichern(&o.rc(), zeit).map_err(io)?;
    atomar(&o.rc(), &zusammen).map_err(io)?;
    atomar(&o.vorgabe_basis(), &neue).map_err(io)?;
    std::fs::remove_file(o.vorgabe_neu()).map_err(io)?;
    Ok(sicherung)
}

/// Text für die Bereiche: wann wirkt „Übernehmen“?
pub fn wirkung_text(w: &Wirkung, stand: &Stand) -> String {
    match (w, stand.quelle) {
        (Wirkung::Sofort { .. }, _) => {
            "Änderungen gelten nach „Übernehmen“ sofort (labwc lädt seine Konfiguration neu).".into()
        }
        (Wirkung::NaechsteAnmeldung { .. }, Quelle::Benutzer) => {
            "Wirksam ab der nächsten Anmeldung. Die laufende Sitzung liest die Benutzerkopie ~/.config/querencia/labwc noch nicht; „Übernehmen“ speichert dort.".into()
        }
        (Wirkung::NaechsteAnmeldung { .. }, _) => {
            "Wirksam ab der nächsten Anmeldung. Es gibt noch keine Benutzerkopie, angezeigt wird die Vorgabe von Querencia; „Übernehmen“ legt ~/.config/querencia/labwc an, die Sitzung liest sie ab der nächsten Anmeldung.".into()
        }
    }
}

/// `--labwc-zeigen`: Orte, Wirkung und die verwalteten Werte als Text (Prüfung).
pub fn zeigen() -> i32 {
    let o = Orte::standard();
    let st = lesen(&o);
    let w = wirkung(&o);
    println!("benutzerkopie={}", o.benutzer.display());
    println!("vorgabe={}", o.vorgabe.display());
    println!("quelle={:?}", st.quelle);
    match &w {
        Wirkung::Sofort { pid, .. } => println!("wirkung=sofort labwc={pid}"),
        Wirkung::NaechsteAnmeldung { ordner } => println!(
            "wirkung=naechste-anmeldung labwc-ordner={}",
            ordner
                .as_ref()
                .map_or("-".into(), |d| d.display().to_string())
        ),
    }
    println!("vorgabe-neu={}", if st.vorgabe_neu { "ja" } else { "nein" });
    for (pfad, schluessel) in labwc_xml::VERWALTET {
        let p: Vec<String> = pfad
            .iter()
            .map(|s| match s.filter {
                labwc_xml::Filter::Ort(o) => format!("{}[{o}]", s.name),
                _ => s.name.to_owned(),
            })
            .collect();
        println!(
            "{}/{}={}",
            p.join("/"),
            schluessel,
            labwc_xml::wert(&st.rc, pfad, schluessel).unwrap_or_else(|| "-".into())
        );
    }
    println!("desktops/names={}", labwc_xml::namen(&st.rc).join("|"));
    for k in [
        "XKB_DEFAULT_LAYOUT",
        "XKB_DEFAULT_VARIANT",
        "XCURSOR_THEME",
        "XCURSOR_SIZE",
    ] {
        println!("env {k}={}", st.env.get(k).map_or("-", String::as_str));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ordner(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("qe-labwc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn orte(wurzel: &Path) -> Orte {
        let o = Orte {
            benutzer: wurzel.join("config/querencia/labwc"),
            vorgabe: wurzel.join("repo/labwc"),
        };
        std::fs::create_dir_all(&o.vorgabe).unwrap();
        std::fs::write(
            o.vorgabe.join("rc.xml"),
            "<?xml version=\"1.0\"?>\n<labwc_config>\n  <desktops number=\"4\" />\n</labwc_config>\n",
        )
        .unwrap();
        std::fs::write(
            o.vorgabe.join("environment"),
            "LANG=en_US.UTF-8\nXKB_DEFAULT_LAYOUT=de\n",
        )
        .unwrap();
        o
    }

    #[test]
    fn ohne_benutzerkopie_gilt_die_vorgabe_und_schreiben_legt_an() {
        let w = ordner("anlegen");
        let o = orte(&w);
        let st = lesen(&o);
        assert_eq!(st.quelle, Quelle::Vorgabe);
        assert_eq!(
            st.env.get("XKB_DEFAULT_LAYOUT").map(String::as_str),
            Some("de")
        );
        let neu = labwc_xml::setzen(&st.rc, labwc_xml::ARBEITSFLAECHEN, "number", "6").unwrap();
        assert_eq!(rc_schreiben(&o, &neu, &st, "1").unwrap(), None);
        assert_eq!(std::fs::read_to_string(o.rc()).unwrap(), neu);
        // Vorgabe als Ausgangspunkt für den Starter notiert
        assert_eq!(
            std::fs::read_to_string(o.vorgabe_basis()).unwrap(),
            st.rc_vorgabe
        );
        // Vorgabe selbst unverändert
        assert!(
            std::fs::read_to_string(o.vorgabe.join("rc.xml"))
                .unwrap()
                .contains("number=\"4\"")
        );
        let st2 = lesen(&o);
        assert_eq!(st2.quelle, Quelle::Benutzer);
        // Sicherung beim zweiten Schreiben
        let neu2 = labwc_xml::setzen(&st2.rc, labwc_xml::ARBEITSFLAECHEN, "number", "2").unwrap();
        let s = rc_schreiben(&o, &neu2, &st2, "2").unwrap().unwrap();
        assert_eq!(std::fs::read_to_string(&s).unwrap(), neu);
        assert_eq!(s.file_name().unwrap(), "rc.xml.vorher-2");
        // keine temporären Dateien
        let reste: Vec<_> = std::fs::read_dir(&o.benutzer)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains(".neu-"))
            .collect();
        assert!(reste.is_empty());
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn ungueltiges_wird_nicht_geschrieben() {
        let w = ordner("ungueltig");
        let o = orte(&w);
        let st = lesen(&o);
        assert!(rc_schreiben(&o, "<labwc_config>", &st, "1").is_err());
        assert!(!o.rc().exists());
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn sicherungen_hoechstens_zehn() {
        let w = ordner("zehn");
        let o = orte(&w);
        for i in 0..15 {
            let st = lesen(&o);
            let neu =
                labwc_xml::setzen(&st.rc, labwc_xml::ARBEITSFLAECHEN, "number", &i.to_string())
                    .unwrap();
            rc_schreiben(&o, &neu, &st, &format!("{i:02}")).unwrap();
        }
        let mut s: Vec<String> = std::fs::read_dir(&o.benutzer)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with("rc.xml.vorher-"))
            .collect();
        s.sort();
        // das erste Schreiben legt die Datei an (keine Sicherung): 14 Sicherungen, die zehn jüngsten bleiben
        assert_eq!(s.len(), 10);
        assert_eq!(s[0], "rc.xml.vorher-05");
        assert_eq!(s[9], "rc.xml.vorher-14");
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn umgebung_eigene_datei_hat_vorrang() {
        let w = ordner("env");
        let o = orte(&w);
        env_schreiben(
            &o,
            &[
                ("XKB_DEFAULT_LAYOUT", Some("us".into())),
                ("XCURSOR_SIZE", Some("32".into())),
            ],
            "1",
        )
        .unwrap();
        let st = lesen(&o);
        assert_eq!(
            st.env.get("XKB_DEFAULT_LAYOUT").map(String::as_str),
            Some("us")
        );
        assert_eq!(
            st.env_ohne_eigene
                .get("XKB_DEFAULT_LAYOUT")
                .map(String::as_str),
            Some("de")
        );
        assert_eq!(st.env.get("LANG").map(String::as_str), Some("en_US.UTF-8"));
        // entfernen → Vorgabe gilt wieder; Sicherung angelegt
        let s = env_schreiben(&o, &[("XKB_DEFAULT_LAYOUT", None)], "2").unwrap();
        assert!(s.is_some_and(|p| !p.to_string_lossy().ends_with(".env")));
        let st = lesen(&o);
        assert_eq!(
            st.env.get("XKB_DEFAULT_LAYOUT").map(String::as_str),
            Some("de")
        );
        assert_eq!(st.env.get("XCURSOR_SIZE").map(String::as_str), Some("32"));
        // nichts geändert → nichts geschrieben
        assert_eq!(
            env_schreiben(&o, &[("XCURSOR_SIZE", Some("32".into()))], "3").unwrap(),
            None
        );
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn env_text_ersetzt_und_entfernt() {
        let alt = "# k\nA=1\nB=2\nA=3\n";
        assert_eq!(env_text(alt, &[("A", Some("9".into()))]), "# k\nA=9\nB=2\n");
        assert_eq!(env_text(alt, &[("A", None)]), "# k\nB=2\n");
        assert_eq!(
            env_text(alt, &[("C", Some("x".into()))]),
            "# k\nA=1\nB=2\nA=3\nC=x\n"
        );
        assert!(env_text("", &[("A", Some("1".into()))]).ends_with("\nA=1\n"));
    }

    #[test]
    fn neue_vorgabe_uebernehmen() {
        let w = ordner("vorgabe");
        let o = orte(&w);
        let st = lesen(&o);
        let ours = labwc_xml::setzen(&st.rc, labwc_xml::TASTATUR, "repeatRate", "40").unwrap();
        rc_schreiben(&o, &ours, &st, "1").unwrap();
        let neue = st.rc_vorgabe.replace("number=\"4\"", "number=\"5\"");
        std::fs::write(o.vorgabe_neu(), &neue).unwrap();
        assert!(lesen(&o).vorgabe_neu);
        vorgabe_uebernehmen(&o, "2").unwrap();
        let rc = std::fs::read_to_string(o.rc()).unwrap();
        assert_eq!(
            labwc_xml::wert(&rc, labwc_xml::TASTATUR, "repeatRate").as_deref(),
            Some("40")
        );
        assert_eq!(
            labwc_xml::wert(&rc, labwc_xml::ARBEITSFLAECHEN, "number").as_deref(),
            Some("5")
        );
        assert_eq!(std::fs::read_to_string(o.vorgabe_basis()).unwrap(), neue);
        assert!(!o.vorgabe_neu().exists());
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn fremder_prozess_ist_kein_labwc() {
        assert_eq!(labwc_von(1), None);
        assert_eq!(labwc_von(std::process::id()), None);
    }
}
