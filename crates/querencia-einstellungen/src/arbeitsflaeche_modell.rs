//! Datenmodell des Bereichs „Symbole der Arbeitsfläche“ (QD-042, G2-X), ohne GTK.
//!
//! Gruppen `[Icons]` und `[Volumes]` in `~/.config/querencia/desktop.ini`;
//! das Format steht in `docs/desktop-ini-arbeitsflaeche.md`:
//!
//! ```ini
//! [Icons]
//! Computer=true      # Rechner
//! Home=true          # Persönlicher Ordner
//! Trash=true         # Papierkorb
//! Size=48            # 32 | 48 | 72 | 96
//!
//! [Volumes]
//! Removable=true     # Wechseldatenträger
//! Optical=true       # optische Medien
//! Network=true       # Netzlaufwerke
//! Internal=false     # eingebaute Laufwerke
//! ```
//!
//! Gelesen wird tolerant wie `[Background]`: Fehlendes und Ungültiges ergibt
//! den Standard, `probleme` nennt es.
use crate::i18n::gettext;
use crate::ini;

/// Symbolgröße in Pixeln (Nautilus-Stufen 66/100/150/200 %).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Symbolgroesse {
    Klein,
    #[default]
    Normal,
    Gross,
    SehrGross,
}

impl Symbolgroesse {
    pub const ALLE: [Symbolgroesse; 4] = [Self::Klein, Self::Normal, Self::Gross, Self::SehrGross];

    pub fn pixel(self) -> u32 {
        match self {
            Self::Klein => 32,
            Self::Normal => 48,
            Self::Gross => 72,
            Self::SehrGross => 96,
        }
    }

    pub fn aus_pixel(wert: &str) -> Option<Self> {
        let px: u32 = wert.trim().parse().ok()?;
        Self::ALLE.into_iter().find(|g| g.pixel() == px)
    }

    pub fn beschriftung(self) -> String {
        match self {
            Self::Klein => gettext("Small (32 pixels)"),
            Self::Normal => gettext("Normal (48 pixels)"),
            Self::Gross => gettext("Large (72 pixels)"),
            Self::SehrGross => gettext("Very large (96 pixels)"),
        }
    }
}

/// Die vollständige Einstellung des Bereichs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arbeitsflaeche {
    pub rechner: bool,
    pub persoenlich: bool,
    pub papierkorb: bool,
    pub groesse: Symbolgroesse,
    pub wechsel: bool,
    pub optisch: bool,
    pub netz: bool,
    pub eingebaut: bool,
}

impl Default for Arbeitsflaeche {
    /// Standard ohne `desktop.ini` – **an genau dieser Stelle** (und im Desktop,
    /// siehe Spezifikation §3). Wie GNOME 2: Wechsel-, optische und
    /// Netzlaufwerke an, eingebaute aus; feste Symbole an, 48 Pixel.
    /// Entschieden am 19.10.2026 (Status-Session, vom Besitzer übertragen):
    /// genau diese Werte.
    fn default() -> Self {
        Self {
            rechner: true,
            persoenlich: true,
            papierkorb: true,
            groesse: Symbolgroesse::Normal,
            wechsel: true,
            optisch: true,
            netz: true,
            eingebaut: false,
        }
    }
}

pub const GRUPPE_SYMBOLE: &str = "Icons";
pub const GRUPPE_LAUFWERKE: &str = "Volumes";

/// `true`/`false`, `1`/`0`, `yes`/`no` (Groß-/Kleinschreibung egal).
pub fn wahrheitswert(wert: &str) -> Option<bool> {
    match wert.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Some(true),
        "false" | "0" | "no" => Some(false),
        _ => None,
    }
}

