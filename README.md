# Querencia Einstellungen

Kontrollzentrum im Stil von GNOME 2 für alle, die Querencia Desktop
benutzen. Teil von
[Querencia Desktop](https://github.com/querencia-desktop/querencia) – einer
klassischen Desktopumgebung im GNOME-2-Stil für Wayland (labwc, Rust, GTK 4).

*English summary:* A GNOME-2-style control centre for Querencia Desktop
(appearance, background, desktop icons, window design, fonts, keyboard,
mouse, display, workspaces, default applications, language and region).
Part of Querencia Desktop, a classic GNOME-2-style desktop for Wayland.
Alpha software; the user interface is available in English and German
(GNU gettext), the documentation is in German.

> **Alpha.** Die Seite „Panels“ zeigt den Aufbau nur an (Einstellen direkt am
> Panel); Lage und Höhe der Panels sind noch fest. Keine Sicherheits- oder
> Datenverlustgarantie.

## Zweck

- Ein Fenster mit Bereichen wie im GNOME-2-Kontrollzentrum:
  Erscheinungsbild, Hintergrundbild (eigenes Bild, Farbe oder Galerie der
  eingebauten Hintergründe), Symbole der Arbeitsfläche, Fensterdesign,
  Schriften, Panels, Tastatur (auch mehrere Belegungen mit Umschalttaste),
  Maus, Bildschirm (Auflösung und Anordnung über `wlr-output-management`),
  Arbeitsflächen, Standardanwendungen, Sprache und Region, Informationen
  über Querencia.
- Schreibt nur die eigenen Einstellungen: `~/.config/querencia/desktop.ini`
  (nur die eigene Gruppe), die labwc-Benutzerkopie unter
  `~/.config/querencia/labwc/` (gezielt geänderte Werte in `rc.xml`, eigene
  Umgebungsdatei `environment.d/zz-querencia-einstellungen.env`, Sicherung
  vor jedem Ersetzen, `labwc --reconfigure`), gsettings
  `org.gnome.desktop.interface`, MIME-Zuordnungen über GIO.
- Bewusst **nicht** Teil dieses Programms: Netzwerk, Lautstärke, Benutzer und
  Software (eigene Programme von Querencia Desktop), Bereiche anderer
  Desktopumgebungen.

Anwendungskennung (App-ID): `org.querencia.Einstellungen` · Binary:
`querencia-einstellungen` · Debian-Paket: noch keins

| Ort | Inhalt |
|---|---|
| `crates/querencia-einstellungen/src/` | Fenster und Bereiche, Modelle, `ini.rs`, labwc-Konfiguration (`labwc_konfig.rs`, `labwc_xml.rs`), Übersetzung (`i18n.rs`) |
| `crates/querencia-einstellungen/po/` | Übersetzungen: Vorlage `querencia-einstellungen.pot`, `de.po`, `en.po`, `LINGUAS` |
| `crates/querencia-einstellungen/data/` | Programm- und Fenstersymbol (SVG), Querencia-Logo, eingebettete Ressource, Desktop-Datei (`.desktop.in` englisch, `.desktop` mit `[de]`) |
| `szenarien/` | Eingabeszenarien und Testdaten der isolierten GUI-Läufe |
| `werkzeug/` | isolierte GUI-Prüfung (`isoliert.sh`, `qe-eingabe.c`), GUI-Läufe und Auswertungen je Meilenstein, `i18n.py` (po-Pflege), `bauen-release.sh` |
| `docs/` | Schnittstellen und Verhalten je Bereich (Doc-Map in Arbeitsregeln) |

## Stand

| | |
|---|---|
| Version | 0.1.3 (Tag `einstellungen/0.1.3`) |
| Letzte Prüfung | 21.10.2026: Unit-Tests, fmt, clippy; isolierte GUI-Läufe deutsch und englisch in 1920×1080 und 1024×768, hell und dunkel (beim Start und beim Wechsel); Regression gegen 0.1.1 (Zahlen im Bericht der Entwicklungs-VM) |
| Bekannte Lücken | Panels nur Anzeige; das Panel hat noch keinen Belegungsanzeiger; Sprache und Formate wirken erst ab der nächsten Anmeldung |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, Rust 1.93 |
| Nicht getestet | andere Distributionen, HiDPI, Mehrmonitor und Tastaturumschaltung an echter Hardware |

## Bauen

Voraussetzungen (Ubuntu 26.04):

```bash
sudo apt install build-essential pkg-config rustc cargo libgtk-4-dev libwayland-dev gettext
```

```bash
cargo build --release --locked
```

Das Ergebnis liegt unter `target/release/querencia-einstellungen`. Die
Kataloge entstehen aus `po/` (`msgfmt` oder `werkzeug/i18n.py mo <ziel>`) und
liegen neben der Binary unter `share/locale/<sprache>/LC_MESSAGES/` oder im
Präfix (`/usr/share/locale`); Einzelheiten in
[docs/sprache-und-uebersetzung.md](docs/sprache-und-uebersetzung.md).

## Prüfen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 werkzeug/i18n.py pruefen
```

Die **GUI-Prüfung** läuft in einer abgeschotteten Sitzung (eigener labwc ohne
Bildschirm, eigenes Profil und `XDG_RUNTIME_DIR`), niemals in der laufenden
Desktopsitzung: `werkzeug/isoliert.sh`, die Läufe je Meilenstein in
`werkzeug/*-gui-lauf.sh`.

## Vorbild und Abgrenzung

Vorbild für Bedienung und Aussehen ist **das Kontrollzentrum von GNOME 2.30
(gnome-control-center, „Erscheinungsbild“, gnome-display-properties,
gnome-keyboard-properties)**. Querencia übernimmt davon die Bedienideen
(Gliederung, Bezeichnungen, Anordnung), **keinen Quellcode, keine Texte,
keine Grafiken**. Das Projekt ist mit GNOME und dessen Autoren weder
verbunden noch von ihnen unterstützt.

Abweichungen vom Vorbild und Dinge, die bewusst fehlen:

- GTK 4 statt GTK 2, Fensterrahmen vom Compositor (labwc), kein libadwaita.
- Einstellungen für labwc (Tastatur, Maus, Fensterdesign, Arbeitsflächen)
  statt GConf/Metacity; Sprache und Formate über die Sitzungsumgebung statt
  über GDM.

Fremdbestandteile: Rust-Abhängigkeiten (gtk4-rs, gettext-rs, roxmltree,
memchr, wayland-client, wayland-protocols-wlr und Umfeld) unter MIT, MIT OR
Apache-2.0 und verwandten freien Lizenzen; Liste mit `cargo metadata
--locked`. Dynamisch gelinkt: GTK 4, libwayland, gettext der glibc. In
`werkzeug/` liegt eine gekürzte Fassung des wlroots-Protokolls
`virtual-keyboard-unstable-v1.xml` mit eigenem Vermerk (MIT, siehe
Dateikopf).

## Lizenz

Programm: GPL-3.0-or-later, siehe [LICENSE](LICENSE). Copyright 2026
Querencia Desktop contributors; Urheberhinweis in
[COPYRIGHT](COPYRIGHT). Das Querencia-Logo in
`crates/querencia-einstellungen/data/logo/` ist unverändert übernommen.
Name und Logo „Querencia“ sind keine freien Inhalte.

