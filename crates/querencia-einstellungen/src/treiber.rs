//! Seite „Treiber“ (0.2.0, QD-170) nach dem Muster „Zusätzliche Treiber“
//! von Ubuntu: Grafikkarten mit geladenem Kerneltreiber und OpenGL-Renderer,
//! je Gerät die Treiber aus `ubuntu-drivers devices` zur Wahl, Wechsel über
//! PackageKit (derselbe Weg wie Querencia Software, `qs_paketkit`; polkit
//! fragt nach dem Passwort), danach der Hinweis auf den Neustart mit einem
//! Knopf, der die Rückfrage der Sitzung öffnet (`querencia-sitzung
//! --neustart`). Vorgabe von Querencia ist der freie Treiber.
//!
//! Die Seite liest erst, wenn sie zum ersten Mal sichtbar wird
//! (`ubuntu-drivers` braucht einige Sekunden); das Kontrollzentrum baut alle
//! Seiten beim Start.

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gio, glib};
use qs_paketkit::PaketKit;
use qs_typen::{Abbruch, Fehler, Fortschritt, Paketquelle, Vorgang};

use crate::i18n::{gettext, gettext_f};
use crate::treiber_modell::{self as modell, Befund, Liste, UdGeraet, UdTreiber, Zustand};

/// Sichtbarer Name der Seite.
pub const NAME: &str = "Drivers";

struct Seite {
    /// Grafik und Treiberliste; wird bei jedem Lesen neu gefüllt.
    inhalt: gtk4::Box,
    /// Hinweis zum Software-OpenGL unter der Liste; ebenso.
    hinweis: gtk4::Box,
    meldung: gtk4::Label,
    fortschritt: gtk4::ProgressBar,
    abbrechen: gtk4::Button,
    neustart: gtk4::Box,
    neustart_knopf: gtk4::Button,
    /// Nach einem Wechsel: Knopf „Neu starten …“ fokussieren, sobald neu
    /// gelesen ist (Return startet dann die Rückfrage).
    neustart_zeigen: Cell<bool>,
    befund: RefCell<Option<Befund>>,
    /// Je Gerät die Knöpfe der Treiber (gleiche Reihenfolge wie `treiber`).
    wahl: RefCell<Vec<(UdGeraet, Vec<gtk4::CheckButton>)>>,
    anwenden: RefCell<Option<gtk4::Button>>,
    zuruecksetzen: RefCell<Option<gtk4::Button>>,
    beschaeftigt: Cell<bool>,
    griff: RefCell<Option<qs_paketkit::AbbruchGriff>>,
}

