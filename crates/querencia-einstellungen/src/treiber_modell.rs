//! Modell der Seite „Treiber“ (0.2.0, QD-170): Grafikkarten aus sysfs,
//! Treiberliste aus `ubuntu-drivers`, installierte Pakete aus dpkg und der
//! OpenGL-Renderer aus `querencia-sitzung --grafik-pruefen`. Nur lesen und
//! zerlegen; geschrieben wird über PackageKit in `treiber.rs`.
//!
//! Vorbild ist „Zusätzliche Treiber“ von Ubuntu (software-properties-gtk mit
//! ubuntu-drivers-common): je Gerät die Treiber, die `ubuntu-drivers devices`
//! nennt, mit frei/proprietär, „empfohlen“ und „eingebaut“. Querencia nimmt
//! den freien Treiber als Vorgabe.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// sysfs-Ordner der PCI-Geräte.
pub const SYSFS_PCI: &str = "/sys/bus/pci/devices";
/// Prüfschalter: anderer sysfs-Ordner (Attrappe der GUI-Prüfung).
pub const UMGEBUNG_SYSFS: &str = "QE_TREIBER_SYSFS";
/// Die drei Variablen, mit denen die Sitzung Software-OpenGL einschaltet
/// (Sitzungsstarter, QD-169).
pub const SOFTWARE_GL: [&str; 3] = [
    "LIBGL_ALWAYS_SOFTWARE",
    "MESA_LOADER_DRIVER_OVERRIDE",
    "GALLIUM_DRIVER",
];
/// Pfad, den der Sitzungsstarter seinen Programmen mitgibt.
pub const UMGEBUNG_SITZUNG: &str = "QUERENCIA_SITZUNG";
/// Bringt `ubuntu-drivers`; fehlt es, bietet die Seite an, es zu installieren.
pub const UBUNTU_DRIVERS_PAKET: &str = "ubuntu-drivers-common";

// ------------------------------------------------------------------ sysfs

/// Eine Grafikkarte (PCI-Klasse 0x03).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Karte {
    /// PCI-Adresse wie `0000:01:00.0`.
    pub slot: String,
    pub hersteller: u16,
    pub geraet: u16,
    /// Geladenes Kernelmodul (Name des `driver`-Links), z. B. `nouveau`.
    pub modul: Option<String>,
    /// Firmware hat mit dieser Karte gestartet (`boot_vga`).
    pub boot_vga: bool,
}

impl Karte {
    pub fn ist_nvidia(&self) -> bool {
        self.hersteller == 0x10de
    }
}

fn hex_lesen(pfad: &Path) -> Option<u32> {
    let t = std::fs::read_to_string(pfad).ok()?;
    u32::from_str_radix(t.trim().trim_start_matches("0x"), 16).ok()
}

/// Liest die Grafikkarten aus `wurzel` (sonst [`SYSFS_PCI`]); sortiert nach
/// Adresse.
pub fn karten(wurzel: &Path) -> Vec<Karte> {
    let Ok(eintraege) = std::fs::read_dir(wurzel) else {
        return Vec::new();
    };
    let mut aus: Vec<Karte> = eintraege
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            let klasse = hex_lesen(&p.join("class"))?;
            if klasse >> 16 != 0x03 {
                return None;
            }
            let modul = link_name(&p.join("driver")).map(|m| {
                if m == "virtio-pci" {
                    virtio_modul(&p).unwrap_or(m)
                } else {
                    m
                }
            });
            Some(Karte {
                slot: e.file_name().to_string_lossy().into_owned(),
                hersteller: hex_lesen(&p.join("vendor")).unwrap_or(0) as u16,
                geraet: hex_lesen(&p.join("device")).unwrap_or(0) as u16,
                modul,
                boot_vga: std::fs::read_to_string(p.join("boot_vga"))
                    .is_ok_and(|t| t.trim() == "1"),
            })
        })
        .collect();
    aus.sort_by(|a, b| a.slot.cmp(&b.slot));
    aus
}

fn link_name(pfad: &Path) -> Option<String> {
    std::fs::read_link(pfad)
        .ok()
        .and_then(|z| z.file_name().map(|n| n.to_string_lossy().into_owned()))
}

/// Bei virtio hängt am PCI-Gerät nur `virtio-pci`; der Grafiktreiber
/// (`virtio_gpu`) sitzt am Kindgerät `virtio<n>`.
fn virtio_modul(geraet: &Path) -> Option<String> {
    std::fs::read_dir(geraet)
        .ok()?
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("virtio"))
        .find_map(|e| link_name(&e.path().join("driver")))
}

/// Ordner der PCI-Geräte: Prüfschalter oder [`SYSFS_PCI`].
pub fn sysfs_wurzel() -> std::path::PathBuf {
    std::env::var_os(UMGEBUNG_SYSFS)
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| SYSFS_PCI.into())
}

/// Herstellername zu bekannten PCI-Kennungen (Grafik und virtuelle Karten).
pub fn hersteller_name(id: u16) -> Option<&'static str> {
    Some(match id {
        0x10de => "NVIDIA",
        0x1002 => "AMD",
        0x8086 => "Intel",
        0x1af4 | 0x1b36 => "Red Hat (virtio/QXL)",
        0x1234 => "QEMU",
        0x15ad => "VMware",
        0x80ee => "VirtualBox",
        _ => return None,
    })
}

/// Gerätename aus `lspci -vmm -s <slot>` (Zeile `Device:`).
pub fn lspci_name(text: &str) -> Option<String> {
    text.lines()
        .find_map(|z| z.strip_prefix("Device:"))
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
}

/// Ist das Kernelmodul frei? (Alles außer dem NVIDIA-Originaltreiber.)
pub fn modul_frei(modul: &str) -> bool {
    !matches!(modul, "nvidia" | "nvidia_drm" | "nvidia-drm")
}

// ------------------------------------------------------------------ ubuntu-drivers

/// Ein Treiber aus `ubuntu-drivers devices`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UdTreiber {
    pub paket: String,
    pub frei: bool,
    pub empfohlen: bool,
    /// Kommt mit dem Kernel bzw. Mesa (`builtin`), kein eigenes Paket nötig.
    pub eingebaut: bool,
    /// Aus einer Fremdquelle (`third-party`) statt aus Ubuntu (`distro`).
    pub fremdquelle: bool,
}

/// Ein Gerät aus `ubuntu-drivers devices`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UdGeraet {
    /// sysfs-Pfad aus der Kopfzeile `== … ==`.
    pub pfad: String,
    pub modalias: String,
    pub hersteller: String,
    pub modell: String,
    pub treiber: Vec<UdTreiber>,
}

impl UdGeraet {
    /// PCI-Adresse (letzter Teil des Pfads), falls es eine ist.
    pub fn slot(&self) -> Option<&str> {
        let s = self.pfad.rsplit('/').next()?;
        (s.len() == 12 && s.as_bytes()[4] == b':').then_some(s)
    }

    /// Grafikgerät nach Modalias (`bc03`).
    pub fn ist_grafik(&self) -> bool {
        self.modalias.contains("bc03")
    }
}

