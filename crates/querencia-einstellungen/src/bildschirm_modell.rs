//! Modell des Bereichs „Bildschirm“ (G2-G): Ausgaben und Modi, wie der
//! Compositor sie meldet; die Wahl des Benutzers je Ausgabe; Anordnung mit
//! Einrasten; die Datei `~/.config/querencia/bildschirm.toml` (seit 0.1.6,
//! vorher `bildschirm.ini`, die weiter gelesen wird). Ohne GTK und ohne
//! Wayland, damit alles hier mit Unit-Tests prüfbar ist.
use crate::i18n::{gettext, pgettext};
use std::path::PathBuf;

/// Ein Modus: Pixelgröße und Wiederholrate in mHz (wie im Protokoll).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modus {
    pub breite: i32,
    pub hoehe: i32,
    pub mhz: i32,
    pub bevorzugt: bool,
}

impl Modus {
    pub const fn neu(breite: i32, hoehe: i32, mhz: i32) -> Self {
        Self {
            breite,
            hoehe,
            mhz,
            bevorzugt: false,
        }
    }

    /// Gleiche Größe und Rate (das Bevorzugt-Merkmal zählt nicht).
    pub fn gleich(&self, anderer: &Modus) -> bool {
        self.breite == anderer.breite && self.hoehe == anderer.hoehe && self.mhz == anderer.mhz
    }

    pub fn hz_text(&self) -> String {
        if self.mhz <= 0 {
            gettext("automatic")
        } else {
            format!("{:.2} Hz", f64::from(self.mhz) / 1000.0).replace('.', &dezimaltrenner())
        }
    }

    /// Wert in `bildschirm.ini` (bis 0.1.5) und im Protokoll: `1920x1080@60000`.
    pub fn dateiwert(&self) -> String {
        format!("{}x{}@{}", self.breite, self.hoehe, self.mhz)
    }

    pub fn aus_dateiwert(wert: &str) -> Option<Self> {
        let (groesse, mhz) = wert.split_once('@')?;
        let (b, h) = groesse.split_once('x')?;
        let (b, h, mhz) = (b.parse().ok()?, h.parse().ok()?, mhz.parse().ok()?);
        if b <= 0 || h <= 0 || mhz < 0 {
            return None;
        }
        Some(Self::neu(b, h, mhz))
    }
}

/// Gebräuchliche Größen für Ausgaben ohne Modusliste (headless, virtuell).
pub const FREIE_GROESSEN: &[(i32, i32)] = &[
    (1024, 768),
    (1280, 720),
    (1280, 1024),
    (1600, 900),
    (1920, 1080),
    (1920, 1200),
    (2560, 1440),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Drehung {
    #[default]
    Normal,
    Rechts90,
    Um180,
    Links270,
}

impl Drehung {
    pub const ALLE: [Drehung; 4] = [
        Drehung::Normal,
        Drehung::Rechts90,
        Drehung::Um180,
        Drehung::Links270,
    ];

    /// `wl_output.transform`: 0 normal, 1 = 90, 2 = 180, 3 = 270 (gespiegelte
    /// Werte 4–7 werden wie die ungespiegelten behandelt).
    pub fn aus_transform(t: u32) -> Self {
        match t % 4 {
            1 => Drehung::Rechts90,
            2 => Drehung::Um180,
            3 => Drehung::Links270,
            _ => Drehung::Normal,
        }
    }

    pub fn gedreht(self) -> bool {
        matches!(self, Drehung::Rechts90 | Drehung::Links270)
    }

    pub fn beschriftung(self) -> String {
        match self {
            Drehung::Normal => gettext("Normal"),
            Drehung::Rechts90 => gettext("90° (right)"),
            Drehung::Um180 => gettext("180° (upside down)"),
            Drehung::Links270 => gettext("270° (left)"),
        }
    }

    pub fn dateiwert(self) -> &'static str {
        match self {
            Drehung::Normal => "0",
            Drehung::Rechts90 => "90",
            Drehung::Um180 => "180",
            Drehung::Links270 => "270",
        }
    }

    pub fn aus_dateiwert(w: &str) -> Option<Self> {
        match w.trim() {
            "0" => Some(Drehung::Normal),
            "90" => Some(Drehung::Rechts90),
            "180" => Some(Drehung::Um180),
            "270" => Some(Drehung::Links270),
            _ => None,
        }
    }
}

/// Angebotene Skalierungen. 1,25 und 1,5 sind gebrochen: GTK 4.10 zeichnet
/// dann mit dem nächsten ganzzahligen Faktor und skaliert herunter.
pub const SKALIERUNGEN: &[f64] = &[1.0, 1.25, 1.5, 2.0];

pub fn skalierung_text(s: f64) -> String {
    if (s - s.round()).abs() < 0.001 {
        format!("{}", s.round() as i64)
    } else {
        format!("{s:.2}")
            .trim_end_matches('0')
            .replace('.', &dezimaltrenner())
    }
}

/// Dezimaltrenner der Oberflächensprache (Bildwiederholrate, Skalierung).
fn dezimaltrenner() -> String {
    // TRANSLATORS: decimal separator of refresh rates and scale factors (59.95 Hz, 1.25 ×)
    pgettext("decimal separator", ".")
}

/// Ist-Zustand einer Ausgabe, wie der Compositor sie meldet.
#[derive(Clone, Debug, PartialEq)]
pub struct Ausgabe {
    pub name: String,
    pub beschreibung: String,
    pub hersteller: String,
    pub modell: String,
    pub seriennummer: String,
    pub physisch_mm: (i32, i32),
    pub modi: Vec<Modus>,
    pub aktiv: bool,
    /// Aktueller Modus; bei headless/virtuellen Ausgaben nicht in `modi`.
    pub modus: Option<Modus>,
    pub position: (i32, i32),
    pub drehung: Drehung,
    pub skalierung: f64,
}

