# Änderungsprotokoll

Format nach [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), Versionen
nach [Semantic Versioning](https://semver.org/lang/de/). Solange die Hauptversion
0 ist, kann sich alles ändern. Jede Zeile nennt, was Nutzer merken; Unfertiges
steht als „unfertig“ da. Die Einträge bis 0.0.8 sind aus den Tag-Texten
zusammengefasst.

## [Unveröffentlicht]

### Bekannte Einschränkungen
- Die Seite „Panels“ zeigt den Aufbau nur an; Lage und Höhe der Panels sind
  noch fest.

## [0.1.2] – 2026-10-21

### Geändert
- Die Vorgabe der labwc-Konfiguration wird zur Laufzeit gefunden statt über
  einen festen Entwicklungspfad: `QE_LABWC_VORGABE`, sonst
  `$QUERENCIA_REPO/session/querencia/labwc`, sonst die Quelle, die der
  Sitzungsstarter in `~/.config/querencia/labwc/.vorgabe/stand` notiert hat,
  sonst `/etc/querencia/labwc` (Paket). `--labwc-zeigen` nennt die Herkunft.
- Hinweis unter „Dunkle Darstellung bevorzugen“: Hat das gewählte GTK-Thema
  eine dunkle Fassung, steht dort, dass auch die Querencia-Programme dunkel
  werden (außer Querencia Files); sonst wie bisher.
- Werkzeuge ohne feste Pfade der Entwicklungsumgebung (`$HOME`, `XDG_*`,
  Parameter); Integrationsskripte früherer Fassungen liegen unter
  `integration/werkzeug/`.

### Behoben
- Unter „Dunkle Darstellung bevorzugen“ startet das Fenster gleich dunkel
  (GTK 4.22 lud beim Start nur die helle Fassung des Themas, QD-081).
- Die Anordnungsfläche auf der Seite „Bildschirm“ ist bei dunklem Thema dunkel
  statt hellgrau; hell unverändert.

## [0.1.1] – 2026-10-21

### Behoben
- Ein auf den Vorgabewert zurückgestellter Wert lässt `rc.xml` so, wie die
  Vorgabe sie hat; der Sitzungsstarter zieht neue Vorgaben dann wieder nach.
- Eigene Sicherungen heißen `<datei>.einstellungen-vorher-<Zeit>` (eine je
  Programmlauf, höchstens fünf); Sicherungen des Sitzungsstarters bleiben
  unberührt.

### Hinzugefügt
- Hinweis im Erscheinungsbild: die Themen liegen in gsettings und gelten auch
  für eine GNOME-Sitzung auf demselben Rechner.

## [0.1.0] – 2026-10-20

### Hinzugefügt
- Oberfläche deutsch und englisch über GNU gettext (Kataloge `de`, `en`),
  zweisprachiger Menüeintrag.
- Bereich „Sprache und Region“: Sprache und Formate der Sitzung (wirken ab
  der nächsten Anmeldung).
- Tastatur: mehrere Belegungen mit Umschalttaste.

## [0.0.9] – 2026-10-20

### Geändert
- Erscheinungsbild und Schriften wirken wirklich (gsettings und
  labwc-Benutzerkopie); Galerie auf der Seite „Hintergrundbild“; die Seite
  „Panels“ ohne Baustellen-Hinweis.

### Behoben
- Zeichenfehler in Listen; Terminal-Starter in den Standardanwendungen.

## [0.0.8] – 2026-10-20

### Hinzugefügt
- Bereiche Tastatur, Maus, Fensterdesign und Arbeitsflächen; sie schreiben die
  labwc-Benutzerkopie unter `~/.config/querencia/labwc/` (mit Sicherung) und
  lösen `labwc --reconfigure` aus.

## [0.0.7] – 2026-10-20

### Hinzugefügt
- Galerie der eingebauten Hintergründe, Fassung nach Seitenverhältnis.

## [0.0.6] – 2026-10-20

### Hinzugefügt
- Eingebettetes Fenstersymbol.

## [0.0.5] – 2026-10-19

### Hinzugefügt
- Bereich Standardanwendungen.

## [0.0.4] – 2026-10-19

### Geändert
- Bereich heißt „Symbole der Arbeitsfläche“.

## [0.0.3] – 2026-10-19

### Hinzugefügt
- Symbole und Laufwerkstypen der Arbeitsfläche.

## [0.0.2] – 2026-10-19

Erster Stand in Git: Hintergrund (Bild, Farbe, Darstellung) und Bildschirm
(Auflösung und Anordnung über `wlr-output-management`).

[Unveröffentlicht]: https://github.com/querencia-desktop/querencia-einstellungen/compare/einstellungen/0.1.2...HEAD
[0.1.2]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.1.2
[0.1.1]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.1.1
[0.1.0]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.1.0
[0.0.9]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.0.9
[0.0.8]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.0.8
[0.0.7]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.0.7
[0.0.6]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.0.6
[0.0.5]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.0.5
[0.0.4]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.0.4
[0.0.3]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.0.3
[0.0.2]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.0.2
