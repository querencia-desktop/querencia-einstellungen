//! Ausgaben lesen und setzen über `wlr-output-management-unstable-v1`
//! (labwc 0.9.8). Eigene Wayland-Verbindung neben der von GTK, wie
//! `toplevel::query_desktop()` im Screenshot 0.2.5. Alle Aufrufe sind
//! blockierend mit kurzen Roundtrips; der Compositor antwortet sofort.
//!
//! Regeln des Protokolls, die hier eingehalten werden: Eine Konfiguration
//! muss **jede** gemeldete Ausgabe nennen (enable_head oder disable_head),
//! sonst `unconfigured_head`; sie trägt die Seriennummer des letzten `done`,
//! sonst antwortet der Compositor mit `cancelled`.
use std::collections::HashMap;
use std::time::{Duration, Instant};

use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
    backend::ObjectId,
    event_created_child,
    protocol::{wl_output, wl_registry},
};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1,
    zwlr_output_configuration_v1::{self, ZwlrOutputConfigurationV1},
    zwlr_output_head_v1::{self, ZwlrOutputHeadV1},
    zwlr_output_manager_v1::{self, ZwlrOutputManagerV1},
    zwlr_output_mode_v1::{self, ZwlrOutputModeV1},
};

use crate::bildschirm_modell::{Ausgabe, Drehung, Modus, Wahl};
use crate::i18n::{gettext, gettext_f};

#[derive(Default)]
struct Kopf {
    name: String,
    beschreibung: String,
    hersteller: String,
    modell: String,
    seriennummer: String,
    physisch: (i32, i32),
    modi: Vec<ObjectId>,
    aktiv: bool,
    aktueller_modus: Option<ObjectId>,
    position: (i32, i32),
    transform: u32,
    skalierung: f64,
    beendet: bool,
}

#[derive(Default)]
struct Sammler {
    manager: Option<ZwlrOutputManagerV1>,
    manager_version: u32,
    serial: u32,
    done: bool,
    koepfe: Vec<(ZwlrOutputHeadV1, Kopf)>,
    modi: HashMap<ObjectId, (ZwlrOutputModeV1, Modus)>,
    ergebnis: Option<Ergebnis>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ergebnis {
    Erfolg,
    Fehlgeschlagen,
    Abgebrochen,
}

pub struct Verbindung {
    verbindung: Connection,
    warteschlange: EventQueue<Sammler>,
    sammler: Sammler,
}

const ZEITGRENZE: Duration = Duration::from_secs(5);

impl Verbindung {
    /// Verbindet sich mit dem Compositor aus `WAYLAND_DISPLAY` und liest die
    /// Ausgaben, bis das erste `done` da ist.
    pub fn oeffnen() -> Result<Self, String> {
        let verbindung = Connection::connect_to_env().map_err(|e| {
            gettext_f(
                "no Wayland connection: {error}",
                &[("error", &e.to_string())],
            )
        })?;
        let warteschlange = verbindung.new_event_queue();
        let griff = warteschlange.handle();
        verbindung.display().get_registry(&griff, ());
        let mut v = Self {
            verbindung,
            warteschlange,
            sammler: Sammler::default(),
        };
        v.roundtrip()?;
        if v.sammler.manager.is_none() {
            return Err(gettext(
                "the compositor does not offer zwlr_output_manager_v1",
            ));
        }
        v.warten_bis(|s| s.done)?;
        Ok(v)
    }

    fn roundtrip(&mut self) -> Result<(), String> {
        self.warteschlange
            .roundtrip(&mut self.sammler)
            .map(drop)
            .map_err(|e| {
                gettext_f(
                    "Wayland query failed: {error}",
                    &[("error", &e.to_string())],
                )
            })
    }

