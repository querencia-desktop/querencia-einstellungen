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
//! - Sicherungen mit eigenem Muster `<datei>.einstellungen-vorher-<Zeit>`
//!   (0.1.1, [`sichern`]): eine je Datei und Programmlauf, höchstens
//!   [`SICHERUNGEN`]; `<datei>.vorher-<Zeit>` gehört dem Sitzungsstarter
//!   und wird nie gelöscht. Schreiben atomar über eine temporäre Datei;
//! - eine Rückstellung auf den Vorgabewert hinterlässt rc.xml so, wie die
//!   Vorgabe sie hat ([`labwc_xml::angleichen`] gegen [`Stand::rc_basis`]);
//! - wirksam wird es mit `labwc --reconfigure`, aber nur, wenn der laufende
//!   labwc (`LABWC_PID`) wirklich die Benutzerkopie liest; sonst ab der
//!   nächsten Anmeldung.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::i18n::{gettext, gettext_f};
use crate::labwc_xml;

/// Vorgaben im Runtime-Repo (nur lesen). Für Prüfungen: `QE_LABWC_VORGABE`.
pub const VORGABE_STANDARD: &str =
    "~/projects/querencia-runtime/session/querencia/labwc";
/// Eigene Umgebungsdatei. labwc liest `environment.d/*.env` nach
/// `environment`, alphabetisch (scandir/alphasort), die letzte Angabe gilt;
/// „zz-“ hält die Wahl des Benutzers hinter den Vorgaben anderer
/// Querencia-Teile (z. B. `querencia-erscheinungsbild.env`, Thema-3).
pub const ENV_DATEI: &str = "zz-querencia-einstellungen.env";
/// Namensteil der eigenen Sicherungen: `<datei>.einstellungen-vorher-<Zeit>`.
/// Bis 0.1.0 hießen sie wie die des Sitzungsstarters `<datei>.vorher-<Zeit>`
/// (`session/querencia-session`, `konfig_abgleichen`); die lassen sich
/// nicht unterscheiden und bleiben deshalb unberührt liegen.
pub const SICHERUNG: &str = ".einstellungen-vorher-";
/// So viele eigene Sicherungen je Datei bleiben liegen.
pub const SICHERUNGEN: usize = 5;

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
    /// die Vorgabe, aus der die Benutzerkopie stammt: `.vorgabe/rc.xml`,
    /// sonst die aktuelle Vorgabe. Gleicht rc.xml ihr byte-gleich, gilt sie
    /// dem Sitzungsstarter als unverändert (Dreiwegevergleich).
    pub rc_basis: String,
    /// wirksame Umgebung: environment (Benutzer, sonst Vorgabe), dann
    /// environment.d/*.env nach Bytes sortiert (labwc sortiert mit strcoll;
    /// für Namen aus Kleinbuchstaben, Ziffern und Bindestrich gleich)
    pub env: BTreeMap<String, String>,
    /// dasselbe ohne die eigene Datei (= „Standard“ für Tastatur/Zeiger)
    pub env_ohne_eigene: BTreeMap<String, String>,
    /// eine neuere Vorgabe liegt bereit, die der Starter nicht übernommen hat
    pub vorgabe_neu: bool,
    /// Benutzer-rc.xml vorhanden, aber nicht lesbar (z. B. Rechte): dann
    /// wird nichts angezeigt oder geschrieben, statt still die Vorgabe zu zeigen
    pub fehler: Option<String>,
    /// G2-AJ: Werte aus `org.gnome.desktop.interface`, die der Bereich liest
    /// (füllt das Seitengerüst, [`lesen`] lässt die Liste leer)
    pub gs: BTreeMap<String, String>,
}

/// Zeilen `NAME=WERT` (`#` am Zeilenanfang = Kommentar), nur zur Anzeige:
/// anders als labwc ohne Ersetzung von `$VAR`/`${VAR}` und `~`.
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
    let rc_basis =
        std::fs::read_to_string(o.vorgabe_basis()).unwrap_or_else(|_| rc_vorgabe.clone());
    let mut fehler = None;
    let (rc, quelle) = match std::fs::read_to_string(o.rc()) {
        Ok(t) => (t, Quelle::Benutzer),
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            fehler = Some(gettext_f(
                "{file} not readable ({error})",
                &[
                    ("file", &o.rc().display().to_string()),
                    ("error", &e.to_string()),
                ],
            ));
            (String::new(), Quelle::Benutzer)
        }
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
        rc_basis,
        env,
        env_ohne_eigene,
        vorgabe_neu: o.vorgabe_neu().is_file(),
        fehler,
        gs: BTreeMap::new(),
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

