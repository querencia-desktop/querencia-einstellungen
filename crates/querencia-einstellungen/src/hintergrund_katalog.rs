//! Eingebaute Hintergründe (G2-AF, QD-058), ohne GTK.
//!
//! Die Dateien liegen wie das bisherige „Querencia Blau“ in
//! `querencia/hintergruende/` der XDG-Datenordner (Benutzer:
//! `~/.local/share`, Pakete: `/usr/share`) und heißen
//! `querencia-<stamm>[-logo][-4x3].png` (Quelle:
//! `querencia-runtime/data/hintergruende/`, erzeugt von `erzeugen.py`, ab
//! Hintergründe 0.2 von `erzeugen-02.py`).
//! Standard ist seit 0.1.4 „Querencia Tiefblau“ ([`Wahl::STANDARD`], erste
//! Kachel, leeres `File=` wie `BUILT_IN` ab Desktop 0.2.1). „Querencia Blau“
//! ([`Wahl::BLAU`], nur 16:9 ohne Logo) war es bis Desktop 0.2.0.
//!
//! Das Seitenverhältnis entscheidet die Datei: unter [`GRENZE`] (5:4, 4:3,
//! 3:2) die 4:3-Fassung, sonst (16:10, 16:9 und breiter) die 16:9-Fassung.
use crate::i18n::{gettext, gettext_f, gettext_noop};
use std::path::{Path, PathBuf};

/// Mitte zwischen 4:3 (1,333) und 16:9 (1,778); 16:10 (1,6) bekommt 16:9.
pub const GRENZE: f64 = 1.555;

/// Zeichen eines Namens, die auf eine Kachel passen (wie `max_width_chars`).
pub const KACHEL_ZEICHEN: usize = 16;

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

/// Reihenfolge der Galerie; das erste Motiv ist der Standard (0.1.4:
/// Tiefblau). Danach die übrigen Motive von Hintergründe 0.2 (Verläufe,
/// Landschaften, Logo-Fassungen; jede Fassung ein eigenes Motiv mit eigenem
/// Stamm), dann die Bilder von 0.1.
pub const MOTIVE: &[Motiv] = &[
    Motiv {
        name: gettext_noop("Querencia Deep Blue"),
        stamm: "tiefblau",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Blue Green"),
        stamm: "blaugruen",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Blue Glow"),
        stamm: "blauschein",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Ember"),
        stamm: "glut",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Hills"),
        stamm: "huegel",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Coast"),
        stamm: "kueste",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Dunes"),
        stamm: "duenen",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Lake"),
        stamm: "see",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Desert"),
        stamm: "wueste",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Mountains"),
        stamm: "berge",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Beach"),
        stamm: "strand",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Glass"),
        stamm: "glas",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Deep Blue with logo"),
        stamm: "tiefblau-logo",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Deep Blue with wordmark"),
        stamm: "tiefblau-wortmarke",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Deep Blue with watermark"),
        stamm: "tiefblau-wasserzeichen",
        logo: false,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Amber with logo"),
        stamm: "bernstein-logo",
        logo: false,
        vier_drei: true,
    },
    // Hintergründe 0.1 (G2-AF)
    Motiv {
        name: gettext_noop("Querencia Blue"),
        stamm: "blau",
        logo: false,
        vier_drei: false,
    },
    Motiv {
        name: gettext_noop("Querencia Evening"),
        stamm: "abend",
        logo: true,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Blue 2"),
        stamm: "blau2",
        logo: true,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Earth"),
        stamm: "erde",
        logo: true,
        vier_drei: true,
    },
    Motiv {
        name: gettext_noop("Querencia Night"),
        stamm: "nacht",
        logo: true,
        vier_drei: true,
    },
];