impl Ausgabe {
    /// Gerät für die Zuordnung in der Datei: Hersteller|Modell|Seriennummer
    /// (zählt vor dem Namen).
    pub fn geraet(&self) -> String {
        format!("{}|{}|{}", self.hersteller, self.modell, self.seriennummer)
    }

    pub fn anzeigename(&self) -> String {
        let geraet = [self.hersteller.as_str(), self.modell.as_str()]
            .iter()
            .filter(|s| !s.trim().is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ");
        if geraet.is_empty() {
            if self.beschreibung.trim().is_empty() {
                self.name.clone()
            } else {
                format!("{} – {}", self.name, self.beschreibung.trim())
            }
        } else {
            format!("{} – {}", self.name, geraet)
        }
    }

    pub fn bevorzugter_modus(&self) -> Option<Modus> {
        self.modi
            .iter()
            .find(|m| m.bevorzugt)
            .or_else(|| self.modi.first())
            .copied()
            .or(self.modus)
    }
}

/// Namensanfänge virtueller Ausgaben von wlroots (headless, verschachtelt in
/// Wayland oder X11): Sie haben nie Gerätedaten.
const VIRTUELL: &[&str] = &["HEADLESS-", "WL-", "X11-", "NOOP-"];

/// Ein echter Anschluss, der weder Hersteller noch Modell meldet: Der
/// Monitor hat keine EDID geliefert. labwc bietet dann nur Grundmodi wie
/// 1024 × 768 an. Auf der Entwicklungs-VM mit durchgereichter Grafikkarte
/// passiert das manchmal nach dem Start; Kabel ab- und wieder anstecken hilft.
/// `zwangsweise` nennt Ausgaben, die als solche gelten (Prüfschalter
/// `QE_BILDSCHIRM_OHNE_EDID`, Namen durch Komma getrennt).
pub fn ohne_edid(a: &Ausgabe, zwangsweise: &str) -> bool {
    if zwangsweise.split(',').any(|n| n.trim() == a.name) {
        return true;
    }
    let leer = |t: &str| {
        let t = t.trim();
        t.is_empty() || t.eq_ignore_ascii_case("unknown")
    };
    !VIRTUELL.iter().any(|v| a.name.starts_with(v)) && leer(&a.hersteller) && leer(&a.modell)
}

/// Soll-Zustand einer Ausgabe, wie der Benutzer ihn wählt.
#[derive(Clone, Debug, PartialEq)]
pub struct Wahl {
    pub name: String,
    pub aktiv: bool,
    pub modus: Modus,
    pub position: (i32, i32),
    pub drehung: Drehung,
    pub skalierung: f64,
    /// Hauptmonitor: liegt bei 0,0; in der Datei vermerkt für Panel/Desktop.
    pub haupt: bool,
}

impl Wahl {
    pub fn aus_ausgabe(a: &Ausgabe) -> Self {
        Self {
            name: a.name.clone(),
            aktiv: a.aktiv,
            modus: a
                .modus
                .or_else(|| a.bevorzugter_modus())
                .unwrap_or(Modus::neu(1024, 768, 0)),
            position: a.position,
            drehung: a.drehung,
            skalierung: if a.skalierung > 0.0 {
                a.skalierung
            } else {
                1.0
            },
            haupt: false,
        }
    }

    /// Größe in logischen Pixeln (nach Drehung und Skalierung), 0×0 wenn aus.
    pub fn logische_groesse(&self) -> (i32, i32) {
        if !self.aktiv {
            return (0, 0);
        }
        logische_groesse(self.modus, self.drehung, self.skalierung)
    }

