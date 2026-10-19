//! Fähigkeiten des **laufenden** Querencia Desktop ermitteln (QE-01).
//!
//! Gesucht wird der `querencia-desktop`-Prozess derselben Sitzung (gleiches
//! `XDG_RUNTIME_DIR`). Sein Programm wird nur dann mit `--faehigkeiten`
//! aufgerufen, wenn die Datei diese Zeichenkette enthält – ein Desktop ohne die
//! Option (HMP-10a) könnte den Aufruf sonst als zweiten Start missverstehen.
//! Jeder Fehler ergibt die Fähigkeiten von HMP-10a: dann bleibt „Übernehmen“
//! für Farbe, Zentrieren und Kacheln gesperrt, nichts wird vorgetäuscht.
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::modell::Faehigkeiten;

const OPTION: &[u8] = b"--faehigkeiten";

pub fn ermitteln() -> Faehigkeiten {
    match laufender_desktop() {
        Some(programm) => {
            let f = abfragen(&programm);
            eprintln!(
                "querencia-einstellungen: Desktop {} – Farbe {}, Zentrieren {}, Kacheln {}",
                programm.display(),
                f.farbe,
                f.zentrieren,
                f.kacheln
            );
            f
        }
        None => {
            eprintln!(
                "querencia-einstellungen: kein laufender Desktop gefunden – Fähigkeiten wie HMP-10a"
            );
            Faehigkeiten::HMP10A
        }
    }
}

/// Programm des `querencia-desktop`-Prozesses dieser Sitzung.
fn laufender_desktop() -> Option<PathBuf> {
    let eigene = std::env::var_os("XDG_RUNTIME_DIR")?;
    let mut suche = b"XDG_RUNTIME_DIR=".to_vec();
    suche.extend_from_slice(eigene.as_encoded_bytes());
    for eintrag in std::fs::read_dir("/proc").ok()?.flatten() {
        let pfad = eintrag.path();
        let Ok(exe) = std::fs::read_link(pfad.join("exe")) else {
            continue;
        };
        let name = exe
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !name.starts_with("querencia-desktop") || name.ends_with(" (deleted)") {
            continue;
        }
        let Ok(umgebung) = std::fs::read(pfad.join("environ")) else {
            continue;
        };
        if umgebung.split(|b| *b == 0).any(|e| e == suche.as_slice()) {
            return Some(exe);
        }
    }
    None
}

fn kennt_option(programm: &Path) -> bool {
    let Ok(inhalt) = std::fs::read(programm) else {
        return false;
    };
    memchr::memmem::find(&inhalt, OPTION).is_some()
}

fn abfragen(programm: &Path) -> Faehigkeiten {
    if !kennt_option(programm) {
        return Faehigkeiten::HMP10A;
    }
    let Ok(mut kind) = Command::new(programm)
        .arg("--faehigkeiten")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Faehigkeiten::HMP10A;
    };
    // höchstens 2 s; danach beenden und nichts freigeben
    let ende = Instant::now() + Duration::from_secs(2);
    loop {
        match kind.try_wait() {
            Ok(Some(status)) if status.success() => {
                let mut text = String::new();
                if let Some(mut aus) = kind.stdout.take() {
                    let _ = aus.read_to_string(&mut text);
                }
                return Faehigkeiten::aus_ausgabe(&text);
            }
            Ok(Some(_)) | Err(_) => return Faehigkeiten::HMP10A,
            Ok(None) if Instant::now() > ende => {
                let _ = kind.kill();
                let _ = kind.wait();
                return Faehigkeiten::HMP10A;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
        }
    }
}