/// Was dieser Programmlauf je Datei zuletzt geschrieben hat (`None` =
/// entfernt), und seine Sicherung des Stands davor (`None` = die Datei gab
/// es vor dem ersten Schreiben nicht).
type Lauf = std::collections::HashMap<PathBuf, (Option<Vec<u8>>, Option<PathBuf>)>;
static LAUF: std::sync::Mutex<Option<Lauf>> = std::sync::Mutex::new(None);

fn lauf<T>(f: impl FnOnce(&mut Lauf) -> T) -> T {
    let mut g = LAUF.lock().unwrap_or_else(|e| e.into_inner());
    f(g.get_or_insert_with(Lauf::new))
}

/// Die eigenen Sicherungen einer Datei, älteste zuerst (der Zeitstempel im
/// Namen sortiert so).
pub fn sicherungen(pfad: &Path) -> Vec<PathBuf> {
    let (Some(ordner), Some(name)) = (pfad.parent(), pfad.file_name()) else {
        return Vec::new();
    };
    let praefix = format!("{}{SICHERUNG}", name.to_string_lossy());
    let mut liste: Vec<PathBuf> = std::fs::read_dir(ordner)
        .map(|e| {
            e.flatten()
                .map(|d| d.path())
                .filter(|p| {
                    p.is_file()
                        && p.file_name()
                            .is_some_and(|n| n.to_string_lossy().starts_with(&praefix))
                })
                .collect()
        })
        .unwrap_or_default();
    liste.sort();
    liste
}

/// Sichert den Stand einer vorhandenen Datei vor dem Schreiben als
/// `<name>.einstellungen-vorher-<zeit>` – einmal je Programmlauf: Hat dieser
/// Lauf die Datei schon gesichert und seither als Letzter geschrieben, reicht
/// diese Sicherung (Werkstatt QREL-06: acht Sicherungen aus einem Ändern und
/// Zurückstellen). Hat jemand anders sie inzwischen geändert, gibt es eine
/// neue.
fn sichern(pfad: &Path, zeit: &str) -> std::io::Result<Option<PathBuf>> {
    if !pfad.is_file() {
        return Ok(None);
    }
    let jetzt = std::fs::read(pfad)?;
    let schon = lauf(|l| {
        l.get(pfad).is_some_and(|(zuletzt, s)| {
            zuletzt.as_deref() == Some(&jetzt[..]) && s.as_ref().is_none_or(|s| s.is_file())
        })
    });
    if schon {
        return Ok(None);
    }
    let name = pfad
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut ziel = pfad.with_file_name(format!("{name}{SICHERUNG}{zeit}"));
    let mut n = 2;
    // gleiche Sekunde: gleicher Inhalt wird wiederverwendet, sonst „-2“, „-3“ …
    while ziel.exists() && std::fs::read(&ziel).ok().as_deref() != Some(&jetzt[..]) {
        ziel = pfad.with_file_name(format!("{name}{SICHERUNG}{zeit}-{n}"));
        n += 1;
    }
    if !ziel.exists() {
        std::fs::copy(pfad, &ziel)?;
    }
    lauf(|l| l.insert(pfad.to_owned(), (None, Some(ziel.clone()))));
    Ok(Some(ziel))
}

/// Nach dem Schreiben (`inhalt`, `None` = Datei entfernt): merken, eigene
/// Sicherungen mit genau diesem Inhalt entfernen (der Stand liegt ja als
/// Datei vor – Ändern und Zurückstellen hinterlässt so keine), dann nur die
/// [`SICHERUNGEN`] jüngsten behalten. Fremde Dateien bleiben.
fn nachher(pfad: &Path, inhalt: Option<&[u8]>) {
    lauf(|l| l.entry(pfad.to_owned()).or_insert((None, None)).0 = inhalt.map(<[u8]>::to_vec));
    let mut liste = sicherungen(pfad);
    if let Some(i) = inhalt {
        liste.retain(|s| {
            let gleich = std::fs::read(s).ok().as_deref() == Some(i);
            if gleich {
                let _ = std::fs::remove_file(s);
            }
            !gleich
        });
    }
    let zuviel = liste.len().saturating_sub(SICHERUNGEN);
    for s in liste.into_iter().take(zuviel) {
        let _ = std::fs::remove_file(s);
    }
}