/// Zerlegt die Ausgabe von `ubuntu-drivers devices`. Zeilen ohne Bezug
/// (Warnungen, leere Zeilen) werden übergangen.
pub fn devices_zerlegen(text: &str) -> Vec<UdGeraet> {
    let mut aus: Vec<UdGeraet> = Vec::new();
    for zeile in text.lines() {
        let z = zeile.trim();
        if let Some(pfad) = z.strip_prefix("==").and_then(|r| r.strip_suffix("==")) {
            aus.push(UdGeraet {
                pfad: pfad.trim().to_string(),
                ..UdGeraet::default()
            });
            continue;
        }
        let Some(g) = aus.last_mut() else { continue };
        let Some((schluessel, wert)) = z.split_once(':') else {
            continue;
        };
        let wert = wert.trim();
        match schluessel.trim() {
            "modalias" => g.modalias = wert.to_string(),
            "vendor" => g.hersteller = wert.to_string(),
            "model" => g.modell = wert.to_string(),
            "driver" => {
                if let Some(t) = treiberzeile(wert)
                    && !g.treiber.iter().any(|x| x.paket == t.paket)
                {
                    g.treiber.push(t);
                }
            }
            _ => {}
        }
    }
    aus
}

/// `nvidia-driver-615-open - distro non-free recommended`
fn treiberzeile(wert: &str) -> Option<UdTreiber> {
    let (paket, rest) = wert.split_once(" - ").unwrap_or((wert, ""));
    let paket = paket.trim();
    if paket.is_empty() || paket.contains(char::is_whitespace) {
        return None;
    }
    let merkmale: Vec<&str> = rest.split_whitespace().collect();
    Some(UdTreiber {
        paket: paket.to_string(),
        frei: merkmale.contains(&"free"),
        empfohlen: merkmale.contains(&"recommended"),
        eingebaut: merkmale.contains(&"builtin"),
        fremdquelle: merkmale.contains(&"third-party"),
    })
}

/// Zerlegt `ubuntu-drivers list`: Treiberpaket → Paket der fertigen
/// Kernelmodule (`nvidia-driver-615-open, (kernel modules provided by
/// linux-modules-nvidia-615-open-generic)`).
pub fn list_zerlegen(text: &str) -> HashMap<String, Option<String>> {
    let mut aus = HashMap::new();
    for zeile in text.lines() {
        let z = zeile.trim();
        let (paket, rest) = z.split_once(',').unwrap_or((z, ""));
        let paket = paket.trim();
        if paket.is_empty() || paket.contains(char::is_whitespace) || paket.contains(':') {
            continue;
        }
        let module = rest
            .split("provided by")
            .nth(1)
            .map(|r| r.trim().trim_end_matches(')').trim().to_string())
            .filter(|m| !m.is_empty() && !m.contains(char::is_whitespace));
        aus.insert(paket.to_string(), module);
    }
    aus
}

// ------------------------------------------------------------------ Pakete

/// Installierte Paketnamen aus `dpkg-query -W -f '${Package}\t${db:Status-Abbrev}\n'`
/// (nur Status `ii`; Architektur-Zusätze `:amd64` entfernt).
pub fn dpkg_zerlegen(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|z| {
            let (name, status) = z.split_once('\t')?;
            status
                .trim()
                .starts_with("ii")
                .then(|| name.split(':').next().unwrap_or(name).to_string())
        })
        .collect()
}

/// NVIDIA-Treiberzweig eines Pakets wie `nvidia-driver-615-open`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NvidiaZweig {
    pub nummer: String,
    pub offen: bool,
    pub server: bool,
}

pub fn nvidia_zweig(paket: &str) -> Option<NvidiaZweig> {
    let rest = paket.strip_prefix("nvidia-driver-")?;
    let mut teile = rest.split('-');
    let nummer = teile.next()?;
    if nummer.is_empty() || !nummer.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut zweig = NvidiaZweig {
        nummer: nummer.to_string(),
        offen: false,
        server: false,
    };
    for t in teile {
        match t {
            "open" => zweig.offen = true,
            "server" => zweig.server = true,
            _ => return None,
        }
    }
    Some(zweig)
}

/// Kernelmodul, das ein Treiberpaket bringt (für „aktiv“).
pub fn modul_von(paket: &str) -> Option<&'static str> {
    if nvidia_zweig(paket).is_some() {
        return Some("nvidia");
    }
    Some(match paket.strip_prefix("xserver-xorg-video-")? {
        "nouveau" => "nouveau",
        "amdgpu" => "amdgpu",
        "radeon" => "radeon",
        "intel" => "i915",
        _ => return None,
    })
}

/// Alle installierten Pakete, die zu einem NVIDIA-Zweig gehören und beim
/// Wechsel auf den freien Treiber gehen müssen: das Treiberpaket, die
/// Bibliotheken und die Kernelmodule desselben Zweigs. Allgemeine Pakete ohne
/// Nummer (z. B. `libnvidia-egl-wayland1`) bleiben. Für andere Pakete nur
/// das Paket selbst, wenn es installiert ist.
pub fn entfernen_satz(installiert: &BTreeSet<String>, paket: &str) -> Vec<String> {
    let Some(zweig) = nvidia_zweig(paket) else {
        return if installiert.contains(paket) {
            vec![paket.to_string()]
        } else {
            Vec::new()
        };
    };
    const PRAEFIXE: [&str; 5] = [
        "nvidia-",
        "libnvidia-",
        "xserver-xorg-video-nvidia-",
        "linux-modules-nvidia-",
        "linux-objects-nvidia-",
    ];
    installiert
        .iter()
        .filter(|n| {
            PRAEFIXE.iter().any(|p| n.starts_with(p)) && n.split('-').any(|t| t == zweig.nummer)
        })
        .cloned()
        .collect()
}

/// Zustand eines Treibers für ein Gerät.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zustand {
    /// Installiert (oder eingebaut) und geladen.
    Aktiv,
    /// Installiert, aber noch nicht geladen: wirkt nach dem Neustart.
    NachNeustart,
    /// Eingebaut, aber gerade nicht geladen (ein anderer Treiber ist aktiv).
    Verfuegbar,
    NichtInstalliert,
}

pub fn zustand(t: &UdTreiber, installiert: &BTreeSet<String>, modul: Option<&str>) -> Zustand {
    let vorhanden = t.eingebaut || installiert.contains(&t.paket);
    let geladen = matches!((modul_von(&t.paket), modul), (Some(a), Some(b)) if a == b);
    match (vorhanden, geladen) {
        (true, true) => Zustand::Aktiv,
        (true, false) if t.eingebaut => Zustand::Verfuegbar,
        (true, false) => Zustand::NachNeustart,
        (false, _) => Zustand::NichtInstalliert,
    }
}