pub fn seite() -> gtk4::Widget {
    let s = crate::bereiche::seite(&gettext(NAME));
    let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);

    let meldung = gtk4::Label::new(None);
    meldung.set_xalign(0.0);
    meldung.set_wrap(true);
    meldung.set_max_width_chars(70);
    meldung.set_selectable(true);
    meldung.set_visible(false);
    let fortschritt = gtk4::ProgressBar::new();
    fortschritt.set_show_text(true);
    fortschritt.set_visible(false);
    let abbrechen = gtk4::Button::with_label(&gettext("Cancel"));
    abbrechen.set_halign(gtk4::Align::Start);
    abbrechen.set_visible(false);

    let neustart = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    neustart.set_visible(false);
    let neustart_text = gtk4::Label::new(Some(&gettext(
        "Restart the computer to load the new driver.",
    )));
    neustart_text.set_xalign(0.0);
    neustart_text.set_wrap(true);
    neustart_text.set_hexpand(true);
    let neustart_knopf = gtk4::Button::with_label(&gettext("Restart\u{a0}…"));
    neustart_knopf.set_tooltip_text(Some(&gettext(
        "Opens the restart question of the session; nothing restarts without your confirmation.",
    )));
    neustart.append(&neustart_text);
    neustart.append(&neustart_knopf);

    let hinweis = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
    // Fortschritt, Meldung und Neustart-Hinweis oben unter der Überschrift:
    // dort bleiben sie sichtbar, auch wenn die Liste lang ist.
    s.append(&fortschritt);
    s.append(&abbrechen);
    s.append(&meldung);
    s.append(&neustart);
    s.append(&inhalt);
    s.append(&hinweis);
    s.append(&crate::bereiche::quelle(&gettext(
        "ubuntu-drivers (devices, list), /sys/bus/pci/devices, dpkg, querencia-sitzung --grafik-pruefen; changes through PackageKit",
    )));

    let o = Rc::new(Seite {
        inhalt,
        hinweis,
        meldung,
        fortschritt,
        abbrechen,
        neustart,
        neustart_knopf: neustart_knopf.clone(),
        neustart_zeigen: Cell::new(false),
        befund: RefCell::new(None),
        wahl: RefCell::new(Vec::new()),
        anwenden: RefCell::new(None),
        zuruecksetzen: RefCell::new(None),
        beschaeftigt: Cell::new(false),
        griff: RefCell::new(None),
    });
    let o2 = o.clone();
    o.abbrechen.connect_clicked(move |_| {
        if let Some(g) = o2.griff.borrow().as_ref() {
            g.abbrechen();
        }
    });
    let o2 = o.clone();
    neustart_knopf.connect_clicked(move |_| o2.neu_starten());
    let o2 = o.clone();
    let gelesen = Cell::new(false);
    s.connect_map(move |_| {
        if !gelesen.replace(true) {
            o2.lesen();
        }
    });
    s.upcast()
}

fn absatz(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(Some(text));
    l.set_xalign(0.0);
    l.set_wrap(true);
    l.set_max_width_chars(70);
    l
}

fn ueberschrift(text: &str) -> gtk4::Label {
    let l = gtk4::Label::new(None);
    l.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(text)));
    l.set_xalign(0.0);
    l.set_margin_top(4);
    l
}

fn klein(text: &str) -> gtk4::Label {
    let l = absatz(text);
    l.add_css_class("dim-label");
    l
}

/// Name eines Treibers in der Liste.
fn treibername(t: &UdTreiber) -> String {
    if let Some(z) = modell::nvidia_zweig(&t.paket) {
        let v = [("version", z.nummer.as_str())];
        return match (z.server, z.offen) {
            (false, true) => gettext_f("NVIDIA driver {version} with open kernel modules", &v),
            (false, false) => gettext_f("NVIDIA driver {version}", &v),
            (true, true) => gettext_f(
                "NVIDIA server driver {version} with open kernel modules",
                &v,
            ),
            (true, false) => gettext_f("NVIDIA server driver {version}", &v),
        };
    }
    match modell::modul_von(&t.paket) {
        Some("nouveau") => gettext("Free driver nouveau (Querencia default)"),
        Some(m) if t.eingebaut => gettext_f("Free driver {module}", &[("module", m)]),
        _ => t.paket.clone(),
    }
}

/// Zweite Zeile: Paket, Lizenz, Empfehlung, Zustand.
fn treiberzeile(t: &UdTreiber, z: Zustand) -> String {
    let mut teile = vec![
        gettext_f("package {package}", &[("package", &t.paket)]),
        if t.frei {
            gettext("free")
        } else {
            gettext("proprietary")
        },
    ];
    if t.empfohlen {
        teile.push(gettext("recommended by Ubuntu"));
    }
    if t.fremdquelle {
        teile.push(gettext("third-party source"));
    }
    teile.push(match z {
        Zustand::Aktiv => gettext("in use"),
        Zustand::NachNeustart => gettext("installed, in use after a restart"),
        Zustand::Verfuegbar => gettext("available"),
        Zustand::NichtInstalliert => gettext("not installed"),
    });
    teile.join(" · ")
}

