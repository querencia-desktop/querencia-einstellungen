//! Datenmodell des Hintergrund-Dialogs (QE-01), ohne GTK.
//!
//! Die Datei ist dieselbe wie beim Desktop (`~/.config/querencia/desktop.ini`,
//! Gruppe `[Background]`, Schlüssel `File`, `Mode`, `Color`; siehe
//! `querencia-desktop/src/config.rs`). Zwei Erweiterungen, die der Desktop
//! HMP-10a noch nicht kennt und die W1 nachrüsten muss (siehe Bericht):
//!
//! ```ini
//! [Background]
//! Type=image          # image (Standard, wie bisher) | color (nur Farbe, kein Bild)
//! File=~/Bilder/x.jpg
//! Mode=zoom           # zoom | scaled | stretched | centered (neu) | tiled (neu)
//! Color=#204a87
//! ```
//!
//! Ein alter Desktop ignoriert `Type` („unbekannter Schlüssel“) und fällt bei
//! `centered`/`tiled` auf `zoom` zurück – beides nur mit Logzeile, nie mit
//! Absturz. Welche Werte der laufende Desktop tatsächlich darstellen kann,
//! sagt [`Faehigkeiten`].
use crate::i18n::{gettext, gettext_f, gettext_noop};
use std::path::PathBuf;

/// Das eingebaute Bild bei leerem `File=` (Name wie im Desktop): seit 0.1.4
/// „Querencia Tiefblau“ wie `BUILT_IN` ab Desktop 0.2.1.
pub const EINGEBAUT: &str = "querencia-tiefblau.png";
/// Das eingebaute Bild bis Desktop 0.2.0 („Querencia Blau“); gilt, solange
/// Tiefblau nicht installiert ist.
pub const EINGEBAUT_0_1: &str = "querencia-blau.png";
/// Standardfarbe: das Blau am unteren Rand von „Querencia Blau“ (wie im Desktop).
pub const STANDARD_FARBE: Farbe = Farbe {
    r: 0x1c,
    g: 0x4e,
    b: 0x96,
};

/// Vordefinierte Palette (Name, Farbe) für die klassische Farbauswahl.
pub const PALETTE: &[(&str, Farbe)] = &[
    (gettext_noop("Querencia Blue"), STANDARD_FARBE),
    (gettext_noop("Tango Blue"), Farbe::neu(0x20, 0x4a, 0x87)),
    (gettext_noop("Anthracite"), Farbe::neu(0x24, 0x24, 0x24)),
    (
        gettext_noop("Querencia Orange"),
        Farbe::neu(0xf0, 0x77, 0x46),
    ),
    (gettext_noop("Slate"), Farbe::neu(0x2e, 0x34, 0x36)),
    (gettext_noop("Moss"), Farbe::neu(0x4e, 0x9a, 0x06)),
    (gettext_noop("Wine"), Farbe::neu(0x8f, 0x1b, 0x1b)),
    (gettext_noop("Sand"), Farbe::neu(0xc4, 0xa0, 0x00)),
    (gettext_noop("Fog"), Farbe::neu(0xba, 0xbd, 0xb6)),
    (gettext_noop("Black"), Farbe::neu(0x00, 0x00, 0x00)),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Farbe {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Farbe {
    pub const fn neu(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// `#rrggbb` oder `rrggbb` (Groß-/Kleinschreibung egal), sonst `None`.
    pub fn aus_hex(text: &str) -> Option<Self> {
        let hex = text.trim();
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let teil = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        Some(Self::neu(teil(0)?, teil(2)?, teil(4)?))
    }

    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Anteile 0.0–1.0 für GTK.
    pub fn anteile(self) -> (f32, f32, f32) {
        (
            f32::from(self.r) / 255.0,
            f32::from(self.g) / 255.0,
            f32::from(self.b) / 255.0,
        )
    }
}

/// Darstellung eines Bildes. Die Namen der Datei sind die des Desktops
/// (`zoom`, `scaled`, `stretched`) plus die beiden neuen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Darstellung {
    /// Bildschirm ausfüllen: Seitenverhältnis halten, Überstand beschneiden.
    #[default]
    Ausfuellen,
    /// Einpassen: ganzes Bild, Seitenverhältnis halten, Ränder in der Farbe.
    Einpassen,
    /// Zentrieren: Originalgröße in der Mitte, Rest in der Farbe.
    Zentrieren,
    /// Strecken: füllen ohne Rücksicht auf das Seitenverhältnis.
    Strecken,
    /// Kacheln: Originalgröße, ab links oben wiederholt.
    Kacheln,
}

impl Darstellung {
    pub const ALLE: [Darstellung; 5] = [
        Self::Ausfuellen,
        Self::Einpassen,
        Self::Zentrieren,
        Self::Strecken,
        Self::Kacheln,
    ];

    pub fn beschriftung(self) -> String {
        match self {
            Self::Ausfuellen => gettext("Zoom"),
            Self::Einpassen => gettext("Scaled"),
            Self::Zentrieren => gettext("Centered"),
            Self::Strecken => gettext("Stretched"),
            Self::Kacheln => gettext("Tiled"),
        }
    }

    /// Wert in `desktop.ini`.
    pub fn dateiwert(self) -> &'static str {
        match self {
            Self::Ausfuellen => "zoom",
            Self::Einpassen => "scaled",
            Self::Zentrieren => "centered",
            Self::Strecken => "stretched",
            Self::Kacheln => "tiled",
        }
    }

    pub fn aus_dateiwert(wert: &str) -> Option<Self> {
        match wert.trim().to_ascii_lowercase().as_str() {
            "zoom" => Some(Self::Ausfuellen),
            "scaled" => Some(Self::Einpassen),
            "centered" => Some(Self::Zentrieren),
            "stretched" => Some(Self::Strecken),
            "tiled" => Some(Self::Kacheln),
            _ => None,
        }
    }

    /// Kann ein Desktop mit diesen Fähigkeiten den Modus zeichnen?
    pub fn moeglich(self, f: &Faehigkeiten) -> bool {
        match self {
            Self::Ausfuellen | Self::Einpassen | Self::Strecken => true,
            Self::Zentrieren => f.zentrieren,
            Self::Kacheln => f.kacheln,
        }
    }
}

/// Was der laufende Desktop über die drei Grundmodi hinaus darstellen kann.
/// Quelle: `querencia-desktop --faehigkeiten` (z. B. „type-color mode-centered
/// mode-tiled“); ein Desktop ohne diese Option (HMP-10a) kann nichts davon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Faehigkeiten {
    pub farbe: bool,
    pub zentrieren: bool,
    pub kacheln: bool,
    /// G2-X: wertet `[Volumes]` aus (Wörter `volumes-types`, `icons-visible`,
    /// `icon-size`; Spezifikation `docs/desktop-ini-arbeitsflaeche.md` §6).
    pub laufwerkstypen: bool,
    pub symbole_sichtbar: bool,
    pub symbolgroesse: bool,
}