    /// Gleich bis auf Rundung der Skalierung.
    pub fn gleich(&self, b: &Wahl) -> bool {
        self.name == b.name
            && self.aktiv == b.aktiv
            && (!self.aktiv
                || (self.modus.gleich(&b.modus)
                    && self.position == b.position
                    && self.drehung == b.drehung
                    && (self.skalierung - b.skalierung).abs() < 0.001))
    }
}

pub fn logische_groesse(modus: Modus, drehung: Drehung, skalierung: f64) -> (i32, i32) {
    let s = if skalierung > 0.0 { skalierung } else { 1.0 };
    let (b, h) = if drehung.gedreht() {
        (modus.hoehe, modus.breite)
    } else {
        (modus.breite, modus.hoehe)
    };
    (
        (f64::from(b) / s).round() as i32,
        (f64::from(h) / s).round() as i32,
    )
}

/// Wahl aus dem Ist-Zustand; Hauptmonitor ist die aktive Ausgabe bei 0,0,
/// sonst die erste aktive.
pub fn wahlen_aus(ausgaben: &[Ausgabe]) -> Vec<Wahl> {
    let mut w: Vec<Wahl> = ausgaben.iter().map(Wahl::aus_ausgabe).collect();
    let haupt = w
        .iter()
        .position(|x| x.aktiv && x.position == (0, 0))
        .or_else(|| w.iter().position(|x| x.aktiv));
    if let Some(i) = haupt {
        w[i].haupt = true;
    }
    w
}

pub fn unterschiedlich(a: &[Wahl], b: &[Wahl]) -> bool {
    a.len() != b.len() || a.iter().zip(b).any(|(x, y)| !x.gleich(y))
}

/// Eingeschaltete Ausgaben, deren gemeldete Position nicht der Wahl
/// entspricht (nach dem Setzen, siehe `Verbindung::setzen`).
pub fn abweichende_lage(wahlen: &[Wahl], ausgaben: &[Ausgabe]) -> Vec<String> {
    wahlen
        .iter()
        .filter(|w| w.aktiv)
        .filter(|w| {
            ausgaben
                .iter()
                .find(|a| a.name == w.name)
                .is_some_and(|a| a.aktiv && a.position != w.position)
        })
        .map(|w| w.name.clone())
        .collect()
}

// ------------------------------------------------------------ Anordnung

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rechteck {
    x: i32,
    y: i32,
    b: i32,
    h: i32,
}

impl Rechteck {
    fn ueberlappt(&self, o: &Rechteck) -> bool {
        self.x < o.x + o.b && o.x < self.x + self.b && self.y < o.y + o.h && o.y < self.y + self.h
    }
}

fn rechteck(w: &Wahl) -> Rechteck {
    let (b, h) = w.logische_groesse();
    Rechteck {
        x: w.position.0,
        y: w.position.1,
        b,
        h,
    }
}

/// Lässt die Ausgabe `index` nach dem Verschieben einrasten: Kanten innerhalb
/// von `fang` logischen Pixeln an Kanten der anderen aktiven Ausgaben
/// ausrichten, Überlappungen auflösen (an die nächstgelegene freie Seite
/// schieben), zuletzt alles so verschieben, dass die kleinste Ecke bei 0,0
/// liegt. Mit genau einer aktiven Ausgabe landet sie bei 0,0.
pub fn einrasten(wahlen: &mut [Wahl], index: usize, fang: i32) {
    if index >= wahlen.len() || !wahlen[index].aktiv {
        normieren(wahlen);
        return;
    }
    let andere: Vec<Rechteck> = wahlen
        .iter()
        .enumerate()
        .filter(|(i, w)| *i != index && w.aktiv)
        .map(|(_, w)| rechteck(w))
        .collect();
    let mut r = rechteck(&wahlen[index]);
    if andere.is_empty() {
        wahlen[index].position = (0, 0);
        normieren(wahlen);
        return;
    }
    // 1. Kanten fangen
    let mut kand_x: Vec<i32> = Vec::new();
    let mut kand_y: Vec<i32> = Vec::new();
    for o in &andere {
        kand_x.extend([o.x - r.b, o.x + o.b, o.x, o.x + o.b - r.b]);
        kand_y.extend([o.y - r.h, o.y + o.h, o.y, o.y + o.h - r.h]);
    }
    if let Some(x) = kand_x
        .iter()
        .filter(|x| (**x - r.x).abs() <= fang)
        .min_by_key(|x| (**x - r.x).abs())
    {
        r.x = *x;
    }
    if let Some(y) = kand_y
        .iter()
        .filter(|y| (**y - r.y).abs() <= fang)
        .min_by_key(|y| (**y - r.y).abs())
    {
        r.y = *y;
    }
    // 2. Überlappung auflösen: kleinste Verschiebung an eine freie Seite
    for _ in 0..8 {
        let Some(o) = andere.iter().find(|o| r.ueberlappt(o)) else {
            break;
        };
        let moeglichkeiten = [
            (o.x - r.b - r.x, 0),
            (o.x + o.b - r.x, 0),
            (0, o.y - r.h - r.y),
            (0, o.y + o.h - r.y),
        ];
        let (dx, dy) = moeglichkeiten
            .iter()
            .min_by_key(|(dx, dy)| dx.abs() + dy.abs())
            .copied()
            .unwrap_or((0, 0));
        r.x += dx;
        r.y += dy;
    }
    wahlen[index].position = (r.x, r.y);
    normieren(wahlen);
}

/// Verschiebt alle aktiven Ausgaben gemeinsam, sodass die kleinste Ecke bei
/// 0,0 liegt (so erwartet es der Compositor).
pub fn normieren(wahlen: &mut [Wahl]) {
    let aktive: Vec<&Wahl> = wahlen.iter().filter(|w| w.aktiv).collect();
    let min_x = aktive.iter().map(|w| w.position.0).min().unwrap_or(0);
    let min_y = aktive.iter().map(|w| w.position.1).min().unwrap_or(0);
    for w in wahlen.iter_mut().filter(|w| w.aktiv) {
        w.position = (w.position.0 - min_x, w.position.1 - min_y);
    }
}

/// Macht `index` zum Hauptmonitor: Merkmal setzen, die Anordnung so
/// verschieben, dass er bei 0,0 liegt, und die anderen ohne Überlappung
/// daneben halten (relative Lage bleibt, negative Ecken werden verschoben).
pub fn hauptmonitor_setzen(wahlen: &mut [Wahl], index: usize) {
    if index >= wahlen.len() {
        return;
    }
    for (i, w) in wahlen.iter_mut().enumerate() {
        w.haupt = i == index;
    }
    let (hx, hy) = wahlen[index].position;
    for w in wahlen.iter_mut() {
        w.position = (w.position.0 - hx, w.position.1 - hy);
    }
    // Ausgaben links/oberhalb des Hauptmonitors kämen ins Negative: nach
    // rechts bzw. unten an den Hauptmonitor anlegen, damit 0,0 gilt.
    let haupt = rechteck(&wahlen[index]);
    for i in 0..wahlen.len() {
        if i == index || !wahlen[i].aktiv {
            continue;
        }
        let r = rechteck(&wahlen[i]);
        let mut x = r.x;
        let mut y = r.y;
        if x < 0 {
            x = haupt.x + haupt.b;
        }
        if y < 0 {
            y = 0;
        }
        wahlen[i].position = (x, y);
        einrasten(wahlen, i, 0);
    }
    normieren(wahlen);
}

// ----------------------------------------------------------------- Datei

/// Gespeicherte Konfiguration (0.1.6).
pub const DATEINAME: &str = "bildschirm.toml";
/// Bis 0.1.5: wird gelesen, solange es keine `bildschirm.toml` gibt, und nie
/// geschrieben oder gelöscht.
pub const ALTE_DATEI: &str = "bildschirm.ini";

fn konfig_ordner() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        })
        .join("querencia")
}