/// Für Prüfungen: einen neuen Programmlauf nachstellen (Dateien unter `ordner`).
#[cfg(test)]
pub fn lauf_vergessen(ordner: &Path) {
    lauf(|l| l.retain(|p, _| !p.starts_with(ordner)));
}

/// Sichern, atomar schreiben, aufräumen.
fn ersetzen_mit_sicherung(
    pfad: &Path,
    inhalt: &str,
    zeit: &str,
) -> std::io::Result<Option<PathBuf>> {
    let sicherung = sichern(pfad, zeit)?;
    atomar(pfad, inhalt)?;
    nachher(pfad, Some(inhalt.as_bytes()));
    Ok(sicherung.filter(|s| s.is_file()))
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
        return Err(gettext("an empty rc.xml is not written"));
    }
    let io = |e: std::io::Error| e.to_string();
    std::fs::create_dir_all(&o.benutzer).map_err(io)?;
    if stand.quelle == Quelle::Vorgabe && !o.vorgabe_basis().exists() {
        atomar(&o.vorgabe_basis(), &stand.rc_vorgabe).map_err(io)?;
    }
    ersetzen_mit_sicherung(&o.rc(), neu, zeit).map_err(io)
}

/// Ändert die eigene Umgebungsdatei: `Some(wert)` setzt, `None` entfernt.
/// Andere Zeilen bleiben; ein Schlüssel steht danach höchstens einmal da.
/// Kopf einer neu angelegten eigenen Umgebungsdatei.
const ENV_KOPF: &[&str] = &[
    "# Querencia Einstellungen (Tastatur, Maus, Sprache und Region). labwc liest",
    "# diese Datei nach ../environment; Werte hier haben Vorrang. Wirksam mit",
    "# labwc --reconfigure, für schon laufende Programme und Sprache/Formate",
    "# ab der nächsten Anmeldung.",
];

pub fn env_text(alt: &str, aenderungen: &[(&str, Option<String>)]) -> String {
    let mut zeilen: Vec<String> = if alt.is_empty() {
        ENV_KOPF.iter().map(|z| (*z).to_owned()).collect()
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
    if neu == env_text("", &[]) {
        // nur noch der eigene Kopf: Die Datei verschwindet wieder, wie vor
        // dem ersten Übernehmen (Rückstellung ohne Rest, 0.1.1)
        let sicherung = sichern(&pfad, zeit).map_err(io)?;
        std::fs::remove_file(&pfad).map_err(io)?;
        nachher(&pfad, None);
        // ein leer gewordenes environment.d auch (remove_dir scheitert, wenn
        // noch etwas darin liegt)
        if let Some(d) = pfad.parent() {
            let _ = std::fs::remove_dir(d);
        }
        return Ok(sicherung.filter(|s| s.is_file()));
    }
    ersetzen_mit_sicherung(&pfad, &neu, zeit).map_err(io)
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
        return Err(gettext("labwc does not read the user copy"));
    };
    let status = std::process::Command::new(programm)
        .arg("--reconfigure")
        .env("LABWC_PID", pid.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| {
            gettext_f(
                "{program} cannot be started: {error}",
                &[
                    ("program", &programm.display().to_string()),
                    ("error", &e.to_string()),
                ],
            )
        })?;
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
    let sicherung = ersetzen_mit_sicherung(&o.rc(), &zusammen, zeit).map_err(io)?;
    atomar(&o.vorgabe_basis(), &neue).map_err(io)?;
    std::fs::remove_file(o.vorgabe_neu()).map_err(io)?;
    Ok(sicherung)
}