/// Meldung für einen Fehler der Paketverwaltung.
fn fehlertext(f: &Fehler) -> String {
    match f {
        Fehler::NichtBerechtigt => gettext("Not authorized: the password was not accepted."),
        Fehler::KeinAgent => gettext(
            "The password dialog of the session (polkit agent) is not running. Log out and in again.",
        ),
        Fehler::SperreBelegt(_) => gettext(
            "Another program is installing software right now. Try again when it has finished.",
        ),
        Fehler::NetzFehlt => gettext("No network connection."),
        Fehler::PaketNichtGefunden(t) => {
            gettext_f("Package not found: {details}", &[("details", t)])
        }
        Fehler::AbhaengigkeitenKonflikt(t) => {
            gettext_f("Packages conflict: {details}", &[("details", t)])
        }
        Fehler::DaemonWeg => gettext("The package management (PackageKit) stopped."),
        Fehler::Abgebrochen => gettext("Cancelled."),
        Fehler::Sonstiges { code, text } => gettext_f(
            "Package management error: {code} {details}",
            &[("code", code), ("details", text)],
        ),
    }
}

impl Seite {
    fn lesen(self: &Rc<Self>) {
        for kasten in [&self.inhalt, &self.hinweis] {
            while let Some(k) = kasten.first_child() {
                kasten.remove(&k);
            }
        }
        self.wahl.borrow_mut().clear();
        let warten = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let spinner = gtk4::Spinner::new();
        spinner.start();
        warten.append(&spinner);
        warten.append(&absatz(&gettext(
            "Looking for graphics cards and drivers\u{a0}…",
        )));
        self.inhalt.append(&warten);
        let o = self.clone();
        let auftrag = gio::spawn_blocking(modell::ermitteln);
        glib::spawn_future_local(async move {
            while let Some(k) = o.inhalt.first_child() {
                o.inhalt.remove(&k);
            }
            match auftrag.await {
                Ok(b) => {
                    eprintln!(
                        "querencia-einstellungen: Treiber gelesen: {}",
                        befund_zeile(&b)
                    );
                    *o.befund.borrow_mut() = Some(b.clone());
                    o.aufbauen(&b);
                    if o.neustart_zeigen.replace(false) {
                        o.neustart_knopf.grab_focus();
                    }
                }
                Err(_) => o
                    .inhalt
                    .append(&absatz(&gettext("The search for drivers failed."))),
            }
        });
    }

    fn aufbauen(self: &Rc<Self>, b: &Befund) {
        self.grafik_zeigen(b);
        self.treiber_zeigen(b);
        self.hinweis_zeigen(b);
    }

    fn grafik_zeigen(&self, b: &Befund) {
        self.inhalt.append(&ueberschrift(&gettext("Graphics")));
        let raster = gtk4::Grid::new();
        raster.set_row_spacing(4);
        raster.set_column_spacing(12);
        let mut zeile = 0;
        let mut setzen = |k: String, v: String| {
            let kl = gtk4::Label::new(Some(&k));
            kl.set_xalign(1.0);
            kl.set_yalign(0.0);
            kl.add_css_class("dim-label");
            let vl = absatz(&v);
            vl.set_selectable(true);
            vl.set_hexpand(true);
            raster.attach(&kl, 0, zeile, 1, 1);
            raster.attach(&vl, 1, zeile, 1, 1);
            zeile += 1;
        };
        if b.karten.is_empty() {
            setzen(gettext("Graphics card:"), gettext("none found"));
        }
        for k in &b.karten {
            let hersteller = modell::hersteller_name(k.hersteller)
                .map(str::to_string)
                .unwrap_or_else(|| format!("{:04x}", k.hersteller));
            let name = b
                .namen
                .get(&k.slot)
                .cloned()
                .unwrap_or_else(|| format!("{:04x}:{:04x}", k.hersteller, k.geraet));
            setzen(
                gettext("Graphics card:"),
                format!(
                    "{hersteller} {name} ({})",
                    k.slot.trim_start_matches("0000:")
                ),
            );
            setzen(
                gettext("Kernel driver:"),
                match k.modul.as_deref() {
                    Some(m) if modell::modul_frei(m) => {
                        gettext_f("{module} (free)", &[("module", m)])
                    }
                    Some(m) => gettext_f("{module} (proprietary)", &[("module", m)]),
                    None => gettext("none loaded"),
                },
            );
        }
        setzen(
            gettext("OpenGL renderer:"),
            match &b.renderer {
                Ok(r) => r.renderer.clone(),
                Err(e) if e.is_empty() => gettext("not measured"),
                Err(e) => gettext_f("not measured ({reason})", &[("reason", e)]),
            },
        );
        setzen(
            gettext("This session:"),
            if b.software_gl {
                gettext("software OpenGL (llvmpipe), set by the session at login")
            } else if b
                .renderer
                .as_ref()
                .is_ok_and(|r| modell::ist_zink_auf_nvk(&r.renderer))
            {
                gettext("OpenGL on the graphics card through zink on NVK")
            } else {
                gettext("OpenGL on the graphics card")
            },
        );
        self.inhalt.append(&raster);
    }