fn wort(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

impl Arbeitsflaeche {
    /// Liest `[Icons]` und `[Volumes]` aus dem ganzen Dateitext.
    pub fn lesen(text: &str) -> (Self, Vec<String>) {
        let mut a = Self::default();
        let mut probleme = Vec::new();
        let mut gruppe: Option<String> = None;
        for zeile in text.lines() {
            let zeile = zeile.trim();
            if zeile.is_empty() || zeile.starts_with('#') {
                continue;
            }
            if let Some(name) = zeile.strip_prefix('[').and_then(|z| z.strip_suffix(']')) {
                gruppe = Some(name.to_owned());
                continue;
            }
            let g = match gruppe.as_deref() {
                Some(g @ (GRUPPE_SYMBOLE | GRUPPE_LAUFWERKE)) => g,
                _ => continue,
            };
            let Some((schluessel, wert)) = zeile.split_once('=') else {
                probleme.push(format!("[{g}] Zeile ohne „=“: {zeile}"));
                continue;
            };
            let (schluessel, wert) = (schluessel.trim(), wert.trim());
            if g == GRUPPE_SYMBOLE && schluessel == "Size" {
                match Symbolgroesse::aus_pixel(wert) {
                    Some(s) => a.groesse = s,
                    None => probleme.push(format!(
                        "[Icons] ungültige Größe „{wert}“ (32, 48, 72 oder 96), 48"
                    )),
                }
                continue;
            }
            let ziel = match (g, schluessel) {
                (GRUPPE_SYMBOLE, "Computer") => &mut a.rechner,
                (GRUPPE_SYMBOLE, "Home") => &mut a.persoenlich,
                (GRUPPE_SYMBOLE, "Trash") => &mut a.papierkorb,
                (GRUPPE_LAUFWERKE, "Removable") => &mut a.wechsel,
                (GRUPPE_LAUFWERKE, "Optical") => &mut a.optisch,
                (GRUPPE_LAUFWERKE, "Network") => &mut a.netz,
                (GRUPPE_LAUFWERKE, "Internal") => &mut a.eingebaut,
                _ => {
                    probleme.push(format!("[{g}] unbekannter Schlüssel „{schluessel}“"));
                    continue;
                }
            };
            match wahrheitswert(wert) {
                Some(b) => *ziel = b,
                None => probleme.push(format!(
                    "[{g}] ungültiger Wert „{wert}“ für {schluessel} (true/false)"
                )),
            }
        }
        (a, probleme)
    }

    pub fn block_symbole(&self) -> String {
        format!(
            "[Icons]\nComputer={}\nHome={}\nTrash={}\nSize={}\n",
            wort(self.rechner),
            wort(self.persoenlich),
            wort(self.papierkorb),
            self.groesse.pixel()
        )
    }

    pub fn block_laufwerke(&self) -> String {
        format!(
            "[Volumes]\nRemovable={}\nOptical={}\nNetwork={}\nInternal={}\n",
            wort(self.wechsel),
            wort(self.optisch),
            wort(self.netz),
            wort(self.eingebaut)
        )
    }

    /// Der neue Dateitext: beide Gruppen ersetzt, alles andere byte-gleich.
    pub fn in_datei(&self, bisher: &str) -> String {
        let t = ini::ersetzen(bisher, GRUPPE_SYMBOLE, &self.block_symbole());
        ini::ersetzen(&t, GRUPPE_LAUFWERKE, &self.block_laufwerke())
    }
}

/// Der Teil der Datei, über den der Bereich wacht (Vergleich beim Übernehmen).
pub fn eigene_gruppen(text: &str) -> String {
    format!(
        "{}{}",
        ini::gruppe(text, GRUPPE_SYMBOLE),
        ini::gruppe(text, GRUPPE_LAUFWERKE)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modell::Hintergrund;

    const HG: &str = "[Background]\nFile=/tmp/berge.jpg\nMode=scaled\nColor=#010203\n";

    #[test]
    fn ohne_datei_gilt_der_standard() {
        let (a, p) = Arbeitsflaeche::lesen("");
        assert_eq!(a, Arbeitsflaeche::default());
        assert!(p.is_empty());
        let (a, p) = Arbeitsflaeche::lesen(HG);
        assert_eq!(a, Arbeitsflaeche::default());
        assert!(p.is_empty(), "{p:?}");
        let d = Arbeitsflaeche::default();
        assert!(d.rechner && d.persoenlich && d.papierkorb);
        assert!(d.wechsel && d.optisch && d.netz && !d.eingebaut);
        assert_eq!(d.groesse.pixel(), 48);
    }

    #[test]
    fn standard_wird_vollstaendig_geschrieben() {
        assert_eq!(
            Arbeitsflaeche::default().in_datei(""),
            "[Icons]\nComputer=true\nHome=true\nTrash=true\nSize=48\n\n\
             [Volumes]\nRemovable=true\nOptical=true\nNetwork=true\nInternal=false\n"
        );
    }

    #[test]
    fn hin_und_zurueck_alle_werte() {
        for g in Symbolgroesse::ALLE {
            for maske in 0u8..128 {
                let bit = |i: u8| maske & (1 << i) != 0;
                let a = Arbeitsflaeche {
                    rechner: bit(0),
                    persoenlich: bit(1),
                    papierkorb: bit(2),
                    groesse: g,
                    wechsel: bit(3),
                    optisch: bit(4),
                    netz: bit(5),
                    eingebaut: bit(6),
                };
                let (zurueck, p) = Arbeitsflaeche::lesen(&a.in_datei(HG));
                assert_eq!(zurueck, a);
                assert!(p.is_empty());
            }
        }
    }

    /// F3/F4 der Spezifikation: der Hintergrund bleibt byte-gleich, und
    /// umgekehrt bleibt die Arbeitsfläche beim Hintergrund-Übernehmen.
    #[test]
    fn hintergrund_bleibt_unberuehrt() {
        let a = Arbeitsflaeche {
            netz: false,
            groesse: Symbolgroesse::Gross,
            ..Arbeitsflaeche::default()
        };
        let text = a.in_datei(HG);
        assert!(text.starts_with(HG));
        assert_eq!(ini::gruppe(&text, "Background"), HG);
        let (h, p) = Hintergrund::lesen(&text);
        assert!(p.is_empty(), "{p:?}");
        assert_eq!(h.schreiben(), HG);
        // erneutes Übernehmen mit anderen Werten: Hintergrund weiter gleich
        let b = Arbeitsflaeche {
            eingebaut: true,
            ..a
        };
        let text2 = b.in_datei(&text);
        assert!(text2.starts_with(HG));
        assert_eq!(Arbeitsflaeche::lesen(&text2).0, b);
        assert_eq!(text2.matches("[Volumes]").count(), 1);
    }

    #[test]
    fn ungueltiges_faellt_auf_den_standard() {
        let text = "[Icons]\nSize=50\nComputer=vielleicht\nHome=0\nFoo=1\nkaputt\n\
                    [Volumes]\nRemovable=NO\nNetwork=Yes\nInternal=1\n";
        let (a, p) = Arbeitsflaeche::lesen(text);
        assert_eq!(a.groesse, Symbolgroesse::Normal);
        assert!(a.rechner); // Standard
        assert!(!a.persoenlich);
        assert!(!a.wechsel && a.netz && a.eingebaut);
        assert_eq!(p.len(), 4, "{p:?}");
    }

    #[test]
    fn spaetere_zeile_gewinnt_und_fremde_gruppen_zaehlen_nicht() {
        let text =
            "[Volumes]\nNetwork=false\n[Background]\nNetwork=true\n[Volumes]\nOptical=false\n";
        let (a, p) = Arbeitsflaeche::lesen(text);
        assert!(!a.netz && !a.optisch);
        assert!(p.is_empty());
        let (a, _) = Arbeitsflaeche::lesen("[Volumes]\nInternal=true\nInternal=false\n");
        assert!(!a.eingebaut);
    }

    #[test]
    fn eigene_gruppen_trennen_fremde_aenderungen() {
        let text = Arbeitsflaeche::default().in_datei(HG);
        let anderer_hintergrund = text.replace("Mode=scaled", "Mode=zoom");
        assert_eq!(eigene_gruppen(&text), eigene_gruppen(&anderer_hintergrund));
        let andere_laufwerke = text.replace("Internal=false", "Internal=true");
        assert_ne!(eigene_gruppen(&text), eigene_gruppen(&andere_laufwerke));
    }

    #[test]
    fn groessen() {
        assert_eq!(Symbolgroesse::aus_pixel(" 72 "), Some(Symbolgroesse::Gross));
        for falsch in ["", "50", "48px", "-48", "groß"] {
            assert_eq!(Symbolgroesse::aus_pixel(falsch), None, "{falsch}");
        }
        assert_eq!(
            Symbolgroesse::ALLE.map(Symbolgroesse::pixel),
            [32, 48, 72, 96]
        );
    }
}