/// Der Treiber, den die Seite als gewählt zeigt: ein installiertes eigenes
/// Paket (auch wenn es erst nach dem Neustart lädt), sonst der geladene
/// eingebaute, sonst der erste freie eingebaute.
pub fn gewaehlt(
    g: &UdGeraet,
    installiert: &BTreeSet<String>,
    modul: Option<&str>,
) -> Option<usize> {
    let zustaende: Vec<Zustand> = g
        .treiber
        .iter()
        .map(|t| zustand(t, installiert, modul))
        .collect();
    g.treiber
        .iter()
        .zip(&zustaende)
        .position(|(t, z)| !t.eingebaut && matches!(z, Zustand::Aktiv | Zustand::NachNeustart))
        .or_else(|| zustaende.iter().position(|z| *z == Zustand::Aktiv))
        .or_else(|| g.treiber.iter().position(|t| t.eingebaut && t.frei))
}

/// Muss neu gestartet werden, damit der installierte Stand gilt? Ja, wenn
/// ein installierter Treiber noch nicht geladen ist oder das geladene
/// Kernelmodul zu keinem vorhandenen Treiber mehr gehört (Paket entfernt).
pub fn neustart_noetig(g: &UdGeraet, installiert: &BTreeSet<String>, modul: Option<&str>) -> bool {
    let zustaende: Vec<Zustand> = g
        .treiber
        .iter()
        .map(|t| zustand(t, installiert, modul))
        .collect();
    zustaende.contains(&Zustand::NachNeustart)
        || (modul.is_some_and(|m| g.treiber.iter().any(|t| modul_von(&t.paket) == Some(m)))
            && !zustaende.contains(&Zustand::Aktiv))
}

/// Was ein Wechsel auf `ziel` tut.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Wechsel {
    pub entfernen: Vec<String>,
    pub installieren: Vec<String>,
}

impl Wechsel {
    pub fn ist_leer(&self) -> bool {
        self.entfernen.is_empty() && self.installieren.is_empty()
    }
}

/// Plant den Wechsel auf den Treiber `ziel` des Geräts `g`: andere
/// installierte eigene Treiber (samt Zweig) entfernen, das Zielpaket und
/// sein Kernelmodul-Paket aus `ubuntu-drivers list` installieren.
pub fn wechsel(
    g: &UdGeraet,
    ziel: &UdTreiber,
    installiert: &BTreeSet<String>,
    module: &HashMap<String, Option<String>>,
) -> Wechsel {
    let mut w = Wechsel::default();
    for t in &g.treiber {
        if t.paket != ziel.paket && !t.eingebaut && installiert.contains(&t.paket) {
            for p in entfernen_satz(installiert, &t.paket) {
                if !w.entfernen.contains(&p) {
                    w.entfernen.push(p);
                }
            }
        }
    }
    if !ziel.eingebaut {
        let zusatz = module.get(&ziel.paket).cloned().flatten();
        for p in std::iter::once(ziel.paket.clone()).chain(zusatz) {
            if w.entfernen.contains(&p) {
                w.entfernen.retain(|e| e != &p);
            } else if !installiert.contains(&p) {
                w.installieren.push(p);
            }
        }
    }
    w
}

// ------------------------------------------------------------------ OpenGL

/// Ausgabe von `querencia-sitzung --grafik-pruefen` (`schlüssel=wert`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Renderer {
    pub renderer: String,
    pub vendor: String,
    pub version: String,
}

pub fn renderer_zerlegen(text: &str) -> Option<Renderer> {
    let mut r = Renderer::default();
    for zeile in text.lines() {
        let Some((k, v)) = zeile.split_once('=') else {
            continue;
        };
        match k.trim() {
            "renderer" => r.renderer = v.trim().to_string(),
            "vendor" => r.vendor = v.trim().to_string(),
            "version" => r.version = v.trim().to_string(),
            _ => {}
        }
    }
    (!r.renderer.is_empty()).then_some(r)
}

/// Software-Renderer von Mesa: llvmpipe, softpipe (und swrast älterer Mesa).
pub fn ist_software(renderer: &str) -> bool {
    let r = renderer.to_ascii_lowercase();
    r.starts_with("llvmpipe") || r.starts_with("softpipe") || r.contains("swrast")
}

/// Der proprietäre NVIDIA-Treiber zeichnet: sein Renderer heißt
/// „NVIDIA …/PCIe/…“ mit Hersteller „NVIDIA Corporation“ (zink auf NVK
/// nennt die Karte auch, aber mit Hersteller Mesa).
pub fn ist_nvidia_renderer(r: &Renderer) -> bool {
    r.vendor.starts_with("NVIDIA") || (r.renderer.starts_with("NVIDIA") && r.vendor.is_empty())
}

/// Wie diese Sitzung OpenGL zeichnet (0.2.1, Befund P-1 aus Alpha 3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Darstellung {
    /// Auf der Grafikkarte über einen freien Treiber (Mesa).
    Frei,
    /// Auf der Grafikkarte über den proprietären NVIDIA-Treiber.
    Proprietaer,
    /// Software, weil die Sitzung beim Anmelden umgeschaltet hat (die drei
    /// Variablen); die Karte selbst könnte OpenGL.
    SoftwareUmgeschaltet,
    /// Software, weil es keine 3D-Beschleunigung gibt (z. B. VM ohne 3D):
    /// schon die Messung ohne die Variablen liefert llvmpipe/softpipe.
    SoftwareOhne3d,
    Unbekannt,
}

/// Leitet die Darstellung aus den Messungen und den Kernelmodulen ab, nicht
/// aus der Paketliste. `sitzung`: Renderer mit der Umgebung dieser Sitzung;
/// `karte`: ohne die Software-Variablen (Hardware-Weg). Die Messung geht
/// den Variablen vor (der NVIDIA-Treiber beachtet sie nicht); nur ohne
/// Messung zählen die Variablen allein.
pub fn darstellung(
    sitzung: Option<&Renderer>,
    karte: Option<&Renderer>,
    software_gl: bool,
    module: &[&str],
) -> Darstellung {
    // Mit den Variablen sagt die Messung der Karte nichts über die Sitzung.
    let gemessen = if software_gl {
        sitzung
    } else {
        sitzung.or(karte)
    };
    let Some(r) = gemessen else {
        return match (
            software_gl,
            karte.is_some_and(|k| ist_software(&k.renderer)),
        ) {
            (true, false) => Darstellung::SoftwareUmgeschaltet,
            (true, true) => Darstellung::SoftwareOhne3d,
            (false, _) => Darstellung::Unbekannt,
        };
    };
    if ist_software(&r.renderer) {
        let karte_kann = karte.is_some_and(|k| !ist_software(&k.renderer));
        return if software_gl && karte_kann {
            Darstellung::SoftwareUmgeschaltet
        } else {
            Darstellung::SoftwareOhne3d
        };
    }
    if ist_nvidia_renderer(r) && module.iter().any(|m| !modul_frei(m)) {
        Darstellung::Proprietaer
    } else {
        Darstellung::Frei
    }
}