    fn treiber_zeigen(self: &Rc<Self>, b: &Befund) {
        self.inhalt
            .append(&ueberschrift(&gettext("Available Drivers")));
        self.inhalt.append(&klein(&gettext(
            "Querencia default: the free driver. Choose the NVIDIA driver only if you need it.",
        )));
        let geraete = match &b.liste {
            Liste::WerkzeugFehlt => {
                self.inhalt.append(&absatz(&gettext(
                    "The list of drivers comes from the program ubuntu-drivers (package ubuntu-drivers-common), which is not installed.",
                )));
                let k = gtk4::Button::with_label(&gettext("Install ubuntu-drivers-common\u{a0}…"));
                k.set_halign(gtk4::Align::Start);
                let o = self.clone();
                k.connect_clicked(move |k| {
                    k.set_sensitive(false);
                    o.ausfuehren(
                        Vec::new(),
                        vec![modell::UBUNTU_DRIVERS_PAKET.to_string()],
                        false,
                    );
                });
                self.inhalt.append(&k);
                return;
            }
            Liste::Fehler(e) => {
                self.inhalt.append(&absatz(&gettext_f(
                    "ubuntu-drivers did not answer: {error}",
                    &[("error", e)],
                )));
                self.inhalt.append(&self.knopfleiste(false));
                return;
            }
            Liste::Geraete { geraete, .. } => geraete,
        };
        if geraete.is_empty() {
            self.inhalt.append(&absatz(&gettext(
                "This computer needs no additional drivers.",
            )));
            self.inhalt.append(&self.knopfleiste(false));
            return;
        }
        // Grafik zuerst, dann andere Geräte, die ubuntu-drivers nennt
        let mut geraete: Vec<&UdGeraet> = geraete.iter().collect();
        geraete.sort_by_key(|g| !g.ist_grafik());
        for g in geraete {
            let modul = b.modul_fuer(g);
            let rahmen = gtk4::Frame::new(Some(&format!("{}: {}", g.hersteller, g.modell)));
            let spalte = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
            spalte.set_margin_top(6);
            spalte.set_margin_bottom(8);
            spalte.set_margin_start(10);
            spalte.set_margin_end(10);
            let gewaehlt = modell::gewaehlt(g, &b.installiert, modul);
            let mut knoepfe: Vec<gtk4::CheckButton> = Vec::new();
            // Gruppe auch bei nur einem Treiber, damit er als Optionsknopf
            // erscheint (ein CheckButton ohne Gruppe zeichnet ein Häkchen)
            let anker = gtk4::CheckButton::new();
            for (i, t) in g.treiber.iter().enumerate() {
                let z = modell::zustand(t, &b.installiert, modul);
                let k = gtk4::CheckButton::new();
                let text = gtk4::Box::new(gtk4::Orientation::Vertical, 1);
                let name = gtk4::Label::new(Some(&treibername(t)));
                name.set_xalign(0.0);
                name.set_wrap(true);
                text.append(&name);
                let zeile = gtk4::Label::new(None);
                zeile.set_markup(&format!(
                    "<small>{}</small>",
                    glib::markup_escape_text(&treiberzeile(t, z))
                ));
                zeile.set_xalign(0.0);
                zeile.set_wrap(true);
                zeile.add_css_class("dim-label");
                text.append(&zeile);
                k.set_child(Some(&text));
                k.set_group(Some(&anker));
                k.set_active(gewaehlt == Some(i));
                let o = self.clone();
                k.connect_toggled(move |_| o.geaendert());
                spalte.append(&k);
                knoepfe.push(k);
            }
            spalte.append(&klein(&zustandssatz(g, &b.installiert, modul)));
            if modell::neustart_noetig(g, &b.installiert, modul) {
                self.neustart.set_visible(true);
            }
            rahmen.set_child(Some(&spalte));
            self.inhalt.append(&rahmen);
            self.wahl.borrow_mut().push((g.clone(), knoepfe));
        }
        self.inhalt.append(&self.knopfleiste(true));
        self.geaendert();
    }