pub fn konfig_pfad() -> PathBuf {
    konfig_ordner().join(DATEINAME)
}

pub fn alte_konfig_pfad() -> PathBuf {
    konfig_ordner().join(ALTE_DATEI)
}

/// Ein gespeicherter Eintrag: Kennung der Ausgabe und die gewählten Werte.
#[derive(Clone, Debug, PartialEq)]
pub struct Eintrag {
    pub name: String,
    /// Hersteller|Modell|Seriennummer wie [`Ausgabe::geraet`]
    pub geraet: String,
    pub wahl: Wahl,
}

/// TOML-Zeichenkette in doppelten Anführungszeichen.
fn toml_text(t: &str) -> String {
    let mut s = String::with_capacity(t.len() + 2);
    s.push('"');
    for c in t.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\t' => s.push_str("\\t"),
            c if c.is_control() => s.push_str(&format!("\\u{:04X}", c as u32)),
            c => s.push(c),
        }
    }
    s.push('"');
    s
}

/// Skalierung als TOML-Zahl mit Nachkommastelle (`1.0`, `1.25`).
fn toml_zahl(f: f64) -> String {
    let t = format!("{f}");
    if t.contains('.') || t.contains('e') {
        t
    } else {
        format!("{t}.0")
    }
}

/// Text von `bildschirm.toml`: eine Tabelle `[[ausgabe]]` je Ausgabe, mit
/// Name, Hersteller, Modell und Seriennummer zur Zuordnung.
pub fn konfig_schreiben(wahlen: &[Wahl], ausgaben: &[Ausgabe]) -> String {
    let mut text = String::from(
        "# Querencia Einstellungen – Bildschirm. Beim Sitzungsstart setzt\n\
         # „querencia-einstellungen --bildschirm-anwenden“ diese Werte. Zuordnung\n\
         # über hersteller, modell und seriennummer, sonst über den Namen.\n\
         # labwc speichert selbst keine Bildschirmeinstellungen.\n\
         version = 1\n",
    );
    for w in wahlen {
        let a = ausgaben.iter().find(|a| a.name == w.name);
        let feld = |f: fn(&Ausgabe) -> &str| a.map(f).unwrap_or_default().trim().to_owned();
        text.push_str("\n[[ausgabe]]\n");
        text.push_str(&format!("name = {}\n", toml_text(&w.name)));
        text.push_str(&format!(
            "hersteller = {}\n",
            toml_text(&feld(|a| &a.hersteller))
        ));
        text.push_str(&format!("modell = {}\n", toml_text(&feld(|a| &a.modell))));
        text.push_str(&format!(
            "seriennummer = {}\n",
            toml_text(&feld(|a| &a.seriennummer))
        ));
        text.push_str(&format!("aktiv = {}\n", w.aktiv));
        text.push_str(&format!("breite = {}\n", w.modus.breite));
        text.push_str(&format!("hoehe = {}\n", w.modus.hoehe));
        text.push_str(&format!("wiederholrate_mhz = {}\n", w.modus.mhz));
        text.push_str(&format!("x = {}\n", w.position.0));
        text.push_str(&format!("y = {}\n", w.position.1));
        text.push_str(&format!("drehung = {}\n", w.drehung.dateiwert()));
        text.push_str(&format!("skalierung = {}\n", toml_zahl(w.skalierung)));
        text.push_str(&format!("primaer = {}\n", w.haupt));
    }
    text
}

/// Ein TOML-Wert für Meldungen (`"abc"`, `1.5`, `true`, sonst die Art).
fn wert_text(v: &toml::Value) -> String {
    match v {
        toml::Value::String(t) => format!("{t:?}"),
        toml::Value::Integer(n) => n.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        anderes => format!("<{}>", anderes.type_str()),
    }
}