impl Faehigkeiten {
    /// Desktop HMP-10a (und jeder Desktop ohne `--faehigkeiten`).
    pub const HMP10A: Self = Self {
        farbe: false,
        zentrieren: false,
        kacheln: false,
        laufwerkstypen: false,
        symbole_sichtbar: false,
        symbolgroesse: false,
    };

    /// Ausgabe von `--faehigkeiten` (Wörter, Reihenfolge egal, Unbekanntes ignoriert).
    pub fn aus_ausgabe(text: &str) -> Self {
        let mut f = Self::HMP10A;
        for wort in text.split_whitespace() {
            match wort {
                "type-color" => f.farbe = true,
                "mode-centered" => f.zentrieren = true,
                "mode-tiled" => f.kacheln = true,
                "volumes-types" => f.laufwerkstypen = true,
                "icons-visible" => f.symbole_sichtbar = true,
                "icon-size" => f.symbolgroesse = true,
                _ => {}
            }
        }
        f
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Typ {
    #[default]
    Bild,
    Farbe,
}

/// Die vollständige Einstellung, wie sie Vorschau und Datei verwenden.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hintergrund {
    pub typ: Typ,
    /// `None`: das eingebaute Bild.
    pub datei: Option<PathBuf>,
    pub darstellung: Darstellung,
    pub farbe: Farbe,
}

impl Default for Hintergrund {
    /// Der ursprüngliche Querencia-Hintergrund: eingebautes Bild, ausfüllen,
    /// Standardfarbe – genau das, was der Desktop ohne `desktop.ini` zeigt.
    fn default() -> Self {
        Self {
            typ: Typ::Bild,
            datei: None,
            darstellung: Darstellung::Ausfuellen,
            farbe: STANDARD_FARBE,
        }
    }
}

impl Hintergrund {
    /// Liest die Gruppe `[Background]`; unbekannte oder ungültige Werte fallen
    /// wie beim Desktop auf den Standard zurück. `probleme` nennt sie.
    pub fn lesen(text: &str) -> (Self, Vec<String>) {
        let mut h = Self::default();
        let mut probleme = Vec::new();
        let mut in_gruppe = false;
        for zeile in text.lines() {
            let zeile = zeile.trim();
            if zeile.is_empty() || zeile.starts_with('#') {
                continue;
            }
            if zeile.starts_with('[') {
                in_gruppe = zeile == "[Background]";
                continue;
            }
            if !in_gruppe {
                continue;
            }
            let Some((schluessel, wert)) = zeile.split_once('=') else {
                probleme.push(format!("Zeile ohne „=“: {zeile}"));
                continue;
            };
            let wert = wert.trim();
            match schluessel.trim() {
                "Type" => match wert.to_ascii_lowercase().as_str() {
                    "image" => h.typ = Typ::Bild,
                    "color" => h.typ = Typ::Farbe,
                    _ => probleme.push(format!("unbekannter Typ „{wert}“")),
                },
                "File" if wert.is_empty() => h.datei = None,
                "File" => h.datei = Some(heim_erweitern(wert)),
                "Mode" => match Darstellung::aus_dateiwert(wert) {
                    Some(d) => h.darstellung = d,
                    None => probleme.push(format!("unbekannter Modus „{wert}“")),
                },
                "Color" => match Farbe::aus_hex(wert) {
                    Some(f) => h.farbe = f,
                    None => probleme.push(format!("ungültige Farbe „{wert}“")),
                },
                anderes => probleme.push(format!("unbekannter Schlüssel „{anderes}“")),
            }
        }
        (h, probleme)
    }