    fn knopfleiste(self: &Rc<Self>, mit_wahl: bool) -> gtk4::Box {
        let leiste = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let neu = gtk4::Button::with_label(&gettext("Search Again"));
        let o = self.clone();
        neu.connect_clicked(move |_| {
            if !o.beschaeftigt.get() {
                o.lesen();
            }
        });
        leiste.append(&neu);
        if mit_wahl {
            let platz = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            platz.set_hexpand(true);
            leiste.append(&platz);
            let zurueck = gtk4::Button::with_label(&gettext("Revert"));
            let anwenden = gtk4::Button::with_label(&gettext("Apply Changes"));
            anwenden.add_css_class("suggested-action");
            let o = self.clone();
            zurueck.connect_clicked(move |_| o.zuruecksetzen());
            let o = self.clone();
            anwenden.connect_clicked(move |k| o.anwenden_klick(k));
            leiste.append(&zurueck);
            leiste.append(&anwenden);
            *self.anwenden.borrow_mut() = Some(anwenden);
            *self.zuruecksetzen.borrow_mut() = Some(zurueck);
        }
        leiste
    }

    fn hinweis_zeigen(&self, b: &Befund) {
        let nvidia = b.karten.iter().any(modell::Karte::ist_nvidia);
        if !nvidia && !b.software_gl {
            return;
        }
        self.hinweis
            .append(&ueberschrift(&gettext("Software OpenGL in the session")));
        for text in [
            gettext(
                "With the free nouveau driver, programs draw OpenGL through Mesa zink on the Vulkan driver NVK. On NVIDIA cards this made web pages in Messages and Firefox crash or freeze. That is why the session switches OpenGL to software (llvmpipe) at login when it finds zink on NVK. Vulkan stays on the graphics card.",
            ),
            gettext(
                "Everything keeps working, but OpenGL is drawn by the processor: video, 3D and web pages with WebGL are slower and need more memory.",
            ),
            gettext(
                "The NVIDIA driver makes the fallback unnecessary: OpenGL then runs on the graphics card, and the session notices this at the next login by itself.",
            ),
        ] {
            self.hinweis.append(&absatz(&text));
        }
        let ini = match b.grafik_ini {
            "an" => gettext(
                "~/.config/querencia/grafik.ini switches software OpenGL on for every login (Software-OpenGL=an).",
            ),
            "aus" => gettext(
                "~/.config/querencia/grafik.ini switches software OpenGL off for every login (Software-OpenGL=aus).",
            ),
            _ => gettext(
                "The session decides at every login (automatic). It can be fixed in ~/.config/querencia/grafik.ini: [Grafik] Software-OpenGL = an or aus.",
            ),
        };
        self.hinweis.append(&klein(&ini));
    }