/// Liest `bildschirm.toml`; unbrauchbare Einträge werden mit Grund
/// übergangen, unbekannte Schlüssel gemeldet. Ist die Datei kein TOML,
/// gibt es keinen Eintrag und eine Meldung.
pub fn konfig_lesen(text: &str) -> (Vec<Eintrag>, Vec<String>) {
    let mut eintraege = Vec::new();
    let mut probleme = Vec::new();
    let tabelle = match text.parse::<toml::Table>() {
        Ok(t) => t,
        Err(e) => {
            probleme.push(format!("kein gültiges TOML: {}", e.to_string().trim()));
            return (eintraege, probleme);
        }
    };
    for (k, v) in &tabelle {
        match (k.as_str(), v) {
            ("version", toml::Value::Integer(1)) | ("ausgabe", _) => {}
            ("version", v) => probleme.push(format!(
                "version = {} unbekannt, gelesen wie 1",
                wert_text(v)
            )),
            (k, _) => probleme.push(format!("unbekannter Schlüssel „{k}“")),
        }
    }
    let liste = match tabelle.get("ausgabe") {
        None => return (eintraege, probleme),
        Some(toml::Value::Array(a)) => a.as_slice(),
        Some(_) => {
            probleme.push("„ausgabe“ ist keine Liste [[ausgabe]]".into());
            return (eintraege, probleme);
        }
    };
    for (i, e) in liste.iter().enumerate() {
        let Some(t) = e.as_table() else {
            probleme.push(format!("ausgabe {}: keine Tabelle", i + 1));
            continue;
        };
        let name = t
            .get("name")
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .to_owned();
        if name.is_empty() {
            probleme.push(format!("ausgabe {}: name fehlt, übergangen", i + 1));
            continue;
        }
        let ort = format!("ausgabe {name}");
        let text_von = |k: &str| {
            t.get(k)
                .and_then(toml::Value::as_str)
                .map(str::trim)
                .unwrap_or_default()
                .to_owned()
        };
        let ganz = |k: &str, probleme: &mut Vec<String>| -> Option<i64> {
            match t.get(k) {
                None => None,
                Some(toml::Value::Integer(n)) => Some(*n),
                Some(v) => {
                    probleme.push(format!(
                        "{ort}: {k} = {} ist keine ganze Zahl",
                        wert_text(v)
                    ));
                    None
                }
            }
        };
        let mut wahl = Wahl {
            name: name.clone(),
            aktiv: true,
            modus: Modus::neu(1024, 768, 0),
            position: (0, 0),
            drehung: Drehung::Normal,
            skalierung: 1.0,
            haupt: false,
        };
        match t.get("aktiv") {
            None => {}
            Some(toml::Value::Boolean(b)) => wahl.aktiv = *b,
            Some(v) => probleme.push(format!("{ort}: aktiv = {} unbekannt", wert_text(v))),
        }
        let breite = ganz("breite", &mut probleme);
        let hoehe = ganz("hoehe", &mut probleme);
        let mhz = ganz("wiederholrate_mhz", &mut probleme).unwrap_or(0);
        let modus = match (breite, hoehe) {
            (Some(b), Some(h)) => Modus::aus_dateiwert(&format!("{b}x{h}@{mhz}")),
            _ => None,
        };
        // eine eingeschaltete Ausgabe ohne Größe wird übergangen (unten,
        // nachdem alle Schlüssel gemeldet sind)
        let ohne_groesse = modus.is_none() && wahl.aktiv;
        if let Some(m) = modus {
            wahl.modus = m;
        }
        let x = ganz("x", &mut probleme).unwrap_or(0);
        let y = ganz("y", &mut probleme).unwrap_or(0);
        match (i32::try_from(x), i32::try_from(y)) {
            (Ok(x), Ok(y)) => wahl.position = (x, y),
            _ => probleme.push(format!("{ort}: x/y außerhalb des Bereichs")),
        }
        if let Some(d) = ganz("drehung", &mut probleme) {
            match Drehung::aus_dateiwert(&d.to_string()) {
                Some(d) => wahl.drehung = d,
                None => probleme.push(format!("{ort}: drehung = {d} unbekannt")),
            }
        }
        match t.get("skalierung") {
            None => {}
            Some(v) => match v.as_float().or_else(|| v.as_integer().map(|n| n as f64)) {
                Some(f) if f > 0.0 && f <= 8.0 => wahl.skalierung = f,
                _ => probleme.push(format!("{ort}: skalierung = {} unbrauchbar", wert_text(v))),
            },
        }
        match t.get("primaer") {
            None => {}
            Some(toml::Value::Boolean(b)) => wahl.haupt = *b,
            Some(v) => probleme.push(format!("{ort}: primaer = {} unbekannt", wert_text(v))),
        }
        for k in t.keys() {
            if ![
                "name",
                "hersteller",
                "modell",
                "seriennummer",
                "aktiv",
                "breite",
                "hoehe",
                "wiederholrate_mhz",
                "x",
                "y",
                "drehung",
                "skalierung",
                "primaer",
            ]
            .contains(&k.as_str())
            {
                probleme.push(format!("{ort}: unbekannter Schlüssel „{k}“"));
            }
        }
        if ohne_groesse {
            probleme.push(format!(
                "{ort}: breite/hoehe fehlen oder sind unbrauchbar, übergangen"
            ));
            continue;
        }
        let geraet = format!(
            "{}|{}|{}",
            text_von("hersteller"),
            text_von("modell"),
            text_von("seriennummer")
        );
        eintraege.push(Eintrag {
            name,
            geraet: if geraet == "||" {
                String::new()
            } else {
                geraet
            },
            wahl,
        });
    }
    (eintraege, probleme)
}