/// Software-OpenGL der Sitzung: steht eine der drei Variablen in der
/// Umgebung, mit der dieses Programm gestartet wurde?
pub fn software_gl_in_umgebung(lesen: impl Fn(&str) -> Option<String>) -> bool {
    lesen("LIBGL_ALWAYS_SOFTWARE").is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        || lesen("GALLIUM_DRIVER").is_some_and(|v| v == "llvmpipe" || v == "softpipe")
        || lesen("MESA_LOADER_DRIVER_OVERRIDE").is_some_and(|v| v == "llvmpipe")
}

/// `[Grafik] Software-OpenGL` aus `grafik.ini` wie der Sitzungsstarter:
/// letzter Wert der Gruppe, klein; unbekannt oder fehlend = `automatisch`.
pub fn grafik_ini_wert(text: &str) -> &'static str {
    let mut in_gruppe = false;
    let mut wert = "automatisch";
    for zeile in text.lines() {
        let z = zeile.trim();
        if z.starts_with('[') {
            in_gruppe = z == "[Grafik]";
            continue;
        }
        if !in_gruppe {
            continue;
        }
        if let Some((k, v)) = z.split_once('=')
            && k.trim() == "Software-OpenGL"
        {
            wert = match v.trim().to_lowercase().as_str() {
                "an" => "an",
                "aus" => "aus",
                _ => "automatisch",
            };
        }
    }
    wert
}

// ------------------------------------------------------------------ Befund

/// Ergebnis von `ubuntu-drivers` (oder warum es keins gibt).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Liste {
    Geraete {
        geraete: Vec<UdGeraet>,
        module: HashMap<String, Option<String>>,
    },
    WerkzeugFehlt,
    Fehler(String),
}

/// Alles, was die Seite zeigt; entsteht in einem Hintergrund-Thread.
#[derive(Clone, Debug)]
pub struct Befund {
    pub karten: Vec<Karte>,
    /// Gerätename je PCI-Adresse (ubuntu-drivers, sonst lspci).
    pub namen: HashMap<String, String>,
    pub liste: Liste,
    pub installiert: BTreeSet<String>,
    /// Renderer ohne die Software-Variablen gemessen (Hardware-Weg).
    pub renderer: Result<Renderer, String>,
    /// Renderer mit der Umgebung dieser Sitzung; nur gemessen, wenn
    /// [`Befund::software_gl`] gilt (sonst gleich `renderer`).
    pub renderer_sitzung: Option<Result<Renderer, String>>,
    pub software_gl: bool,
    pub grafik_ini: &'static str,
    /// Aufgerufenes `querencia-sitzung` (Pfad oder Name über `PATH`).
    pub sitzung_programm: String,
}

impl Befund {
    pub fn modul_fuer(&self, g: &UdGeraet) -> Option<&str> {
        let slot = g.slot()?;
        self.karten
            .iter()
            .find(|k| k.slot == slot)
            .and_then(|k| k.modul.as_deref())
    }

    /// Was die Programme dieser Sitzung als Renderer bekommen.
    pub fn sitzungs_renderer(&self) -> &Result<Renderer, String> {
        self.renderer_sitzung.as_ref().unwrap_or(&self.renderer)
    }

    pub fn darstellung(&self) -> Darstellung {
        let module: Vec<&str> = self
            .karten
            .iter()
            .filter_map(|k| k.modul.as_deref())
            .collect();
        darstellung(
            self.sitzungs_renderer().as_ref().ok(),
            self.renderer.as_ref().ok(),
            self.software_gl,
            &module,
        )
    }
}

/// Führt ein Programm mit Zeitgrenze aus; stdout bei Exit 0.
pub fn ausfuehren(
    programm: &str,
    argumente: &[&str],
    ohne: &[&str],
    frist: std::time::Duration,
) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let mut befehl = Command::new(programm);
    befehl
        .args(argumente)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for v in ohne {
        befehl.env_remove(v);
    }
    let mut kind = befehl.spawn().map_err(|e| e.to_string())?;
    let start = std::time::Instant::now();
    loop {
        match kind.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if start.elapsed() > frist => {
                let _ = kind.kill();
                let _ = kind.wait();
                return Err(format!("{programm}: > {} s", frist.as_secs()));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(20)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let ausgabe = kind.wait_with_output().map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&ausgabe.stdout).into_owned();
    if ausgabe.status.success() {
        Ok(text)
    } else {
        let fehler = String::from_utf8_lossy(&ausgabe.stderr);
        Err(fehler.lines().last().unwrap_or("").trim().to_string())
    }
}

fn werkzeug_da(name: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|pfade| std::env::split_paths(&pfade).any(|d| ist_ausfuehrbar(&d.join(name))))
}

/// Wo `querencia-sitzung` liegt (0.2.1-2, P-2): erst der Pfad, den die
/// Sitzung selbst mitgibt (`QUERENCIA_SITZUNG`), dann die Ablage
/// `~/opt/querencia/bin`, dann `/usr/bin`, zuletzt der Name über `PATH`.
/// Auf der Entwicklungs-VM steht `~/opt/querencia/bin` nicht im `PATH`.
pub fn sitzung_programm(
    lesen: impl Fn(&str) -> Option<String>,
    usr_bin: &Path,
    ausfuehrbar: impl Fn(&Path) -> bool,
) -> std::path::PathBuf {
    const NAME: &str = "querencia-sitzung";
    let mut kandidaten: Vec<std::path::PathBuf> = Vec::new();
    if let Some(p) = lesen(UMGEBUNG_SITZUNG).filter(|p| Path::new(p).is_absolute()) {
        kandidaten.push(p.into());
    }
    if let Some(h) = lesen("HOME").filter(|h| !h.is_empty()) {
        kandidaten.push(Path::new(&h).join("opt/querencia/bin").join(NAME));
    }
    kandidaten.push(usr_bin.join(NAME));
    kandidaten
        .into_iter()
        .find(|p| ausfuehrbar(p))
        .unwrap_or_else(|| NAME.into())
}