    /// Gewählte Treiber gegen den Ist-Stand: was ein Klick auf „Anwenden“ täte.
    fn geplant(&self) -> modell::Wechsel {
        let mut w = modell::Wechsel::default();
        let befund = self.befund.borrow();
        let (Some(b), wahl) = (befund.as_ref(), self.wahl.borrow()) else {
            return w;
        };
        let Liste::Geraete { module, .. } = &b.liste else {
            return w;
        };
        for (g, knoepfe) in wahl.iter() {
            let Some(i) = knoepfe.iter().position(|k| k.is_active()) else {
                continue;
            };
            if Some(i) == modell::gewaehlt(g, &b.installiert, b.modul_fuer(g)) {
                continue;
            }
            let teil = modell::wechsel(g, &g.treiber[i], &b.installiert, module);
            for p in teil.entfernen {
                if !w.entfernen.contains(&p) {
                    w.entfernen.push(p);
                }
            }
            for p in teil.installieren {
                if !w.installieren.contains(&p) {
                    w.installieren.push(p);
                }
            }
        }
        w
    }

    fn geaendert(&self) {
        let offen = !self.geplant().ist_leer() && !self.beschaeftigt.get();
        if let Some(k) = self.anwenden.borrow().as_ref() {
            k.set_sensitive(offen);
        }
        if let Some(k) = self.zuruecksetzen.borrow().as_ref() {
            k.set_sensitive(offen);
        }
    }

    fn zuruecksetzen(&self) {
        let befund = self.befund.borrow();
        let Some(b) = befund.as_ref() else { return };
        for (g, knoepfe) in self.wahl.borrow().iter() {
            if let Some(i) = modell::gewaehlt(g, &b.installiert, b.modul_fuer(g)) {
                knoepfe[i].set_active(true);
            }
        }
    }

    fn anwenden_klick(self: &Rc<Self>, knopf: &gtk4::Button) {
        let w = self.geplant();
        if w.ist_leer() {
            return;
        }
        let eltern = knopf.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
        let dialog = gtk4::Window::builder()
            .title(gettext("Change Driver?"))
            .modal(true)
            .default_width(480)
            .resizable(false)
            .build();
        if let Some(e) = &eltern {
            dialog.set_transient_for(Some(e));
        }
        let inhalt = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        inhalt.set_margin_top(16);
        inhalt.set_margin_bottom(14);
        inhalt.set_margin_start(16);
        inhalt.set_margin_end(16);
        for (titel, pakete) in [
            (gettext("These packages will be removed:"), &w.entfernen),
            (
                gettext("These packages will be installed:"),
                &w.installieren,
            ),
        ] {
            if pakete.is_empty() {
                continue;
            }
            inhalt.append(&absatz(&titel));
            let liste = klein(&pakete.join("\n"));
            liste.set_margin_start(12);
            inhalt.append(&liste);
        }
        inhalt.append(&absatz(&gettext(
            "Your password is needed. The new driver is loaded after a restart of the computer.",
        )));
        let knoepfe = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        knoepfe.set_halign(gtk4::Align::End);
        let nein = gtk4::Button::with_label(&gettext("Cancel"));
        let ja = gtk4::Button::with_label(&gettext("Change Driver"));
        ja.add_css_class("suggested-action");
        knoepfe.append(&nein);
        knoepfe.append(&ja);
        inhalt.append(&knoepfe);
        dialog.set_child(Some(&inhalt));
        let d = dialog.clone();
        nein.connect_clicked(move |_| d.destroy());
        let d = dialog.clone();
        let o = self.clone();
        ja.connect_clicked(move |_| {
            d.destroy();
            o.ausfuehren(w.entfernen.clone(), w.installieren.clone(), true);
        });
        dialog.present();
    }

