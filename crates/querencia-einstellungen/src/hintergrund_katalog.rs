//! Eingebaute Hintergründe (G2-AF, QD-058), ohne GTK.
//!
//! Die Dateien liegen wie das bisherige „Querencia Blau“ in
//! `querencia/hintergruende/` der XDG-Datenordner (Benutzer:
//! `~/.local/share`, Pakete: `/usr/share`) und heißen
//! `querencia-<stamm>[-logo][-4x3].png` (Quelle:
//! `querencia-runtime/data/hintergruende/`, erzeugt von `erzeugen.py`).
//! „Querencia Blau“ gibt es nur als 16:9 ohne Logo und bleibt der Standard:
//! wer es wählt, schreibt `File=` leer wie bisher.
//!
//! Das Seitenverhältnis entscheidet die Datei: unter [`GRENZE`] (5:4, 4:3,
//! 3:2) die 4:3-Fassung, sonst (16:10, 16:9 und breiter) die 16:9-Fassung.
use std::path::{Path, PathBuf};

/// Mitte zwischen 4:3 (1,333) und 16:9 (1,778); 16:10 (1,6) bekommt 16:9.
pub const GRENZE: f64 = 1.555;

/// Ein Motiv der Galerie.
#[derive(Debug, PartialEq, Eq)]
pub struct Motiv {
    pub name: &'static str,
    pub stamm: &'static str,
    /// Gibt es eine Fassung mit aufgeprägtem Q-Logo?
    pub logo: bool,
    /// Gibt es eine eigene 4:3-Fassung?
    pub vier_drei: bool,
}

/// Reihenfolge der Galerie; das erste Motiv ist der Standard.
pub const MOTIVE: &[Motiv] = &[
    Motiv {
        name: "Querencia Blau",
        stamm: "blau",
        logo: false,
        vier_drei: false,
    },
    Motiv {
        name: "Querencia Abend",
        stamm: "abend",
        logo: true,
        vier_drei: true,
    },
    Motiv {
        name: "Querencia Blau 2",
        stamm: "blau2",
        logo: true,
        vier_drei: true,
    },
    Motiv {
        name: "Querencia Erde",
        stamm: "erde",
        logo: true,
        vier_drei: true,
    },
    Motiv {
        name: "Querencia Nacht",
        stamm: "nacht",
        logo: true,
        vier_drei: true,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Breit,
    VierDrei,
}

impl Format {
    /// Passende Fassung für eine Fläche von `breite`×`hoehe`.
    pub fn fuer(breite: f64, hoehe: f64) -> Self {
        if hoehe > 0.0 && breite / hoehe < GRENZE {
            Self::VierDrei
        } else {
            Self::Breit
        }
    }

    pub fn beschriftung(self) -> &'static str {
        match self {
            Self::Breit => "16:9",
            Self::VierDrei => "4:3",
        }
    }
}

/// Eine Kachel der Galerie: Motiv (Index in [`MOTIVE`]) mit oder ohne Logo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wahl {
    pub motiv: usize,
    pub logo: bool,
}

impl Wahl {
    pub const STANDARD: Self = Self {
        motiv: 0,
        logo: false,
    };

    fn motiv(self) -> &'static Motiv {
        &MOTIVE[self.motiv]
    }

    /// Kurzname für die Kachel (ohne „Querencia“), z. B. „Abend mit Logo“.
    pub fn kurzname(self) -> String {
        let name = self.name();
        name.strip_prefix("Querencia ")
            .map_or(name.clone(), str::to_string)
    }

    pub fn name(self) -> String {
        if self.logo {
            format!("{} mit Logo", self.motiv().name)
        } else {
            self.motiv().name.to_string()
        }
    }

    /// Dateiname der Fassung; ohne eigene 4:3-Fassung die 16:9-Datei.
    pub fn dateiname(self, format: Format) -> String {
        let m = self.motiv();
        let logo = if self.logo && m.logo { "-logo" } else { "" };
        let format = if format == Format::VierDrei && m.vier_drei {
            "-4x3"
        } else {
            ""
        };
        format!("querencia-{}{logo}{format}.png", m.stamm)
    }

    /// Erste vorhandene Datei der Fassung in `ordner`; fehlt die 4:3-Fassung,
    /// die 16:9-Fassung.
    pub fn finden(
        self,
        format: Format,
        ordner: &[PathBuf],
        vorhanden: impl Fn(&Path) -> bool,
    ) -> Option<PathBuf> {
        let suchen = |name: String| ordner.iter().map(|o| o.join(&name)).find(|p| vorhanden(p));
        suchen(self.dateiname(format)).or_else(|| suchen(self.dateiname(Format::Breit)))
    }
}

/// Alle Kacheln in Galerie-Reihenfolge: je Motiv ohne, dann mit Logo.
pub fn kacheln() -> Vec<Wahl> {
    MOTIVE
        .iter()
        .enumerate()
        .flat_map(|(i, m)| {
            let ohne = Wahl {
                motiv: i,
                logo: false,
            };
            let mit = m.logo.then_some(Wahl {
                motiv: i,
                logo: true,
            });
            std::iter::once(ohne).chain(mit)
        })
        .collect()
}