    /// Text der Datei. `Type` steht nur bei Farbe, damit eine Bild-Einstellung
    /// für den Desktop HMP-10a byte-weise wie bisher aussieht.
    pub fn schreiben(&self) -> String {
        let mut text = String::from("[Background]\n");
        if self.typ == Typ::Farbe {
            text.push_str("Type=color\n");
        }
        text.push_str(&format!(
            "File={}\nMode={}\nColor={}\n",
            self.datei
                .as_ref()
                .map(|d| d.display().to_string())
                .unwrap_or_default(),
            self.darstellung.dateiwert(),
            self.farbe.hex()
        ));
        text
    }

    /// Kann ein Desktop mit den Fähigkeiten `f` diese Einstellung so darstellen?
    /// Sonst der Grund (für Statuszeile und Tooltip).
    pub fn tauglich(&self, f: &Faehigkeiten) -> Result<(), String> {
        if self.typ == Typ::Farbe {
            if f.farbe {
                return Ok(());
            }
            return Err(gettext(
                "A plain solid-color background needs the desktop extension \
                 (Type=color); the running desktop shows the built-in image instead.",
            ));
        }
        if !self.darstellung.moeglich(f) {
            return Err(gettext_f(
                "“{style}” needs the desktop extension; the running desktop shows \
                 “{zoom}” instead.",
                &[
                    ("style", &self.darstellung.beschriftung()),
                    ("zoom", &Darstellung::Ausfuellen.beschriftung()),
                ],
            ));
        }
        Ok(())
    }
}

fn heim_erweitern(wert: &str) -> PathBuf {
    match wert.strip_prefix("~/") {
        Some(rest) => std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join(rest),
        None => PathBuf::from(wert),
    }
}

/// Zeichenrechteck (x, y, Breite, Höhe) eines Bildes der Größe `bild` auf einer
/// Fläche `flaeche` – für Ausfüllen, Einpassen und Strecken dieselbe Rechnung wie
/// `config::placement` im Desktop; Zentrieren legt das Bild unskaliert in die Mitte.
/// Kacheln hat kein einzelnes Rechteck (siehe [`kachel_raster`]).
pub fn platzierung(d: Darstellung, bild: (f64, f64), flaeche: (f64, f64)) -> (f64, f64, f64, f64) {
    let (bw, bh) = bild;
    let (fw, fh) = flaeche;
    if bw <= 0.0 || bh <= 0.0 || d == Darstellung::Strecken {
        return (0.0, 0.0, fw, fh);
    }
    let faktor = match d {
        Darstellung::Ausfuellen => (fw / bw).max(fh / bh),
        Darstellung::Einpassen => (fw / bw).min(fh / bh),
        _ => 1.0,
    };
    let (w, h) = (bw * faktor, bh * faktor);
    ((fw - w) / 2.0, (fh - h) / 2.0, w, h)
}

/// Anzahl Kacheln (Spalten, Zeilen) ab links oben, mindestens 1×1. Spezifikation
/// für W1 (der Desktop braucht sie zum Kacheln ohne `push_repeat`); hier nur getestet.
#[cfg(test)]
pub fn kachel_raster(bild: (f64, f64), flaeche: (f64, f64)) -> (u32, u32) {
    let (bw, bh) = bild;
    let (fw, fh) = flaeche;
    if bw <= 0.0 || bh <= 0.0 {
        return (1, 1);
    }
    (
        (fw / bw).ceil().max(1.0) as u32,
        (fh / bh).ceil().max(1.0) as u32,
    )
}

/// Die Vorschau zeigt die Fläche verkleinert: Maßstab, der `flaeche` in
/// `vorschau` einpasst, ohne zu vergrößern.
pub fn vorschau_massstab(flaeche: (f64, f64), vorschau: (f64, f64)) -> f64 {
    (vorschau.0 / flaeche.0)
        .min(vorschau.1 / flaeche.1)
        .min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_wird_gelesen_und_geschrieben() {
        for (text, soll) in [
            ("#204A87", Farbe::neu(0x20, 0x4a, 0x87)),
            ("242424", Farbe::neu(0x24, 0x24, 0x24)),
            (" #f07746 ", Farbe::neu(0xf0, 0x77, 0x46)),
        ] {
            assert_eq!(Farbe::aus_hex(text), Some(soll), "{text}");
        }
        assert_eq!(Farbe::neu(0x20, 0x4a, 0x87).hex(), "#204a87");
    }

    #[test]
    fn ungueltige_hex_eingaben_werden_abgelehnt() {
        for text in [
            "", "#", "#12345", "#1234567", "#12345g", "blau", "#GGGGGG", "# 204a87",
        ] {
            assert_eq!(Farbe::aus_hex(text), None, "{text:?}");
        }
    }

    #[test]
    fn standard_ist_der_desktop_standard() {
        let h = Hintergrund::default();
        assert_eq!(h.typ, Typ::Bild);
        assert_eq!(h.datei, None);
        assert_eq!(h.darstellung, Darstellung::Ausfuellen);
        assert_eq!(h.farbe, Farbe::neu(0x1c, 0x4e, 0x96));
        // Genau der Text, den der Desktop HMP-09/10a selbst schreibt.
        assert_eq!(
            h.schreiben(),
            "[Background]\nFile=\nMode=zoom\nColor=#1c4e96\n"
        );
    }

    #[test]
    fn bild_einstellung_bleibt_fuer_den_alten_desktop_lesbar() {
        let h = Hintergrund {
            typ: Typ::Bild,
            datei: Some(PathBuf::from("/tmp/berge.jpg")),
            darstellung: Darstellung::Einpassen,
            farbe: Farbe::neu(1, 2, 3),
        };
        let text = h.schreiben();
        assert_eq!(
            text,
            "[Background]\nFile=/tmp/berge.jpg\nMode=scaled\nColor=#010203\n"
        );
        assert!(!text.contains("Type="));
        let (zurueck, probleme) = Hintergrund::lesen(&text);
        assert_eq!(zurueck, h);
        assert!(probleme.is_empty());
        assert_eq!(h.tauglich(&Faehigkeiten::HMP10A), Ok(()));
    }

    #[test]
    fn farbe_und_neue_modi_brauchen_die_erweiterung() {
        let farbe = Hintergrund {
            typ: Typ::Farbe,
            ..Hintergrund::default()
        };
        assert!(farbe.schreiben().starts_with("[Background]\nType=color\n"));
        assert!(farbe.tauglich(&Faehigkeiten::HMP10A).is_err());
        let (zurueck, probleme) = Hintergrund::lesen(&farbe.schreiben());
        assert_eq!(zurueck, farbe);
        assert!(probleme.is_empty());
        for d in [Darstellung::Zentrieren, Darstellung::Kacheln] {
            let h = Hintergrund {
                darstellung: d,
                ..Hintergrund::default()
            };
            assert!(h.tauglich(&Faehigkeiten::HMP10A).is_err(), "{d:?}");
        }
    }

    #[test]
    fn lesen_ist_tolerant_wie_der_desktop() {
        let text =
            "[Andere]\nFile=x\n[Background]\nFile=~/b.png\nMode=unsinn\nColor=rot\nFoo=1\nkaputt\n";
        let (h, probleme) = Hintergrund::lesen(text);
        assert!(h.datei.unwrap().ends_with("b.png"));
        assert_eq!(h.darstellung, Darstellung::Ausfuellen);
        assert_eq!(h.farbe, STANDARD_FARBE);
        assert_eq!(probleme.len(), 4, "{probleme:?}");
    }

    #[test]
    fn platzierung_haelt_proportionen() {
        // 4:3-Bild auf 16:9-Fläche
        let bild = (1600.0, 1200.0);
        let flaeche = (1920.0, 1080.0);
        let (x, y, w, h) = platzierung(Darstellung::Ausfuellen, bild, flaeche);
        assert!((w - 1920.0).abs() < 1e-9 && (h - 1440.0).abs() < 1e-9);
        assert!((x).abs() < 1e-9 && (y + 180.0).abs() < 1e-9);
        let (x, y, w, h) = platzierung(Darstellung::Einpassen, bild, flaeche);
        assert!((h - 1080.0).abs() < 1e-9 && (w - 1440.0).abs() < 1e-9);
        assert!((x - 240.0).abs() < 1e-9 && y.abs() < 1e-9);
        let (x, y, w, h) = platzierung(Darstellung::Zentrieren, bild, flaeche);
        assert_eq!((x, y, w, h), (160.0, -60.0, 1600.0, 1200.0));
        assert_eq!(
            platzierung(Darstellung::Strecken, bild, flaeche),
            (0.0, 0.0, 1920.0, 1080.0)
        );
        assert_eq!(
            platzierung(Darstellung::Ausfuellen, (0.0, 0.0), flaeche),
            (0.0, 0.0, 1920.0, 1080.0)
        );
    }

    #[test]
    fn kacheln_und_massstab() {
        assert_eq!(kachel_raster((500.0, 400.0), (1920.0, 1080.0)), (4, 3));
        assert_eq!(kachel_raster((1920.0, 1080.0), (1920.0, 1080.0)), (1, 1));
        assert_eq!(kachel_raster((0.0, 0.0), (1920.0, 1080.0)), (1, 1));
        assert!((vorschau_massstab((1920.0, 1080.0), (480.0, 300.0)) - 0.25).abs() < 1e-9);
        assert_eq!(vorschau_massstab((400.0, 300.0), (800.0, 600.0)), 1.0);
    }

    #[test]
    fn faehigkeiten_geben_genau_das_frei_was_der_desktop_kann() {
        let alle = Faehigkeiten::aus_ausgabe("type-color mode-centered mode-tiled\n");
        assert_eq!(
            alle,
            Faehigkeiten {
                farbe: true,
                zentrieren: true,
                kacheln: true,
                ..Faehigkeiten::HMP10A
            }
        );
        // G2-X: QE-03 meldet nur die drei QE-01-Wörter; ein QE-04-Desktop mehr
        assert!(!alle.laufwerkstypen && !alle.symbole_sichtbar && !alle.symbolgroesse);
        let qe04 = Faehigkeiten::aus_ausgabe(
            "type-color mode-centered mode-tiled volumes-types icons-visible icon-size",
        );
        assert!(qe04.laufwerkstypen && qe04.symbole_sichtbar && qe04.symbolgroesse && qe04.farbe);
        assert_eq!(Faehigkeiten::aus_ausgabe(""), Faehigkeiten::HMP10A);
        assert_eq!(
            Faehigkeiten::aus_ausgabe("unsinn mode-tiled"),
            Faehigkeiten {
                kacheln: true,
                ..Faehigkeiten::HMP10A
            }
        );
        let farbe = Hintergrund {
            typ: Typ::Farbe,
            ..Hintergrund::default()
        };
        assert_eq!(farbe.tauglich(&alle), Ok(()));
        let nur_farbe = Faehigkeiten {
            farbe: true,
            ..Faehigkeiten::HMP10A
        };
        for d in [Darstellung::Zentrieren, Darstellung::Kacheln] {
            let h = Hintergrund {
                darstellung: d,
                ..Hintergrund::default()
            };
            assert_eq!(h.tauglich(&alle), Ok(()), "{d:?}");
            assert!(h.tauglich(&nur_farbe).is_err(), "{d:?}");
        }
        // Farbmodus ist unabhängig von der Darstellung des (ausgeblendeten) Bildes
        let farbe_kacheln = Hintergrund {
            typ: Typ::Farbe,
            darstellung: Darstellung::Kacheln,
            ..Hintergrund::default()
        };
        assert_eq!(farbe_kacheln.tauglich(&nur_farbe), Ok(()));
    }

    #[test]
    fn palette_enthaelt_die_beispiele_des_auftrags() {
        for hex in ["#204a87", "#242424", "#f07746"] {
            assert!(PALETTE.iter().any(|(_, f)| f.hex() == hex), "{hex}");
        }
    }
}