/// Liest die alte `bildschirm.ini` (bis 0.1.5); unbrauchbare Gruppen werden
/// mit Grund übergangen.
pub fn ini_lesen(text: &str) -> (Vec<Eintrag>, Vec<String>) {
    let mut eintraege = Vec::new();
    let mut probleme = Vec::new();
    let mut aktueller: Option<(String, String, Wahl, bool)> = None; // name, geraet, wahl, modus gesetzt
    let abschliessen = |a: Option<(String, String, Wahl, bool)>,
                        eintraege: &mut Vec<Eintrag>,
                        probleme: &mut Vec<String>| {
        if let Some((name, geraet, wahl, modus_da)) = a {
            if wahl.aktiv && !modus_da {
                probleme.push(format!("[Ausgabe {name}]: Modus fehlt, Gruppe übergangen"));
            } else {
                eintraege.push(Eintrag { name, geraet, wahl });
            }
        }
    };
    for zeile in text.lines() {
        let zeile = zeile.trim();
        if zeile.is_empty() || zeile.starts_with('#') {
            continue;
        }
        if let Some(rest) = zeile.strip_prefix('[') {
            abschliessen(aktueller.take(), &mut eintraege, &mut probleme);
            let kopf = rest.trim_end_matches(']').trim();
            match kopf.strip_prefix("Ausgabe ") {
                Some(name) if !name.trim().is_empty() => {
                    let name = name.trim().to_owned();
                    aktueller = Some((
                        name.clone(),
                        String::new(),
                        Wahl {
                            name,
                            aktiv: true,
                            modus: Modus::neu(1024, 768, 0),
                            position: (0, 0),
                            drehung: Drehung::Normal,
                            skalierung: 1.0,
                            haupt: false,
                        },
                        false,
                    ));
                }
                _ => probleme.push(format!("unbekannte Gruppe [{kopf}]")),
            }
            continue;
        }
        let Some((schluessel, wert)) = zeile.split_once('=') else {
            probleme.push(format!("Zeile ohne „=“: {zeile}"));
            continue;
        };
        let Some((name, geraet, wahl, modus_da)) = aktueller.as_mut() else {
            probleme.push(format!("Wert außerhalb einer Gruppe: {zeile}"));
            continue;
        };
        let wert = wert.trim();
        match schluessel.trim() {
            "Geraet" => *geraet = wert.to_owned(),
            "Aktiv" => match wert.to_ascii_lowercase().as_str() {
                "ja" | "true" | "1" => wahl.aktiv = true,
                "nein" | "false" | "0" => wahl.aktiv = false,
                _ => probleme.push(format!("[Ausgabe {name}]: Aktiv „{wert}“ unbekannt")),
            },
            "Modus" => match Modus::aus_dateiwert(wert) {
                Some(m) => {
                    wahl.modus = m;
                    *modus_da = true;
                }
                None => probleme.push(format!("[Ausgabe {name}]: Modus „{wert}“ unbrauchbar")),
            },
            "Position" => match wert
                .split_once(',')
                .and_then(|(x, y)| Some((x.trim().parse().ok()?, y.trim().parse().ok()?)))
            {
                Some(p) => wahl.position = p,
                None => probleme.push(format!("[Ausgabe {name}]: Position „{wert}“ unbrauchbar")),
            },
            "Drehung" => match Drehung::aus_dateiwert(wert) {
                Some(d) => wahl.drehung = d,
                None => probleme.push(format!("[Ausgabe {name}]: Drehung „{wert}“ unbekannt")),
            },
            "Skalierung" => match wert.replace(',', ".").parse::<f64>() {
                Ok(s) if s > 0.0 && s <= 8.0 => wahl.skalierung = s,
                _ => probleme.push(format!("[Ausgabe {name}]: Skalierung „{wert}“ unbrauchbar")),
            },
            "Haupt" => {
                wahl.haupt = matches!(wert.to_ascii_lowercase().as_str(), "ja" | "true" | "1")
            }
            anderes => probleme.push(format!(
                "[Ausgabe {name}]: unbekannter Schlüssel „{anderes}“"
            )),
        }
    }
    abschliessen(aktueller.take(), &mut eintraege, &mut probleme);
    (eintraege, probleme)
}

/// Ordnet gespeicherte Einträge den vorhandenen Ausgaben zu: zuerst über das
/// Gerät (Hersteller|Modell|Seriennummer, wenn nicht leer), sonst über den
/// Namen. Ausgaben ohne Eintrag behalten ihren Ist-Zustand. Liefert außerdem,
/// wie viele Ausgaben einen Eintrag bekamen.
pub fn zuordnen(eintraege: &[Eintrag], ausgaben: &[Ausgabe]) -> (Vec<Wahl>, usize) {
    let mut treffer = 0;
    let wahlen = ausgaben
        .iter()
        .map(|a| {
            let geraet = a.geraet();
            let e = eintraege
                .iter()
                .find(|e| geraet != "||" && !e.geraet.is_empty() && e.geraet == geraet)
                .or_else(|| eintraege.iter().find(|e| e.name == a.name));
            match e {
                Some(e) => {
                    treffer += 1;
                    Wahl {
                        name: a.name.clone(),
                        ..e.wahl.clone()
                    }
                }
                None => Wahl::aus_ausgabe(a),
            }
        })
        .collect();
    (wahlen, treffer)
}