/// Index des Motivs mit dem Stamm `stamm` in [`MOTIVE`] (zur Übersetzungszeit).
const fn index(stamm: &str) -> usize {
    let mut i = 0;
    while i < MOTIVE.len() {
        let a = MOTIVE[i].stamm.as_bytes();
        let b = stamm.as_bytes();
        if a.len() == b.len() {
            let mut j = 0;
            while j < a.len() && a[j] == b[j] {
                j += 1;
            }
            if j == a.len() {
                return i;
            }
        }
        i += 1;
    }
    panic!("Stamm fehlt im Katalog")
}

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
    /// Standard (0.1.4: „Querencia Tiefblau“, erste Kachel).
    pub const STANDARD: Self = Self {
        motiv: 0,
        logo: false,
    };

    /// „Querencia Blau“, der Standard bis 0.1.3 (Desktop bis 0.2.0).
    pub const BLAU: Self = Self {
        motiv: index("blau"),
        logo: false,
    };

    /// Motiv von Hintergründe 0.2 (alle vor „Querencia Blau“ in [`MOTIVE`])?
    pub fn aus_0_2(self) -> bool {
        self.motiv < Self::BLAU.motiv
    }

    fn motiv(self) -> &'static Motiv {
        &MOTIVE[self.motiv]
    }

    /// Kurzname für die Kachel (ohne „Querencia“), z. B. „Abend mit Logo“.
    pub fn kurzname(self) -> String {
        let name = self.name();
        name.strip_prefix("Querencia ")
            .map_or(name.clone(), str::to_string)
    }

    /// Name auf der Kachel: der Kurzname; ist er bei einem Motiv von
    /// Hintergründe 0.2 länger als [`KACHEL_ZEICHEN`], in zwei Zeilen,
    /// getrennt an der Leerstelle, die die längere Zeile am kürzesten macht.
    pub fn kachelname(self) -> String {
        let name = self.kurzname();
        let laenge = name.chars().count();
        if !self.aus_0_2() || laenge <= KACHEL_ZEICHEN {
            return name;
        }
        let trenner = name
            .char_indices()
            .enumerate()
            .filter(|(_, (_, c))| *c == ' ')
            .min_by_key(|(z, _)| (*z).max(laenge - z - 1));
        match trenner {
            Some((_, (b, _))) => format!("{}\n{}", &name[..b], &name[b + 1..]),
            None => name,
        }
    }

    pub fn name(self) -> String {
        if self.logo {
            gettext_f("{name} with logo", &[("name", &gettext(self.motiv().name))])
        } else {
            gettext(self.motiv().name)
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

/// Kacheln, deren 16:9-Datei in `ordner` vorhanden ist, mit dieser Datei
/// (für das Vorschaubild); Motive ohne installierte Datei fehlen, damit die
/// Galerie auch ohne das Datenpaket von Hintergründe 0.2 vollständig bleibt.
pub fn verfuegbar(ordner: &[PathBuf], vorhanden: impl Fn(&Path) -> bool) -> Vec<(Wahl, PathBuf)> {
    kacheln()
        .into_iter()
        .filter_map(|w| w.finden(Format::Breit, ordner, &vorhanden).map(|p| (w, p)))
        .collect()
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
    fn kacheln_tiefblau_zuerst_dann_0_2_dann_0_1() {
        let k = kacheln();
        assert_eq!(k.len(), 25);
        assert_eq!(k[0], Wahl::STANDARD);
        assert_eq!(k[0].name(), "Querencia Deep Blue");
        assert_eq!(k[0].kurzname(), "Deep Blue");
        assert_eq!(k[11].name(), "Querencia Glass");
        assert_eq!(k[12].name(), "Querencia Deep Blue with logo");
        assert_eq!(k[15].name(), "Querencia Amber with logo");
        assert_eq!(k[16], Wahl::BLAU);
        assert_eq!(k[16].name(), "Querencia Blue");
        assert_eq!(k[16].kurzname(), "Blue");
        assert_eq!(k[17].name(), "Querencia Evening");
        assert_eq!(k[18].name(), "Querencia Evening with logo");
        assert_eq!(k[18].kurzname(), "Evening with logo");
        assert_eq!(k[24].name(), "Querencia Night with logo");
        // Die 0.2-Fassungen mit Logo sind eigene Motive ohne zweite Kachel.
        assert!(MOTIVE[..16].iter().all(|m| !m.logo && m.vier_drei));
        assert_eq!(k.iter().filter(|w| w.aus_0_2()).count(), 16);
        assert!(k[15].aus_0_2() && !k[16].aus_0_2() && !k[24].aus_0_2());
    }

    #[test]
    fn kachelnamen_lang_nur_bei_0_2_zweizeilig() {
        let w = |s: &str| Wahl {
            motiv: index(s),
            logo: false,
        };
        // msgid englisch (Testumgebung ohne Katalog)
        assert_eq!(w("tiefblau").kachelname(), "Deep Blue");
        assert_eq!(w("tiefblau-logo").kachelname(), "Deep Blue\nwith logo");
        assert_eq!(
            w("tiefblau-wortmarke").kachelname(),
            "Deep Blue\nwith wordmark"
        );
        assert_eq!(
            w("tiefblau-wasserzeichen").kachelname(),
            "Deep Blue\nwith watermark"
        );
        assert_eq!(w("bernstein-logo").kachelname(), "Amber with logo");
        // 0.1 bleibt einzeilig, auch über 16 Zeichen („Evening with logo“)
        let abend = Wahl {
            motiv: index("abend"),
            logo: true,
        };
        assert_eq!(abend.kachelname(), "Evening with logo");
    }

    #[test]
    fn standard_tiefblau_eingebaut_blau() {
        assert_eq!(MOTIVE[Wahl::STANDARD.motiv].stamm, "tiefblau");
        assert_eq!(MOTIVE[Wahl::BLAU.motiv].stamm, "blau");
        assert_eq!(
            Wahl::STANDARD.dateiname(Format::Breit),
            crate::modell::EINGEBAUT
        );
        assert_eq!(
            Wahl::STANDARD.dateiname(Format::VierDrei),
            "querencia-tiefblau-4x3.png"
        );
        assert_eq!(index("nacht"), MOTIVE.len() - 1);
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
            // Hintergründe 0.2 (docs/hintergruende-02.md im Runtime-Repo)
            "querencia-tiefblau.png",
            "querencia-tiefblau-4x3.png",
            "querencia-blaugruen.png",
            "querencia-blauschein-4x3.png",
            "querencia-glut.png",
            "querencia-huegel-4x3.png",
            "querencia-kueste.png",
            "querencia-duenen.png",
            "querencia-see-4x3.png",
            "querencia-wueste.png",
            "querencia-berge.png",
            "querencia-strand-4x3.png",
            "querencia-glas.png",
            "querencia-tiefblau-logo.png",
            "querencia-tiefblau-logo-4x3.png",
            "querencia-tiefblau-wortmarke.png",
            "querencia-tiefblau-wasserzeichen-4x3.png",
            "querencia-bernstein-logo.png",
            "querencia-bernstein-logo-4x3.png",
        ] {
            assert!(namen.iter().any(|x| x == n), "{n} fehlt");
        }
        // Blau hat keine 4:3-Fassung: immer die eine Datei.
        assert_eq!(
            Wahl::BLAU.dateiname(Format::VierDrei),
            crate::modell::EINGEBAUT_0_1
        );
        // 0.1: 16 + Blau; 0.2: 16 Motive je 16:9 und 4:3
        let mut einzeln = namen.clone();
        einzeln.sort();
        einzeln.dedup();
        assert_eq!(einzeln.len(), 17 + 32);
    }

    /// Dateien eines Ordners wie auf der VM ohne das Datenpaket 0.2.
    fn nur_0_1(p: &Path) -> bool {
        let n = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        ["blau", "abend", "blau2", "erde", "nacht"].iter().any(|s| {
            n.starts_with(&format!("querencia-{s}.")) || n.starts_with(&format!("querencia-{s}-"))
        })
    }

    #[test]
    fn verfuegbar_ohne_datenpaket_0_2_wie_0_1() {
        let ordner = vec![PathBuf::from("/usr/share/querencia/hintergruende")];
        let v = verfuegbar(&ordner, nur_0_1);
        let namen: Vec<String> = v.iter().map(|(w, _)| w.name()).collect();
        assert_eq!(
            namen,
            [
                "Querencia Blue",
                "Querencia Evening",
                "Querencia Evening with logo",
                "Querencia Blue 2",
                "Querencia Blue 2 with logo",
                "Querencia Earth",
                "Querencia Earth with logo",
                "Querencia Night",
                "Querencia Night with logo",
            ]
        );
        assert_eq!(v[0].0, Wahl::BLAU);
        assert_eq!(v[0].1, ordner[0].join("querencia-blau.png"));
        assert_eq!(verfuegbar(&ordner, |_| false), []);
    }

    #[test]
    fn verfuegbar_mit_datenpaket_0_2_und_teilweise() {
        let ordner = vec![
            PathBuf::from("/home/q/.local/share/querencia/hintergruende"),
            PathBuf::from("/usr/share/querencia/hintergruende"),
        ];
        let alle = verfuegbar(&ordner, |_| true);
        assert_eq!(alle.len(), 25);
        assert_eq!(alle[0].0, Wahl::STANDARD);
        assert_eq!(alle[0].1, ordner[0].join("querencia-tiefblau.png"));
        // Nur die 4:3-Fassung von Tiefblau installiert: keine Kachel (die
        // Galerie zeigt die 16:9-Datei), Glas nur im Systemordner: Kachel.
        let teil = |p: &Path| {
            nur_0_1(p)
                || p == ordner[0].join("querencia-tiefblau-4x3.png")
                || p == ordner[1].join("querencia-glas.png")
        };
        let v = verfuegbar(&ordner, teil);
        assert_eq!(v.len(), 10);
        assert_eq!(v[0].0.name(), "Querencia Glass");
        assert_eq!(v[0].1, ordner[1].join("querencia-glas.png"));
        assert!(v.iter().all(|(w, _)| *w != Wahl::STANDARD));
    }

    #[test]
    fn finden_faellt_auf_16_9_zurueck_und_nimmt_den_ersten_ordner() {
        let ordner = vec![
            PathBuf::from("/home/q/.local/share/querencia/hintergruende"),
            PathBuf::from("/usr/share/querencia/hintergruende"),
        ];
        let abend = Wahl {
            motiv: index("abend"),
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
                motiv: index("erde"),
                logo: true
            })
        );
        assert_eq!(
            erkennen(&o.join("querencia-blau2.png")),
            Some(Wahl {
                motiv: index("blau2"),
                logo: false
            })
        );
        assert_eq!(erkennen(&o.join("querencia-blau.png")), Some(Wahl::BLAU));
        assert_eq!(
            erkennen(&o.join("querencia-tiefblau-4x3.png")),
            Some(Wahl::STANDARD)
        );
        assert_eq!(
            erkennen(&o.join("querencia-tiefblau-wortmarke.png")),
            Some(Wahl {
                motiv: index("tiefblau-wortmarke"),
                logo: false
            })
        );
        // „-logo“ der 0.2-Motive ist Teil des Stamms, keine zweite Kachel.
        assert_eq!(
            erkennen(&o.join("querencia-bernstein-logo-4x3.png")),
            Some(Wahl {
                motiv: index("bernstein-logo"),
                logo: false
            })
        );
        assert_eq!(erkennen(&o.join("querencia-tiefblau-1920.png")), None);
        assert_eq!(erkennen(&o.join("urlaub.png")), None);
        assert_eq!(
            erkennen(Path::new("/home/q/Bilder/querencia-erde.png")),
            None
        );
        assert_eq!(erkennen(Path::new("querencia-erde.png")), None);
    }
}