/// Welche Kachel gehört zu einer Datei in einem `querencia/hintergruende/`-Ordner?
/// Fremde Ordner und Namen ergeben `None`.
pub fn erkennen(pfad: &Path) -> Option<Wahl> {
    let ordner = pfad.parent()?;
    if ordner.file_name()? != "hintergruende" || ordner.parent()?.file_name()? != "querencia" {
        return None;
    }
    let name = pfad.file_name()?.to_str()?;
    kacheln()
        .into_iter()
        .find(|w| name == w.dateiname(Format::Breit) || name == w.dateiname(Format::VierDrei))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seitenverhaeltnis_waehlt_die_fassung() {
        assert_eq!(Format::fuer(1024.0, 768.0), Format::VierDrei);
        assert_eq!(Format::fuer(1600.0, 1200.0), Format::VierDrei);
        assert_eq!(Format::fuer(1280.0, 1024.0), Format::VierDrei);
        assert_eq!(Format::fuer(1440.0, 960.0), Format::VierDrei);
        assert_eq!(Format::fuer(1920.0, 1200.0), Format::Breit);
        assert_eq!(Format::fuer(1920.0, 1080.0), Format::Breit);
        assert_eq!(Format::fuer(2560.0, 1440.0), Format::Breit);
        assert_eq!(Format::fuer(3440.0, 1440.0), Format::Breit);
        assert_eq!(Format::fuer(0.0, 0.0), Format::Breit);
    }

    #[test]
    fn neun_kacheln_blau_zuerst_ohne_logo() {
        let k = kacheln();
        assert_eq!(k.len(), 9);
        assert_eq!(k[0], Wahl::STANDARD);
        assert_eq!(k[0].name(), "Querencia Blau");
        assert_eq!(k[1].name(), "Querencia Abend");
        assert_eq!(k[2].name(), "Querencia Abend mit Logo");
        assert_eq!(k[8].name(), "Querencia Nacht mit Logo");
        assert_eq!(k[0].kurzname(), "Blau");
        assert_eq!(k[2].kurzname(), "Abend mit Logo");
    }

    #[test]
    fn dateinamen_wie_im_runtime_repo() {
        let namen: Vec<String> = kacheln()
            .iter()
            .flat_map(|w| [w.dateiname(Format::Breit), w.dateiname(Format::VierDrei)])
            .collect();
        for n in [
            "querencia-blau.png",
            "querencia-abend.png",
            "querencia-abend-4x3.png",
            "querencia-abend-logo.png",
            "querencia-abend-logo-4x3.png",
            "querencia-blau2-logo-4x3.png",
            "querencia-erde-logo.png",
            "querencia-nacht-4x3.png",
        ] {
            assert!(namen.iter().any(|x| x == n), "{n} fehlt");
        }
        // Blau hat keine 4:3-Fassung: immer die eine Datei.
        assert_eq!(
            Wahl::STANDARD.dateiname(Format::VierDrei),
            crate::modell::EINGEBAUT
        );
        // 16 neue + Blau
        let mut einzeln = namen.clone();
        einzeln.sort();
        einzeln.dedup();
        assert_eq!(einzeln.len(), 17);
    }

    #[test]
    fn finden_faellt_auf_16_9_zurueck_und_nimmt_den_ersten_ordner() {
        let ordner = vec![
            PathBuf::from("/home/q/.local/share/querencia/hintergruende"),
            PathBuf::from("/usr/share/querencia/hintergruende"),
        ];
        let abend = Wahl {
            motiv: 1,
            logo: true,
        };
        let alles = |_: &Path| true;
        assert_eq!(
            abend.finden(Format::VierDrei, &ordner, alles),
            Some(ordner[0].join("querencia-abend-logo-4x3.png"))
        );
        let nur_system_breit = |p: &Path| {
            p == Path::new("/usr/share/querencia/hintergruende/querencia-abend-logo.png")
        };
        assert_eq!(
            abend.finden(Format::VierDrei, &ordner, nur_system_breit),
            Some(ordner[1].join("querencia-abend-logo.png"))
        );
        assert_eq!(abend.finden(Format::Breit, &ordner, |_| false), None);
    }

    #[test]
    fn erkennen_nur_in_querencia_hintergruende() {
        let o = Path::new("/home/q/.local/share/querencia/hintergruende");
        assert_eq!(
            erkennen(&o.join("querencia-erde-logo-4x3.png")),
            Some(Wahl {
                motiv: 3,
                logo: true
            })
        );
        assert_eq!(
            erkennen(&o.join("querencia-blau2.png")),
            Some(Wahl {
                motiv: 2,
                logo: false
            })
        );
        assert_eq!(
            erkennen(&o.join("querencia-blau.png")),
            Some(Wahl::STANDARD)
        );
        assert_eq!(erkennen(&o.join("urlaub.png")), None);
        assert_eq!(
            erkennen(Path::new("/home/q/Bilder/querencia-erde.png")),
            None
        );
        assert_eq!(erkennen(Path::new("querencia-erde.png")), None);
    }
}