    fn beschaeftigt_setzen(&self, an: bool) {
        self.beschaeftigt.set(an);
        self.fortschritt.set_visible(an);
        self.abbrechen.set_visible(an);
        for (_, knoepfe) in self.wahl.borrow().iter() {
            for k in knoepfe {
                k.set_sensitive(!an);
            }
        }
        self.geaendert();
    }

    fn melden(&self, text: &str) {
        self.meldung.set_text(text);
        self.meldung.set_visible(!text.is_empty());
    }

    /// Entfernt und installiert über PackageKit (erst entfernen, dann
    /// installieren); danach neu lesen und, wenn verlangt, den Neustart
    /// anbieten.
    fn ausfuehren(
        self: &Rc<Self>,
        entfernen: Vec<String>,
        installieren: Vec<String>,
        treiber: bool,
    ) {
        if self.beschaeftigt.get() {
            return;
        }
        self.melden("");
        self.beschaeftigt_setzen(true);
        self.fortschritt.set_fraction(0.0);
        self.fortschritt.set_text(Some(&gettext(
            "Connecting to the package management\u{a0}…",
        )));
        let o = self.clone();
        glib::spawn_future_local(async move {
            let ergebnis = o.ausfuehren_async(&entfernen, &installieren).await;
            o.griff.borrow_mut().take();
            o.beschaeftigt_setzen(false);
            match ergebnis {
                Ok(()) => {
                    if treiber {
                        o.melden(&gettext("The driver was changed."));
                        o.neustart.set_visible(true);
                        o.neustart_zeigen.set(true);
                    } else {
                        o.melden(&gettext("Installed."));
                    }
                    eprintln!(
                        "querencia-einstellungen: Treiber: entfernt [{}], installiert [{}]",
                        entfernen.join(" "),
                        installieren.join(" ")
                    );
                    o.lesen();
                }
                Err(f) => {
                    eprintln!("querencia-einstellungen: Treiber: {f:?}");
                    o.melden(&fehlertext(&f));
                    o.lesen();
                }
            }
        });
    }

    async fn ausfuehren_async(
        &self,
        entfernen: &[String],
        installieren: &[String],
    ) -> Result<(), Fehler> {
        let pk = PaketKit::neu().await?;
        if !entfernen.is_empty() {
            let mut ids = Vec::new();
            for name in entfernen {
                // Resolve mit „newest“ liefert bei neuerer Fassung im Archiv
                // nicht die installierte: deshalb über die installierten suchen.
                if let Some(p) = pk
                    .suchen(name, true)
                    .await?
                    .into_iter()
                    .find(|p| &p.name == name && p.installiert)
                {
                    ids.push(p.id);
                }
            }
            if !ids.is_empty() {
                self.vorgang(&pk, Vorgang::Entfernen, ids).await?;
            }
        }
        if !installieren.is_empty() {
            let mut ids = Vec::new();
            for name in installieren {
                match pk.paket_nach_name(name).await? {
                    Some(p) if p.installiert => {}
                    Some(p) => ids.push(p.id),
                    None => return Err(Fehler::PaketNichtGefunden(name.clone())),
                }
            }
            if !ids.is_empty() {
                self.vorgang(&pk, Vorgang::Installieren, ids).await?;
            }
        }
        Ok(())
    }