fn ist_ausfuehrbar(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// Sammelt den Befund (blockierend, für `gio::spawn_blocking`).
pub fn ermitteln() -> Befund {
    use std::time::Duration;
    let wurzel = sysfs_wurzel();
    let echt = std::env::var_os(UMGEBUNG_SYSFS).is_none();
    let karten = karten(&wurzel);
    let liste = if !werkzeug_da("ubuntu-drivers") {
        Liste::WerkzeugFehlt
    } else {
        match ausfuehren(
            "ubuntu-drivers",
            &["devices"],
            &[],
            Duration::from_secs(120),
        ) {
            Ok(text) => {
                let module = ausfuehren("ubuntu-drivers", &["list"], &[], Duration::from_secs(120))
                    .map(|t| list_zerlegen(&t))
                    .unwrap_or_default();
                Liste::Geraete {
                    geraete: devices_zerlegen(&text),
                    module,
                }
            }
            Err(e) => Liste::Fehler(e),
        }
    };
    let mut namen = HashMap::new();
    if let Liste::Geraete { geraete, .. } = &liste {
        for g in geraete {
            if let (Some(slot), false) = (g.slot(), g.modell.is_empty()) {
                namen.insert(slot.to_string(), g.modell.clone());
            }
        }
    }
    for k in &karten {
        if echt
            && !namen.contains_key(&k.slot)
            && let Some(n) = ausfuehren(
                "lspci",
                &["-vmm", "-s", &k.slot],
                &[],
                Duration::from_secs(5),
            )
            .ok()
            .and_then(|t| lspci_name(&t))
        {
            namen.insert(k.slot.clone(), n);
        }
    }
    let installiert = ausfuehren(
        "dpkg-query",
        &["-W", "-f", "${Package}\t${db:Status-Abbrev}\n"],
        &[],
        Duration::from_secs(30),
    )
    .map(|t| dpkg_zerlegen(&t))
    .unwrap_or_default();
    let sitzung = sitzung_programm(
        |v| std::env::var(v).ok(),
        Path::new("/usr/bin"),
        ist_ausfuehrbar,
    );
    let sitzung = sitzung.to_string_lossy().into_owned();
    let messen = |ohne: &[&str]| {
        ausfuehren(
            &sitzung,
            &["--grafik-pruefen"],
            ohne,
            Duration::from_secs(5),
        )
        .and_then(|t| renderer_zerlegen(&t).ok_or_else(|| "renderer=".to_string()))
    };
    let renderer = messen(&SOFTWARE_GL);
    let software_gl = software_gl_in_umgebung(|v| std::env::var(v).ok());
    let renderer_sitzung = software_gl.then(|| messen(&[]));
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| Path::new(&h).join(".config")))
        .unwrap_or_default();
    let grafik_ini = std::fs::read_to_string(config.join("querencia/grafik.ini"))
        .map(|t| grafik_ini_wert(&t))
        .unwrap_or("automatisch");
    Befund {
        karten,
        namen,
        liste,
        installiert,
        renderer,
        renderer_sitzung,
        software_gl,
        grafik_ini,
        sitzung_programm: sitzung,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NVIDIA_NOUVEAU: &str = include_str!("../testdaten/treiber/devices-nvidia-nouveau.txt");
    const NVIDIA_ORIGINAL: &str = include_str!("../testdaten/treiber/devices-nvidia-original.txt");
    const AMD: &str = include_str!("../testdaten/treiber/devices-amd.txt");
    const INTEL: &str = include_str!("../testdaten/treiber/devices-intel.txt");
    const KEINE: &str = include_str!("../testdaten/treiber/devices-keine.txt");
    const LIST: &str = include_str!("../testdaten/treiber/list-nvidia.txt");
    const DPKG_NOUVEAU: &str = include_str!("../testdaten/treiber/dpkg-nouveau.txt");
    const DPKG_ORIGINAL: &str = include_str!("../testdaten/treiber/dpkg-nvidia-615.txt");

    fn menge(namen: &[&str]) -> BTreeSet<String> {
        namen.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn nvidia_mit_nouveau() {
        let g = devices_zerlegen(NVIDIA_NOUVEAU);
        assert_eq!(g.len(), 1);
        let g = &g[0];
        assert_eq!(g.slot(), Some("0000:01:00.0"));
        assert!(g.ist_grafik());
        assert_eq!(g.hersteller, "NVIDIA Corporation");
        assert_eq!(g.modell, "GA106 [GeForce RTX 3060 Lite Hash Rate]");
        let pakete: Vec<&str> = g.treiber.iter().map(|t| t.paket.as_str()).collect();
        assert_eq!(
            pakete,
            [
                "nvidia-driver-615-open",
                "nvidia-driver-615",
                "nvidia-driver-580-server-open",
                "xserver-xorg-video-nouveau"
            ]
        );
        let empfohlen: Vec<bool> = g.treiber.iter().map(|t| t.empfohlen).collect();
        assert_eq!(empfohlen, [true, false, false, false]);
        assert!(g.treiber[3].frei && g.treiber[3].eingebaut);
        assert!(!g.treiber[0].frei && !g.treiber[0].eingebaut && !g.treiber[0].fremdquelle);
        let inst = dpkg_zerlegen(DPKG_NOUVEAU);
        let z: Vec<Zustand> = g
            .treiber
            .iter()
            .map(|t| zustand(t, &inst, Some("nouveau")))
            .collect();
        assert_eq!(
            z,
            [
                Zustand::NichtInstalliert,
                Zustand::NichtInstalliert,
                Zustand::NichtInstalliert,
                Zustand::Aktiv
            ]
        );
        assert_eq!(gewaehlt(g, &inst, Some("nouveau")), Some(3));
    }

    #[test]
    fn nvidia_mit_originaltreiber() {
        let g = &devices_zerlegen(NVIDIA_ORIGINAL)[0];
        let inst = dpkg_zerlegen(DPKG_ORIGINAL);
        assert!(inst.contains("nvidia-driver-615-open"));
        assert!(
            inst.contains("libnvidia-gl-615"),
            "Architektur-Zusatz entfernt"
        );
        assert!(!inst.contains("nvidia-driver-580"), "Status rc zählt nicht");
        let z: Vec<Zustand> = g
            .treiber
            .iter()
            .map(|t| zustand(t, &inst, Some("nvidia")))
            .collect();
        assert_eq!(z[0], Zustand::Aktiv);
        assert_eq!(z.last(), Some(&Zustand::Verfuegbar));
        assert_eq!(gewaehlt(g, &inst, Some("nvidia")), Some(0));
        // installiert, aber noch nouveau geladen: wirkt nach dem Neustart
        assert_eq!(
            zustand(&g.treiber[0], &inst, Some("nouveau")),
            Zustand::NachNeustart
        );
        assert_eq!(gewaehlt(g, &inst, Some("nouveau")), Some(0));
    }

    #[test]
    fn amd_und_intel() {
        let a = devices_zerlegen(AMD);
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].treiber.len(), 1);
        assert!(a[0].treiber[0].frei && a[0].treiber[0].eingebaut);
        let inst = BTreeSet::new();
        assert_eq!(
            zustand(&a[0].treiber[0], &inst, Some("amdgpu")),
            Zustand::Aktiv
        );
        assert_eq!(gewaehlt(&a[0], &inst, Some("amdgpu")), Some(0));
        let i = devices_zerlegen(INTEL);
        assert_eq!(i.len(), 1);
        assert_eq!(i[0].modell, "Alder Lake-P GT2 [Iris Xe Graphics]");
        assert_eq!(i[0].treiber[0].paket, "xserver-xorg-video-intel");
        assert_eq!(
            zustand(&i[0].treiber[0], &inst, Some("i915")),
            Zustand::Aktiv
        );
        assert_eq!(
            zustand(&i[0].treiber[0], &inst, Some("xe")),
            Zustand::Verfuegbar
        );
    }

    #[test]
    fn keine_karte_und_rauschen() {
        assert!(devices_zerlegen(KEINE).is_empty());
        assert!(devices_zerlegen("").is_empty());
        // Zeilen vor dem ersten Gerät und kaputte Treiberzeilen fallen weg
        let g = devices_zerlegen(
            "driver : x - free\n== /sys/x ==\ndriver   : - distro\ndriver : a b - free\n",
        );
        assert_eq!(g.len(), 1);
        assert!(g[0].treiber.is_empty());
        assert_eq!(g[0].slot(), None);
        assert!(!g[0].ist_grafik());
    }

    #[test]
    fn fremdquelle_und_doppelte() {
        let g = devices_zerlegen(
            "== /sys/devices/pci0000:00/0000:00:02.0/0000:02:00.0 ==\n\
             modalias : pci:v000010DEd00002504bc03sc00i00\n\
             driver   : nvidia-driver-620 - third-party non-free\n\
             driver   : nvidia-driver-620 - third-party non-free\n",
        );
        assert_eq!(g[0].treiber.len(), 1);
        assert!(g[0].treiber[0].fremdquelle);
        assert_eq!(g[0].slot(), Some("0000:02:00.0"));
    }

    #[test]
    fn liste_mit_kernelmodulen() {
        let m = list_zerlegen(LIST);
        assert_eq!(
            m.get("nvidia-driver-615-open"),
            Some(&Some("linux-modules-nvidia-615-open-generic".to_string()))
        );
        assert_eq!(m.get("nvidia-driver-580-server-open"), Some(&None));
        assert!(!m.keys().any(|k| k.contains("WARNING")));
    }

    #[test]
    fn zweige() {
        assert_eq!(
            nvidia_zweig("nvidia-driver-615-open"),
            Some(NvidiaZweig {
                nummer: "615".into(),
                offen: true,
                server: false
            })
        );
        assert!(nvidia_zweig("nvidia-driver-580-server-open").is_some_and(|z| z.server && z.offen));
        assert_eq!(nvidia_zweig("nvidia-driver-x"), None);
        assert_eq!(nvidia_zweig("nvidia-driver-615-foo"), None);
        assert_eq!(modul_von("nvidia-driver-615"), Some("nvidia"));
        assert_eq!(modul_von("xserver-xorg-video-nouveau"), Some("nouveau"));
        assert_eq!(modul_von("oem-foo-meta"), None);
    }

    #[test]
    fn entfernen_nimmt_den_ganzen_zweig() {
        let inst = dpkg_zerlegen(DPKG_ORIGINAL);
        let satz = entfernen_satz(&inst, "nvidia-driver-615-open");
        assert!(satz.contains(&"nvidia-driver-615-open".to_string()));
        assert!(satz.contains(&"linux-modules-nvidia-615-open-generic".to_string()));
        assert!(satz.contains(&"libnvidia-gl-615".to_string()));
        assert!(satz.contains(&"nvidia-kernel-common-615".to_string()));
        assert!(satz.contains(&"nvidia-firmware-615-615.48.03".to_string()));
        assert!(!satz.contains(&"libnvidia-egl-wayland1".to_string()));
        assert!(!satz.contains(&"bash".to_string()));
        assert!(!satz.iter().any(|p| p.contains("6150")));
        assert_eq!(entfernen_satz(&menge(&["a"]), "a"), ["a"]);
        assert!(entfernen_satz(&menge(&["a"]), "b").is_empty());
    }

    #[test]
    fn neustart_erkennen() {
        let g = &devices_zerlegen(NVIDIA_NOUVEAU)[0];
        let ohne = dpkg_zerlegen(DPKG_NOUVEAU);
        let mit = dpkg_zerlegen(DPKG_ORIGINAL);
        assert!(!neustart_noetig(g, &ohne, Some("nouveau")));
        assert!(!neustart_noetig(g, &mit, Some("nvidia")));
        // installiert, noch nouveau geladen
        assert!(neustart_noetig(g, &mit, Some("nouveau")));
        // entfernt, aber nvidia noch geladen
        assert!(neustart_noetig(g, &ohne, Some("nvidia")));
        // kein Modul geladen, nichts installiert: kein Neustart-Hinweis
        assert!(!neustart_noetig(g, &ohne, None));
    }

    #[test]
    fn wechsel_planen() {
        let g = &devices_zerlegen(NVIDIA_NOUVEAU)[0];
        let module = list_zerlegen(LIST);
        // frei → NVIDIA: Treiber und fertige Kernelmodule installieren
        let inst = dpkg_zerlegen(DPKG_NOUVEAU);
        let w = wechsel(g, &g.treiber[0], &inst, &module);
        assert_eq!(
            w.installieren,
            [
                "nvidia-driver-615-open",
                "linux-modules-nvidia-615-open-generic"
            ]
        );
        assert!(w.entfernen.is_empty());
        // gewählter Treiber schon da: nichts zu tun
        assert!(wechsel(g, &g.treiber[3], &inst, &module).ist_leer());
        // NVIDIA → frei: den ganzen Zweig entfernen, nichts installieren
        let inst = dpkg_zerlegen(DPKG_ORIGINAL);
        let w = wechsel(g, &g.treiber[3], &inst, &module);
        assert!(w.installieren.is_empty());
        assert!(w.entfernen.contains(&"nvidia-driver-615-open".to_string()));
        assert!(
            w.entfernen
                .contains(&"linux-modules-nvidia-615-open-generic".to_string())
        );
        // 615-open → 615: anderer Treiber desselben Zweigs; Gemeinsames bleibt
        let w = wechsel(g, &g.treiber[1], &inst, &module);
        assert_eq!(
            w.installieren,
            ["nvidia-driver-615", "linux-modules-nvidia-615-generic"]
        );
        assert!(w.entfernen.contains(&"nvidia-driver-615-open".to_string()));
        assert!(!w.entfernen.contains(&"nvidia-driver-615".to_string()));
    }

    #[test]
    fn renderer_und_sitzung() {
        let r = renderer_zerlegen(
            "renderer=zink Vulkan 1.4(NVIDIA GeForce RTX 3060 (NVK GA106) (MESA_NVK))\n\
             vendor=Mesa\nversion=OpenGL ES 3.2 Mesa 26.0.8\ntreiber=zink\ndauer-ms=381\n",
        )
        .expect("Renderer");
        assert!(r.renderer.starts_with("zink") && !ist_software(&r.renderer));
        assert_eq!(r.vendor, "Mesa");
        assert!(!ist_nvidia_renderer(&r));
        assert_eq!(renderer_zerlegen("dauer-ms=3\n"), None);
        let umgebung = |paare: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                paare
                    .iter()
                    .find(|(a, _)| *a == k)
                    .map(|(_, b)| b.to_string())
            }
        };
        assert!(software_gl_in_umgebung(umgebung(&[
            ("LIBGL_ALWAYS_SOFTWARE", "1"),
            ("GALLIUM_DRIVER", "llvmpipe")
        ])));
        assert!(!software_gl_in_umgebung(umgebung(&[(
            "LIBGL_ALWAYS_SOFTWARE",
            "0"
        )])));
        assert!(!software_gl_in_umgebung(umgebung(&[])));
        assert_eq!(grafik_ini_wert("[Grafik]\nSoftware-OpenGL = AUS\n"), "aus");
        assert_eq!(
            grafik_ini_wert("[Grafik]\nSoftware-OpenGL=an\n[X]\nSoftware-OpenGL=aus\n"),
            "an"
        );
        assert_eq!(
            grafik_ini_wert("[Grafik]\nSoftware-OpenGL=vielleicht\n"),
            "automatisch"
        );
        assert_eq!(grafik_ini_wert(""), "automatisch");
    }

    fn grafik(name: &str) -> Renderer {
        let t = match name {
            "llvmpipe-vm" => include_str!("../testdaten/treiber/grafik-llvmpipe-vm.txt"),
            "llvmpipe-umgeschaltet" => {
                include_str!("../testdaten/treiber/grafik-llvmpipe-umgeschaltet.txt")
            }
            "softpipe" => include_str!("../testdaten/treiber/grafik-softpipe.txt"),
            "zink-nvk" => include_str!("../testdaten/treiber/grafik-zink-nvk.txt"),
            "radeonsi" => include_str!("../testdaten/treiber/grafik-radeonsi.txt"),
            "nvidia" => include_str!("../testdaten/treiber/grafik-nvidia.txt"),
            _ => unreachable!(),
        };
        renderer_zerlegen(t).expect("Renderer")
    }

    #[test]
    fn darstellung_aus_messung_und_modul() {
        use Darstellung::*;
        let llvm_vm = grafik("llvmpipe-vm");
        let llvm_sw = grafik("llvmpipe-umgeschaltet");
        let zink = grafik("zink-nvk");
        let radeon = grafik("radeonsi");
        let nv = grafik("nvidia");
        assert!(ist_software(&llvm_vm.renderer) && ist_software(&grafik("softpipe").renderer));
        assert!(!ist_software(&zink.renderer) && !ist_software(&radeon.renderer));
        assert!(ist_nvidia_renderer(&nv) && !ist_nvidia_renderer(&zink));
        // VM ohne 3D (Weg B, Alpha 3.4): llvmpipe schon ohne die Variablen
        assert_eq!(
            darstellung(None, Some(&llvm_vm), false, &["virtio_gpu"]),
            SoftwareOhne3d
        );
        assert_eq!(
            darstellung(None, Some(&grafik("softpipe")), false, &[]),
            SoftwareOhne3d
        );
        // VM, aber grafik.ini=an: die Karte kann trotzdem kein 3D
        assert_eq!(
            darstellung(Some(&llvm_sw), Some(&llvm_vm), true, &["virtio_gpu"]),
            SoftwareOhne3d
        );
        // zink auf NVK, die Sitzung hat umgeschaltet
        assert_eq!(
            darstellung(Some(&llvm_sw), Some(&zink), true, &["nouveau"]),
            SoftwareUmgeschaltet
        );
        // zink auf NVK ohne Software-Variablen (grafik.ini=aus)
        assert_eq!(darstellung(None, Some(&zink), false, &["nouveau"]), Frei);
        assert_eq!(darstellung(None, Some(&radeon), false, &["amdgpu"]), Frei);
        assert_eq!(
            darstellung(None, Some(&nv), false, &["nvidia"]),
            Proprietaer
        );
        // Der NVIDIA-Treiber beachtet die Mesa-Variablen nicht: die Messung zählt
        assert_eq!(
            darstellung(Some(&nv), Some(&nv), true, &["nvidia"]),
            Proprietaer
        );
        // Laptop: Intel zeichnet, NVIDIA-Modul nur für die zweite Karte
        let intel = renderer_zerlegen(
            "renderer=Mesa Intel(R) Iris(R) Xe Graphics (ADL GT2)\nvendor=Intel\n",
        )
        .expect("Renderer");
        assert_eq!(
            darstellung(None, Some(&intel), false, &["i915", "nvidia"]),
            Frei
        );
        // ohne Messung: nur die Variablen
        assert_eq!(
            darstellung(None, None, true, &["nouveau"]),
            SoftwareUmgeschaltet
        );
        assert_eq!(darstellung(None, None, false, &["nvidia"]), Unbekannt);
        assert_eq!(
            darstellung(None, Some(&zink), true, &["nouveau"]),
            SoftwareUmgeschaltet
        );
        assert_eq!(
            darstellung(None, Some(&llvm_vm), true, &["virtio_gpu"]),
            SoftwareOhne3d
        );
    }

    #[test]
    fn befund_nimmt_den_sitzungsrenderer() {
        let mut b = Befund {
            karten: vec![Karte {
                slot: "0000:01:00.0".into(),
                hersteller: 0x10de,
                geraet: 0x2504,
                modul: Some("nouveau".into()),
                boot_vga: true,
            }],
            namen: HashMap::new(),
            liste: Liste::WerkzeugFehlt,
            installiert: BTreeSet::new(),
            renderer: Ok(grafik("zink-nvk")),
            renderer_sitzung: None,
            software_gl: false,
            grafik_ini: "aus",
            sitzung_programm: String::new(),
        };
        assert_eq!(b.darstellung(), Darstellung::Frei);
        assert!(
            b.sitzungs_renderer()
                .as_ref()
                .is_ok_and(|r| r.renderer.starts_with("zink"))
        );
        b.software_gl = true;
        b.renderer_sitzung = Some(Ok(grafik("llvmpipe-umgeschaltet")));
        assert_eq!(b.darstellung(), Darstellung::SoftwareUmgeschaltet);
        assert!(
            b.sitzungs_renderer()
                .as_ref()
                .is_ok_and(|r| r.renderer.starts_with("llvmpipe"))
        );
        b.renderer_sitzung = Some(Err("x".into()));
        assert_eq!(b.darstellung(), Darstellung::SoftwareUmgeschaltet);
        b.renderer = Err("x".into());
        b.software_gl = false;
        b.renderer_sitzung = None;
        assert_eq!(b.darstellung(), Darstellung::Unbekannt);
        b.karten[0].modul = Some("nvidia".into());
        b.renderer = Ok(grafik("nvidia"));
        assert_eq!(b.darstellung(), Darstellung::Proprietaer);
    }

    #[test]
    fn virtio_grafiktreiber_am_kindgeraet() {
        let w = attrappe(
            "v",
            &[(
                "0000:00:01.0",
                "0x030000",
                "0x1af4",
                "0x1050",
                Some("virtio-pci"),
                true,
            )],
        );
        let kind = w.join("devices/0000:00:01.0/virtio0");
        std::fs::create_dir_all(&kind).unwrap();
        std::fs::create_dir_all(w.join("module/virtio_gpu")).unwrap();
        std::os::unix::fs::symlink(w.join("module/virtio_gpu"), kind.join("driver")).unwrap();
        let k = karten(&w.join("devices"));
        assert_eq!(k[0].modul.as_deref(), Some("virtio_gpu"));
        std::fs::remove_file(kind.join("driver")).unwrap();
        let k = karten(&w.join("devices"));
        assert_eq!(
            k[0].modul.as_deref(),
            Some("virtio-pci"),
            "ohne Kindtreiber"
        );
        let _ = std::fs::remove_dir_all(&w);
    }

    /// (Adresse, Klasse, Hersteller, Gerät, Modul, boot_vga)
    type Eintrag<'a> = (&'a str, &'a str, &'a str, &'a str, Option<&'a str>, bool);

    fn attrappe(name: &str, geraete: &[Eintrag]) -> std::path::PathBuf {
        let w = std::env::temp_dir().join(format!("qe-treiber-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&w);
        let module = w.join("module");
        for (slot, klasse, hersteller, geraet, modul, boot) in geraete {
            let d = w.join("devices").join(slot);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("class"), format!("{klasse}\n")).unwrap();
            std::fs::write(d.join("vendor"), format!("{hersteller}\n")).unwrap();
            std::fs::write(d.join("device"), format!("{geraet}\n")).unwrap();
            if *boot {
                std::fs::write(d.join("boot_vga"), "1\n").unwrap();
            }
            if let Some(m) = modul {
                std::fs::create_dir_all(module.join(m)).unwrap();
                std::os::unix::fs::symlink(module.join(m), d.join("driver")).unwrap();
            }
        }
        w
    }

    #[test]
    fn sysfs_erkennung() {
        let w = attrappe(
            "a",
            &[
                (
                    "0000:01:00.0",
                    "0x030000",
                    "0x10de",
                    "0x2504",
                    Some("nouveau"),
                    true,
                ),
                (
                    "0000:01:00.1",
                    "0x040300",
                    "0x10de",
                    "0x228e",
                    Some("snd_hda_intel"),
                    false,
                ),
                (
                    "0000:00:02.0",
                    "0x030000",
                    "0x8086",
                    "0x46a6",
                    Some("i915"),
                    false,
                ),
                ("0000:06:00.0", "0x038000", "0x1002", "0x73ff", None, false),
                ("0000:00:1f.0", "0x060100", "0x8086", "0x7a06", None, false),
            ],
        );
        let k = karten(&w.join("devices"));
        let slots: Vec<&str> = k.iter().map(|k| k.slot.as_str()).collect();
        assert_eq!(slots, ["0000:00:02.0", "0000:01:00.0", "0000:06:00.0"]);
        assert_eq!(k[1].modul.as_deref(), Some("nouveau"));
        assert!(k[1].ist_nvidia() && k[1].boot_vga);
        assert_eq!(k[1].geraet, 0x2504);
        assert_eq!(k[0].modul.as_deref(), Some("i915"));
        assert_eq!(k[2].modul, None, "Karte ohne geladenen Treiber");
        assert_eq!(hersteller_name(k[2].hersteller), Some("AMD"));
        let _ = std::fs::remove_dir_all(&w);
        let leer = attrappe(
            "b",
            &[("0000:00:1f.0", "0x060100", "0x8086", "0x7a06", None, false)],
        );
        assert!(karten(&leer.join("devices")).is_empty());
        assert!(karten(&leer.join("fehlt")).is_empty());
        let _ = std::fs::remove_dir_all(&leer);
    }

    #[test]
    fn befund_ordnet_module_zu() {
        let g = devices_zerlegen(NVIDIA_NOUVEAU);
        let b = Befund {
            karten: vec![Karte {
                slot: "0000:01:00.0".into(),
                hersteller: 0x10de,
                geraet: 0x2504,
                modul: Some("nouveau".into()),
                boot_vga: true,
            }],
            namen: HashMap::new(),
            liste: Liste::WerkzeugFehlt,
            installiert: BTreeSet::new(),
            renderer: Err(String::new()),
            renderer_sitzung: None,
            software_gl: false,
            grafik_ini: "automatisch",
            sitzung_programm: String::new(),
        };
        assert_eq!(b.modul_fuer(&g[0]), Some("nouveau"));
        assert!(modul_frei("nouveau") && !modul_frei("nvidia"));
        assert_eq!(
            lspci_name(
                "Slot:\t01:00.0\nClass:\tVGA compatible controller\nDevice:\tGA106 [GeForce RTX 3060]\n"
            ),
            Some("GA106 [GeForce RTX 3060]".into())
        );
    }

    #[test]
    fn sitzung_programm_wie_die_sitzung_aufloesen() {
        let w = std::env::temp_dir().join(format!("qe-treiber-sitzung-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&w);
        let heim = w.join("heim");
        let ablage = heim.join("opt/querencia/bin");
        let usr_bin = w.join("usr/bin");
        let eigen = w.join("eigen/querencia-sitzung");
        for d in [&ablage, &usr_bin, &w.join("eigen")] {
            std::fs::create_dir_all(d).unwrap();
        }
        let anlegen = |p: &Path, modus: u32| {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(p, "#!/bin/sh\n").unwrap();
            std::fs::set_permissions(p, std::fs::Permissions::from_mode(modus)).unwrap();
        };
        let h = heim.to_string_lossy().into_owned();
        let e = eigen.to_string_lossy().into_owned();
        let finden = |sitzung: Option<&str>| {
            let (h, s) = (h.clone(), sitzung.map(str::to_string));
            sitzung_programm(
                move |k| match k {
                    "HOME" => Some(h.clone()),
                    UMGEBUNG_SITZUNG => s.clone(),
                    _ => None,
                },
                &usr_bin,
                ist_ausfuehrbar,
            )
        };
        // nichts da: der Name, also PATH (Stubs der GUI-Prüfung)
        assert_eq!(finden(None), Path::new("querencia-sitzung"));
        anlegen(&usr_bin.join("querencia-sitzung"), 0o755);
        assert_eq!(finden(None), usr_bin.join("querencia-sitzung"));
        // nicht ausführbar zählt nicht
        anlegen(&ablage.join("querencia-sitzung"), 0o644);
        assert_eq!(finden(None), usr_bin.join("querencia-sitzung"));
        anlegen(&ablage.join("querencia-sitzung"), 0o755);
        assert_eq!(finden(None), ablage.join("querencia-sitzung"));
        // der Pfad der Sitzung geht vor; relativ oder fehlend zählt nicht
        assert_eq!(finden(Some(&e)), ablage.join("querencia-sitzung"));
        anlegen(&eigen, 0o755);
        assert_eq!(finden(Some(&e)), eigen);
        assert_eq!(
            finden(Some("querencia-sitzung")),
            ablage.join("querencia-sitzung")
        );
        let _ = std::fs::remove_dir_all(&w);
    }

    #[test]
    fn werkzeug_fehlt_wenn_nicht_im_pfad() {
        // Ein Name, den es nirgends gibt, gilt als fehlendes Werkzeug.
        assert!(!werkzeug_da("qe-gibt-es-nicht-ubuntu-drivers"));
        assert!(werkzeug_da("sh"));
        assert!(
            ausfuehren(
                "qe-gibt-es-nicht",
                &[],
                &[],
                std::time::Duration::from_secs(1)
            )
            .is_err()
        );
        assert_eq!(
            ausfuehren(
                "sh",
                &["-c", "echo ja"],
                &[],
                std::time::Duration::from_secs(5)
            )
            .as_deref(),
            Ok("ja\n")
        );
        assert!(
            ausfuehren(
                "sh",
                &["-c", "sleep 3"],
                &[],
                std::time::Duration::from_millis(200)
            )
            .is_err()
        );
    }
}