/// Schreibt die Datei atomar (`.neu` + rename).
pub fn konfig_speichern(pfad: &std::path::Path, text: &str) -> Result<(), String> {
    let ordner = pfad.parent().ok_or("kein Ordner")?;
    std::fs::create_dir_all(ordner).map_err(|e| format!("{}: {e}", ordner.display()))?;
    let neu = pfad.with_extension("toml.neu");
    std::fs::write(&neu, text).map_err(|e| format!("{}: {e}", neu.display()))?;
    std::fs::rename(&neu, pfad).map_err(|e| format!("{}: {e}", pfad.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ausgabe(name: &str, b: i32, h: i32, x: i32, y: i32) -> Ausgabe {
        Ausgabe {
            name: name.into(),
            beschreibung: String::new(),
            hersteller: String::new(),
            modell: String::new(),
            seriennummer: String::new(),
            physisch_mm: (0, 0),
            modi: vec![],
            aktiv: true,
            modus: Some(Modus::neu(b, h, 60000)),
            position: (x, y),
            drehung: Drehung::Normal,
            skalierung: 1.0,
        }
    }

    #[test]
    fn modus_wird_gelesen_und_geschrieben() {
        let m = Modus::neu(1920, 1080, 60000);
        assert_eq!(m.dateiwert(), "1920x1080@60000");
        assert_eq!(Modus::aus_dateiwert("1920x1080@60000"), Some(m));
        assert_eq!(Modus::aus_dateiwert("1920x1080"), None);
        assert_eq!(Modus::aus_dateiwert("0x1080@60000"), None);
        assert_eq!(m.hz_text(), "60.00 Hz");
        assert_eq!(Modus::neu(1024, 768, 0).hz_text(), "automatic");
    }

    #[test]
    fn logische_groesse_beruecksichtigt_drehung_und_skalierung() {
        let m = Modus::neu(1920, 1080, 60000);
        assert_eq!(logische_groesse(m, Drehung::Normal, 1.0), (1920, 1080));
        assert_eq!(logische_groesse(m, Drehung::Rechts90, 1.0), (1080, 1920));
        assert_eq!(logische_groesse(m, Drehung::Normal, 2.0), (960, 540));
        assert_eq!(logische_groesse(m, Drehung::Normal, 1.5), (1280, 720));
        assert_eq!(skalierung_text(1.0), "1");
        assert_eq!(skalierung_text(1.25), "1.25");
        assert_eq!(skalierung_text(1.5), "1.5");
    }

    #[test]
    fn einrasten_legt_an_die_naechste_kante() {
        let a = [
            ausgabe("A", 1920, 1080, 0, 0),
            ausgabe("B", 1024, 768, 1900, 30),
        ];
        let mut w = wahlen_aus(&a);
        einrasten(&mut w, 1, 60);
        assert_eq!(w[1].position, (1920, 0), "rechts anlegen, oben ausrichten");
        // weit links: Überlappung wird nach links aufgelöst, dann normiert
        w[1].position = (-500, 100);
        einrasten(&mut w, 1, 60);
        assert_eq!(w[1].position, (0, 100));
        assert_eq!(w[0].position, (1024, 0));
        // mitten hinein: an die nächstgelegene Seite
        w[1].position = (1700, 500);
        w[0].position = (0, 0);
        einrasten(&mut w, 1, 60);
        assert_eq!(w[1].position, (1920, 500));
    }

    #[test]
    fn einrasten_mit_einer_ausgabe_landet_bei_null() {
        let a = [ausgabe("A", 1920, 1080, 300, 200)];
        let mut w = wahlen_aus(&a);
        einrasten(&mut w, 0, 60);
        assert_eq!(w[0].position, (0, 0));
    }

    #[test]
    fn hauptmonitor_liegt_bei_null() {
        let a = [
            ausgabe("A", 1920, 1080, 0, 0),
            ausgabe("B", 1024, 768, 1920, 0),
        ];
        let mut w = wahlen_aus(&a);
        assert!(w[0].haupt && !w[1].haupt);
        hauptmonitor_setzen(&mut w, 1);
        assert!(w[1].haupt && !w[0].haupt);
        assert_eq!(w[1].position, (0, 0));
        assert_eq!(w[0].position, (1024, 0), "A wandert rechts neben B");
    }

    #[test]
    fn datei_rundreise() {
        let mut a = ausgabe("HDMI-A-1", 1920, 1080, 0, 0);
        a.hersteller = "Dell".into();
        a.modell = "U2412M \"Pro\"".into();
        a.seriennummer = "123".into();
        let b = ausgabe("DP-1", 1024, 768, 1920, 0);
        let mut w = wahlen_aus(&[a.clone(), b.clone()]);
        w[1].aktiv = false;
        w[0].skalierung = 1.5;
        w[0].drehung = Drehung::Rechts90;
        let text = konfig_schreiben(&w, &[a.clone(), b.clone()]);
        assert!(
            text.contains(
                "\n[[ausgabe]]\nname = \"HDMI-A-1\"\nhersteller = \"Dell\"\nmodell = \"U2412M \\\"Pro\\\"\"\nseriennummer = \"123\"\naktiv = true\nbreite = 1920\nhoehe = 1080\nwiederholrate_mhz = 60000\nx = 0\ny = 0\ndrehung = 90\nskalierung = 1.5\nprimaer = true\n"
            ),
            "{text}"
        );
        assert!(
            text.contains("skalierung = 1.0\nprimaer = false\n"),
            "{text}"
        );
        let (eintraege, probleme) = konfig_lesen(&text);
        assert!(probleme.is_empty(), "{probleme:?}");
        assert_eq!(eintraege.len(), 2);
        assert_eq!(eintraege[0].geraet, "Dell|U2412M \"Pro\"|123");
        assert_eq!(eintraege[1].geraet, "", "ohne Gerätedaten zählt der Name");
        let (zu, treffer) = zuordnen(&eintraege, &[a, b]);
        assert_eq!(treffer, 2);
        assert!(!unterschiedlich(&zu, &w));
        // alle angebotenen Skalierungen kommen genau zurück
        for s in SKALIERUNGEN {
            let mut w2 = w.clone();
            w2[0].skalierung = *s;
            let (e, p) = konfig_lesen(&konfig_schreiben(&w2, &[]));
            assert!(p.is_empty(), "{p:?}");
            assert_eq!(e[0].wahl.skalierung, *s);
        }
    }

    #[test]
    fn zuordnung_ueber_geraet_schlaegt_namen() {
        let mut alt = ausgabe("DP-1", 1920, 1080, 0, 0);
        alt.hersteller = "Dell".into();
        alt.modell = "X".into();
        alt.seriennummer = "1".into();
        let w = wahlen_aus(&[alt.clone()]);
        let (eintraege, _) = konfig_lesen(&konfig_schreiben(&w, std::slice::from_ref(&alt)));
        // dieselbe Ausgabe hängt jetzt an DP-2
        let mut neu = alt.clone();
        neu.name = "DP-2".into();
        neu.position = (500, 500);
        let (zu, treffer) = zuordnen(&eintraege, &[neu]);
        assert_eq!(treffer, 1);
        assert_eq!(zu[0].name, "DP-2");
        assert_eq!(zu[0].position, (0, 0));
        // ohne Gerätedaten zählt der Name
        let anders = ausgabe("DP-1", 1920, 1080, 7, 7);
        let (zu, treffer) = zuordnen(&eintraege, &[anders]);
        assert_eq!(treffer, 1);
        assert_eq!(zu[0].position, (0, 0));
        // unbekannte Ausgabe behält den Ist-Zustand
        let fremd = ausgabe("HDMI-A-2", 1024, 768, 9, 9);
        let (zu, treffer) = zuordnen(&eintraege, &[fremd]);
        assert_eq!(treffer, 0);
        assert_eq!(zu[0].position, (9, 9));
    }

    #[test]
    fn abweichende_lage_nur_eingeschaltete() {
        // labwc legte HEADLESS-2 (automatisch platziert) neben die verschobene
        let soll = wahlen_aus(&[
            ausgabe("HEADLESS-1", 1920, 1080, 0, 720),
            ausgabe("HEADLESS-2", 1280, 720, 0, 0),
        ]);
        let ist = [
            ausgabe("HEADLESS-1", 1920, 1080, 0, 720),
            ausgabe("HEADLESS-2", 1280, 720, 1920, 720),
        ];
        assert_eq!(abweichende_lage(&soll, &ist), ["HEADLESS-2"]);
        let mut aus = soll.clone();
        aus[1].aktiv = false;
        assert!(abweichende_lage(&aus, &ist).is_empty());
        assert!(abweichende_lage(&soll, &ist[..1]).is_empty());
    }

    #[test]
    fn ohne_edid_nur_an_echten_anschluessen() {
        let mut a = ausgabe("HDMI-A-1", 1024, 768, 0, 0);
        assert!(ohne_edid(&a, ""));
        a.hersteller = "Unknown".into();
        assert!(ohne_edid(&a, ""));
        a.modell = "ASUS VS247".into();
        assert!(!ohne_edid(&a, ""));
        let h = ausgabe("HEADLESS-1", 1920, 1080, 0, 0);
        assert!(!ohne_edid(&h, ""));
        assert!(ohne_edid(&h, "HEADLESS-2, HEADLESS-1"));
        assert!(!ohne_edid(&h, "HEADLESS-10"));
    }

    #[test]
    fn alte_ini_wird_gelesen_wie_bisher() {
        let ini = "# alt\n\n[Ausgabe HDMI-A-1]\nGeraet=Dell|U2412M|123\nAktiv=ja\nModus=1920x1080@60000\nPosition=0,0\nDrehung=90\nSkalierung=1.5\nHaupt=ja\n\n[Ausgabe DP-1]\nGeraet=||\nAktiv=nein\nModus=1024x768@60000\nPosition=1920,0\nDrehung=0\nSkalierung=1\nHaupt=nein\n";
        let (e, p) = ini_lesen(ini);
        assert!(p.is_empty(), "{p:?}");
        let mut a = ausgabe("HDMI-A-1", 1920, 1080, 0, 0);
        a.hersteller = "Dell".into();
        a.modell = "U2412M".into();
        a.seriennummer = "123".into();
        let b = ausgabe("DP-1", 1024, 768, 1920, 0);
        // dieselben Werte über die neue Datei
        let (zu, treffer) = zuordnen(&e, &[a.clone(), b.clone()]);
        assert_eq!(treffer, 2);
        let (e2, p2) = konfig_lesen(&konfig_schreiben(&zu, &[a.clone(), b.clone()]));
        assert!(p2.is_empty(), "{p2:?}");
        let (zu2, _) = zuordnen(&e2, &[a, b]);
        assert_eq!(zu, zu2);
        assert!(zu[0].haupt && !zu[1].aktiv && zu[0].skalierung == 1.5);
    }

    #[test]
    fn toml_lesen_ist_tolerant() {
        let (e, p) = konfig_lesen("das ist [kein toml");
        assert!(e.is_empty());
        assert_eq!(p.len(), 1, "{p:?}");
        let (e, p) = konfig_lesen(
            "version = 2\nfremd = 1\n[[ausgabe]]\nname = \"X\"\naktiv = true\nskalierung = \"abc\"\nfarbe = 1\n[[ausgabe]]\nname = \"Y\"\naktiv = false\n[[ausgabe]]\naktiv = true\n[[ausgabe]]\nname = \"Z\"\nbreite = 1600\nhoehe = 900\nskalierung = 2\nx = 1.5\n",
        );
        let namen: Vec<&str> = e.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            namen,
            ["Y", "Z"],
            "X ohne Größe, dritte ohne Namen: übergangen"
        );
        assert_eq!(e[1].wahl.modus, Modus::neu(1600, 900, 0));
        assert_eq!(e[1].wahl.skalierung, 2.0, "ganze Zahl gilt als Skalierung");
        assert_eq!(
            p.len(),
            7,
            "version, fremd, skalierung, farbe, X übergangen, ohne Namen, x: {p:?}"
        );
    }

    #[test]
    fn ini_lesen_ist_tolerant() {
        let (e, p) =
            ini_lesen("[Ausgabe X]\nAktiv=ja\nSkalierung=abc\nFremd=1\n[Sonst]\nModus=1x1@0\n");
        assert!(e.is_empty(), "Gruppe ohne Modus wird übergangen: {e:?}");
        assert_eq!(p.len(), 5, "{p:?}");
        let (e, p) = ini_lesen("[Ausgabe Y]\nAktiv=nein\n");
        assert_eq!(e.len(), 1, "ausgeschaltete Ausgabe braucht keinen Modus");
        assert!(p.is_empty());
    }
}