    async fn vorgang(
        &self,
        pk: &PaketKit,
        vorgang: Vorgang,
        ids: Vec<String>,
    ) -> Result<(), Fehler> {
        let text = match vorgang {
            Vorgang::Entfernen => gettext("Removing packages\u{a0}…"),
            _ => gettext("Downloading and installing packages\u{a0}…"),
        };
        self.fortschritt.set_text(Some(&text));
        self.fortschritt.set_fraction(0.0);
        let (empfaenger, griff) = pk.ausfuehren_kanal(vorgang, ids);
        *self.griff.borrow_mut() = Some(griff);
        let mut ende = Err(Fehler::DaemonWeg);
        while let Some(f) = empfaenger.naechster().await {
            match f {
                Fortschritt::Prozent(p) if p <= 100 => {
                    self.fortschritt.set_fraction(f64::from(p) / 100.0);
                    self.fortschritt
                        .set_text(Some(&format!("{text} {p}\u{a0}%")));
                }
                Fortschritt::Prozent(_) => self.fortschritt.pulse(),
                Fortschritt::WartetAufSperre => self.fortschritt.set_text(Some(&gettext(
                    "Waiting for another program that installs software\u{a0}…",
                ))),
                Fortschritt::Fertig => ende = Ok(()),
                Fortschritt::Abgebrochen => ende = Err(Fehler::Abgebrochen),
                Fortschritt::Fehler(e) => ende = Err(e),
                _ => {}
            }
        }
        ende
    }

    fn neu_starten(&self) {
        match std::process::Command::new("querencia-sitzung")
            .arg("--neustart")
            .spawn()
        {
            Ok(mut kind) => {
                std::thread::spawn(move || kind.wait());
            }
            Err(e) => self.melden(&gettext_f(
                "Could not open the restart question: {error}",
                &[("error", &e.to_string())],
            )),
        }
    }
}

/// Kurzfassung des Befunds für das Protokoll (Prüfung, Fehlersuche).
fn befund_zeile(b: &Befund) -> String {
    let karten: Vec<String> = b
        .karten
        .iter()
        .map(|k| format!("{}:{}", k.slot, k.modul.as_deref().unwrap_or("-")))
        .collect();
    let liste = match &b.liste {
        Liste::WerkzeugFehlt => "ubuntu-drivers fehlt".to_string(),
        Liste::Fehler(e) => format!("Fehler {e}"),
        Liste::Geraete { geraete, .. } => geraete
            .iter()
            .map(|g| {
                let modul = b.modul_fuer(g);
                let treiber: Vec<String> = g
                    .treiber
                    .iter()
                    .map(|t| {
                        format!(
                            "{}={:?}",
                            t.paket,
                            modell::zustand(t, &b.installiert, modul)
                        )
                    })
                    .collect();
                format!("{}[{}]", g.slot().unwrap_or("?"), treiber.join(" "))
            })
            .collect::<Vec<_>>()
            .join(" "),
    };
    format!(
        "karten=[{}] liste=[{}] renderer={} software-gl={} grafik.ini={}",
        karten.join(" "),
        liste,
        match &b.renderer {
            Ok(r) => format!("„{}“", r.renderer),
            Err(e) => format!("nicht gemessen ({e})"),
        },
        b.software_gl,
        b.grafik_ini
    )
}

/// Ein Satz zum Stand des Geräts unter der Liste.
fn zustandssatz(g: &UdGeraet, installiert: &BTreeSet<String>, modul: Option<&str>) -> String {
    let aktiv = g
        .treiber
        .iter()
        .find(|t| modell::zustand(t, installiert, modul) == Zustand::Aktiv);
    let wartet = g
        .treiber
        .iter()
        .find(|t| modell::zustand(t, installiert, modul) == Zustand::NachNeustart);
    match (aktiv, wartet) {
        (_, Some(t)) => gettext_f(
            "{driver} is installed and is used after a restart.",
            &[("driver", &treibername(t))],
        ),
        (Some(t), None) if t.frei => gettext("A free driver is in use."),
        (Some(_), None) => gettext("A proprietary driver is in use."),
        (None, None) => match modul {
            Some(m) if modell::neustart_noetig(g, installiert, modul) => gettext_f(
                "The driver {module} stays loaded until the restart.",
                &[("module", m)],
            ),
            _ => gettext("None of these drivers is loaded right now."),
        },
    }
}