/// Text für die Bereiche: wann wirkt „Übernehmen“?
pub fn wirkung_text(w: &Wirkung, stand: &Stand) -> String {
    match (w, stand.quelle) {
        (Wirkung::Sofort { .. }, _) => gettext(
            "Changes take effect immediately after “Apply” (labwc reloads its configuration).",
        ),
        (Wirkung::NaechsteAnmeldung { .. }, Quelle::Benutzer) => gettext(
            "Effective from the next login. The running session does not read the user copy ~/.config/querencia/labwc yet; “Apply” saves there.",
        ),
        (Wirkung::NaechsteAnmeldung { .. }, _) => gettext(
            "Effective from the next login. There is no user copy yet, Querencia’s default is shown; “Apply” creates ~/.config/querencia/labwc, the session reads it from the next login.",
        ),
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
    if let Some(f) = &st.fehler {
        println!("fehler={f}");
    }
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
        "XKB_DEFAULT_OPTIONS",
        "XCURSOR_THEME",
        "XCURSOR_SIZE",
        "LANG",
        "LANGUAGE",
        "LC_TIME",
        "LC_NUMERIC",
        "LC_MONETARY",
        "LC_PAPER",
        "LC_MEASUREMENT",
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
        // Sicherung beim Schreiben im nächsten Programmlauf
        lauf_vergessen(&w);
        let neu2 = labwc_xml::setzen(&st2.rc, labwc_xml::ARBEITSFLAECHEN, "number", "2").unwrap();
        let s = rc_schreiben(&o, &neu2, &st2, "2").unwrap().unwrap();
        assert_eq!(std::fs::read_to_string(&s).unwrap(), neu);
        assert_eq!(s.file_name().unwrap(), "rc.xml.einstellungen-vorher-2");
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

    fn namen_im(o: &Orte, praefix: &str) -> Vec<String> {
        let mut s: Vec<String> = std::fs::read_dir(&o.benutzer)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(praefix))
            .collect();
        s.sort();
        s
    }

    fn anzahl_setzen(o: &Orte, n: &str, zeit: &str) -> Option<PathBuf> {
        let st = lesen(o);
        let neu = labwc_xml::setzen(&st.rc, labwc_xml::ARBEITSFLAECHEN, "number", n).unwrap();
        rc_schreiben(o, &neu, &st, zeit).unwrap()
    }

    #[test]
    fn sicherungen_hoechstens_fuenf_fremde_bleiben() {
        let w = ordner("fuenf");
        let o = orte(&w);
        // Sicherungen des Sitzungsstarters (und der Fassungen bis 0.1.0)
        std::fs::create_dir_all(&o.benutzer).unwrap();
        for n in [
            "rc.xml.vorher-20261020-112812",
            "rc.xml.vorher-20261020-201950",
        ] {
            std::fs::write(o.benutzer.join(n), "fremd").unwrap();
        }
        // 15 Programmläufe mit je einem Übernehmen
        for i in 0..15 {
            lauf_vergessen(&w);
            anzahl_setzen(&o, &(i + 1).to_string(), &format!("{i:02}"));
        }
        let s = namen_im(&o, "rc.xml.einstellungen-vorher-");
        // das erste Schreiben legt die Datei an (keine Sicherung): 14 Sicherungen, die fünf jüngsten bleiben
        assert_eq!(
            s,
            ["10", "11", "12", "13", "14"].map(|z| format!("rc.xml.einstellungen-vorher-{z}"))
        );
        assert_eq!(
            namen_im(&o, "rc.xml.vorher-"),
            [
                "rc.xml.vorher-20261020-112812",
                "rc.xml.vorher-20261020-201950"
            ]
        );
        for n in namen_im(&o, "rc.xml.vorher-") {
            assert_eq!(
                std::fs::read_to_string(o.benutzer.join(n)).unwrap(),
                "fremd"
            );
        }
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn eine_sicherung_je_lauf_und_keine_nach_zurueckstellen() {
        let w = ordner("lauf");
        let o = orte(&w);
        anzahl_setzen(&o, "4", "00");
        let ausgang = std::fs::read(o.rc()).unwrap();
        lauf_vergessen(&w);
        // ein Lauf: achtmal Übernehmen, eine Sicherung (der Stand vor dem Lauf)
        let erste = anzahl_setzen(&o, "5", "01").unwrap();
        assert_eq!(std::fs::read(&erste).unwrap(), ausgang);
        for (i, n) in ["6", "7", "3", "2", "8", "9"].iter().enumerate() {
            assert_eq!(anzahl_setzen(&o, n, &format!("1{i}")), None);
        }
        assert_eq!(namen_im(&o, "rc.xml.einstellungen-vorher-").len(), 1);
        // zurück auf den Ausgang: die Sicherung ist überflüssig und geht
        anzahl_setzen(&o, "4", "20");
        assert_eq!(std::fs::read(o.rc()).unwrap(), ausgang);
        assert!(namen_im(&o, "rc.xml.einstellungen-vorher-").is_empty());
        // danach ändert jemand anders die Datei: neue Sicherung im selben Lauf
        anzahl_setzen(&o, "5", "21").unwrap();
        let fremd = std::fs::read_to_string(o.rc())
            .unwrap()
            .replace("number=\"5\"", "number=\"6\"");
        std::fs::write(o.rc(), &fremd).unwrap();
        let s = anzahl_setzen(&o, "7", "22").unwrap();
        assert_eq!(std::fs::read_to_string(s).unwrap(), fremd);
        assert_eq!(namen_im(&o, "rc.xml.einstellungen-vorher-").len(), 2);
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn gleiche_sekunde_ueberschreibt_keine_sicherung() {
        let w = ordner("sekunde");
        let o = orte(&w);
        anzahl_setzen(&o, "4", "00");
        lauf_vergessen(&w);
        let a = anzahl_setzen(&o, "5", "x").unwrap();
        lauf_vergessen(&w);
        let b = anzahl_setzen(&o, "6", "x").unwrap();
        assert_ne!(a, b);
        assert_eq!(b.file_name().unwrap(), "rc.xml.einstellungen-vorher-x-2");
        assert!(
            std::fs::read_to_string(&a)
                .unwrap()
                .contains("number=\"4\"")
        );
        assert!(
            std::fs::read_to_string(&b)
                .unwrap()
                .contains("number=\"5\"")
        );
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn umgebung_zurueckgestellt_ohne_rest() {
        let w = ordner("env-zurueck");
        let o = orte(&w);
        env_schreiben(&o, &[("XKB_DEFAULT_LAYOUT", Some("us".into()))], "1").unwrap();
        assert!(o.env_datei().is_file());
        env_schreiben(&o, &[("XKB_DEFAULT_LAYOUT", None)], "2").unwrap();
        assert!(!o.env_datei().exists());
        // keine Sicherung, kein leerer Ordner environment.d
        assert!(!o.env_datei().parent().unwrap().exists());
        // eine fremde Datei hält den Ordner
        std::fs::create_dir_all(o.env_datei().parent().unwrap()).unwrap();
        std::fs::write(o.benutzer.join("environment.d/querencia-x.env"), "B=1\n").unwrap();
        env_schreiben(&o, &[("XKB_DEFAULT_LAYOUT", Some("us".into()))], "2a").unwrap();
        env_schreiben(&o, &[("XKB_DEFAULT_LAYOUT", None)], "2b").unwrap();
        assert!(!o.env_datei().exists());
        assert!(o.benutzer.join("environment.d/querencia-x.env").is_file());
        std::fs::remove_file(o.benutzer.join("environment.d/querencia-x.env")).unwrap();
        // fremde Zeilen halten die Datei
        std::fs::write(o.env_datei(), "# eigen\nA=1\n").unwrap();
        lauf_vergessen(&w);
        env_schreiben(&o, &[("XKB_DEFAULT_LAYOUT", Some("us".into()))], "3").unwrap();
        env_schreiben(&o, &[("XKB_DEFAULT_LAYOUT", None)], "4").unwrap();
        assert_eq!(
            std::fs::read_to_string(o.env_datei()).unwrap(),
            "# eigen\nA=1\n"
        );
        assert!(sicherungen(&o.env_datei()).is_empty());
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
        // entfernen → Vorgabe gilt wieder; Sicherung im nächsten Programmlauf
        lauf_vergessen(&w);
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
    fn unlesbare_benutzerdatei_ist_ein_fehler_kein_rueckfall() {
        let w = ordner("unlesbar");
        let o = orte(&w);
        // ein Ordner an Stelle der Datei: lesen scheitert nicht mit NotFound
        std::fs::create_dir_all(o.rc()).unwrap();
        let st = lesen(&o);
        assert_eq!(st.quelle, Quelle::Benutzer);
        assert!(st.rc.is_empty());
        assert!(
            st.fehler
                .as_deref()
                .is_some_and(|f| f.contains("not readable"))
        );
        std::fs::remove_dir_all(&w).unwrap();
    }

    #[test]
    fn fremder_prozess_ist_kein_labwc() {
        assert_eq!(labwc_von(1), None);
        assert_eq!(labwc_von(std::process::id()), None);
    }
}