    /// Wartet mit Roundtrips (nie ohne Zeitgrenze blockierend), bis `fertig`
    /// gilt. labwc antwortet auf test/apply sofort; ein hängender Compositor
    /// ergibt nach 5 s einen Fehler statt einer eingefrorenen Oberfläche.
    fn warten_bis(&mut self, fertig: impl Fn(&Sammler) -> bool) -> Result<(), String> {
        let start = Instant::now();
        loop {
            self.roundtrip()?;
            if fertig(&self.sammler) {
                return Ok(());
            }
            if start.elapsed() > ZEITGRENZE {
                return Err(gettext("the compositor does not respond (5 s)"));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Aktueller Zustand aller Ausgaben (ein Roundtrip holt anstehende
    /// Änderungen nach).
    pub fn lesen(&mut self) -> Result<Vec<Ausgabe>, String> {
        self.roundtrip()?;
        Ok(self.sammler.ausgaben())
    }

    pub fn protokollversion(&self) -> u32 {
        self.sammler.manager_version
    }

    /// Prüft (`nur_test`) oder setzt die Wahl. Jede Ausgabe wird genannt;
    /// Ausgaben, für die keine Wahl vorliegt, bleiben wie sie sind.
    pub fn anwenden(&mut self, wahlen: &[Wahl], nur_test: bool) -> Result<(), String> {
        self.roundtrip()?;
        let manager = self
            .sammler
            .manager
            .clone()
            .ok_or_else(|| gettext("no zwlr_output_manager_v1"))?;
        let griff = self.warteschlange.handle();
        let konfiguration = manager.create_configuration(self.sammler.serial, &griff, ());
        let mut koepfe: Vec<(ZwlrOutputHeadV1, Wahl, Vec<ObjectId>)> = Vec::new();
        for (kopf, daten) in self.sammler.koepfe.iter().filter(|(_, k)| !k.beendet) {
            let wahl = wahlen
                .iter()
                .find(|w| w.name == daten.name)
                .cloned()
                .unwrap_or_else(|| Wahl::aus_ausgabe(&self.sammler.ausgabe(daten)));
            koepfe.push((kopf.clone(), wahl, daten.modi.clone()));
        }
        for (kopf, wahl, modi) in &koepfe {
            if !wahl.aktiv {
                konfiguration.disable_head(kopf);
                continue;
            }
            let kk = konfiguration.enable_head(kopf, &griff, ());
            let modus_objekt = modi
                .iter()
                .filter_map(|id| self.sammler.modi.get(id))
                .find(|(_, m)| m.gleich(&wahl.modus))
                .map(|(o, _)| o.clone());
            match modus_objekt {
                Some(m) => kk.set_mode(&m),
                None => kk.set_custom_mode(wahl.modus.breite, wahl.modus.hoehe, wahl.modus.mhz),
            }
            kk.set_position(wahl.position.0, wahl.position.1);
            kk.set_transform(transform_aus(wahl.drehung));
            kk.set_scale(wahl.skalierung);
        }
        if nur_test {
            konfiguration.test();
        } else {
            konfiguration.apply();
        }
        self.sammler.ergebnis = None;
        self.verbindung.flush().map_err(|e| format!("{e}"))?;
        let ergebnis = self.warten_bis(|s| s.ergebnis.is_some());
        konfiguration.destroy();
        ergebnis?;
        match self.sammler.ergebnis {
            Some(Ergebnis::Erfolg) => {
                // labwc schickt den neuen Zustand (Köpfe + done) noch vor
                // `succeeded`; ein weiterer Roundtrip holt Nachzügler.
                if !nur_test {
                    self.roundtrip()?;
                }
                Ok(())
            }
            Some(Ergebnis::Fehlgeschlagen) => Err(if nur_test {
                gettext("the compositor rejects this setting (test failed)")
            } else {
                gettext("the compositor could not apply the setting")
            }),
            Some(Ergebnis::Abgebrochen) | None => Err(gettext(
                "the outputs have changed in the meantime; please try again",
            )),
        }
    }
}

fn transform_aus(d: Drehung) -> wl_output::Transform {
    match d {
        Drehung::Normal => wl_output::Transform::Normal,
        Drehung::Rechts90 => wl_output::Transform::_90,
        Drehung::Um180 => wl_output::Transform::_180,
        Drehung::Links270 => wl_output::Transform::_270,
    }
}

impl Sammler {
    fn kopf(&mut self, id: &ObjectId) -> Option<&mut Kopf> {
        self.koepfe
            .iter_mut()
            .find(|(k, _)| k.id() == *id)
            .map(|(_, daten)| daten)
    }

    fn ausgabe(&self, k: &Kopf) -> Ausgabe {
        let modi: Vec<Modus> = k
            .modi
            .iter()
            .filter_map(|id| self.modi.get(id))
            .map(|(_, m)| *m)
            .collect();
        Ausgabe {
            name: k.name.clone(),
            beschreibung: k.beschreibung.clone(),
            hersteller: k.hersteller.clone(),
            modell: k.modell.clone(),
            seriennummer: k.seriennummer.clone(),
            physisch_mm: k.physisch,
            aktiv: k.aktiv,
            modus: k
                .aktueller_modus
                .as_ref()
                .and_then(|id| self.modi.get(id))
                .map(|(_, m)| *m),
            position: k.position,
            drehung: Drehung::aus_transform(k.transform),
            skalierung: if k.skalierung > 0.0 {
                k.skalierung
            } else {
                1.0
            },
            modi,
        }
    }

    fn ausgaben(&self) -> Vec<Ausgabe> {
        self.koepfe
            .iter()
            .filter(|(_, k)| !k.beendet && !k.name.is_empty())
            .map(|(_, k)| self.ausgabe(k))
            .collect()
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for Sammler {
    fn event(
        s: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        griff: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == ZwlrOutputManagerV1::interface().name
        {
            let v = version.min(4);
            s.manager_version = v;
            s.manager = Some(registry.bind(name, v, griff, ()));
        }
    }
}

impl Dispatch<ZwlrOutputManagerV1, ()> for Sammler {
    fn event(
        s: &mut Self,
        _: &ZwlrOutputManagerV1,
        event: zwlr_output_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_manager_v1::Event::Head { head } => s.koepfe.push((
                head,
                Kopf {
                    skalierung: 1.0,
                    ..Kopf::default()
                },
            )),
            zwlr_output_manager_v1::Event::Done { serial } => {
                s.serial = serial;
                s.done = true;
            }
            _ => {}
        }
    }

    event_created_child!(Sammler, ZwlrOutputManagerV1, [
        zwlr_output_manager_v1::EVT_HEAD_OPCODE => (ZwlrOutputHeadV1, ()),
    ]);
}

impl Dispatch<ZwlrOutputHeadV1, ()> for Sammler {
    fn event(
        s: &mut Self,
        head: &ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use zwlr_output_head_v1::Event;
        if let Event::Mode { mode } = &event {
            s.modi
                .insert(mode.id(), (mode.clone(), Modus::neu(0, 0, 0)));
        }
        let Some(k) = s.kopf(&head.id()) else {
            return;
        };
        match event {
            Event::Name { name } => k.name = name,
            Event::Description { description } => k.beschreibung = description,
            Event::PhysicalSize { width, height } => k.physisch = (width, height),
            Event::Mode { mode } => k.modi.push(mode.id()),
            Event::Enabled { enabled } => k.aktiv = enabled != 0,
            Event::CurrentMode { mode } => k.aktueller_modus = Some(mode.id()),
            Event::Position { x, y } => k.position = (x, y),
            Event::Transform { transform } => {
                k.transform = match transform {
                    WEnum::Value(t) => t as u32,
                    WEnum::Unknown(u) => u,
                }
            }
            Event::Scale { scale } => k.skalierung = scale,
            Event::Make { make } => k.hersteller = make,
            Event::Model { model } => k.modell = model,
            Event::SerialNumber { serial_number } => k.seriennummer = serial_number,
            Event::Finished => k.beendet = true,
            _ => {}
        }
    }

    event_created_child!(Sammler, ZwlrOutputHeadV1, [
        zwlr_output_head_v1::EVT_MODE_OPCODE => (ZwlrOutputModeV1, ()),
    ]);
}

impl Dispatch<ZwlrOutputModeV1, ()> for Sammler {
    fn event(
        s: &mut Self,
        mode: &ZwlrOutputModeV1,
        event: zwlr_output_mode_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use zwlr_output_mode_v1::Event;
        match event {
            Event::Finished => {
                s.modi.remove(&mode.id());
                for (_, k) in s.koepfe.iter_mut() {
                    k.modi.retain(|id| *id != mode.id());
                    if k.aktueller_modus.as_ref() == Some(&mode.id()) {
                        k.aktueller_modus = None;
                    }
                }
            }
            _ => {
                let Some((_, m)) = s.modi.get_mut(&mode.id()) else {
                    return;
                };
                match event {
                    Event::Size { width, height } => {
                        m.breite = width;
                        m.hoehe = height;
                    }
                    Event::Refresh { refresh } => m.mhz = refresh,
                    Event::Preferred => m.bevorzugt = true,
                    _ => {}
                }
            }
        }
    }
}

impl Dispatch<ZwlrOutputConfigurationV1, ()> for Sammler {
    fn event(
        s: &mut Self,
        _: &ZwlrOutputConfigurationV1,
        event: zwlr_output_configuration_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        s.ergebnis = Some(match event {
            zwlr_output_configuration_v1::Event::Succeeded => Ergebnis::Erfolg,
            zwlr_output_configuration_v1::Event::Failed => Ergebnis::Fehlgeschlagen,
            _ => Ergebnis::Abgebrochen,
        });
    }
}

wayland_client::delegate_noop!(Sammler: ignore ZwlrOutputConfigurationHeadV1);

/// Textform für `--bildschirm-zeigen` und die Prüfwerkzeuge, eine Zeile je
/// Ausgabe.
pub fn zeilen(ausgaben: &[Ausgabe]) -> String {
    let mut text = String::new();
    for a in ausgaben {
        let w = Wahl::aus_ausgabe(a);
        let (lb, lh) = w.logische_groesse();
        text.push_str(&format!(
            "{} {} modus={} position={},{} drehung={} skalierung={} logisch={}x{} geraet={} modi={}\n",
            a.name,
            if a.aktiv { "an" } else { "aus" },
            a.modus.map(|m| m.dateiwert()).unwrap_or_else(|| "-".into()),
            a.position.0,
            a.position.1,
            a.drehung.dateiwert(),
            a.skalierung,
            lb,
            lh,
            a.geraet(),
            a.modi
                .iter()
                .map(|m| format!("{}{}", m.dateiwert(), if m.bevorzugt { "*" } else { "" }))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    text
}
