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

## [0.1.7] – 2026-10-26

### Hinzugefügt
- Oberfläche auf Italienisch, Spanisch und Französisch (`po/it.po`, `po/es.po`,
  `po/fr.po`, je 448 Texte) samt Desktop-Datei. Wortwahl nach den
  GNOME-Einstellungen der jeweiligen Sprache und der gemeinsamen
  Querencia-Begriffstabelle; Texte so gewählt, dass das Fenster bei 1024 × 768
  ganz auf den Bildschirm passt. Die Seite „Sprache und Region“ bietet die neuen
  Sprachen an, sobald die passende Locale installiert ist.

## [0.1.6] – 2026-10-22

### Hinzugefügt
- Bildschirm: Die Einstellungen werden in `~/.config/querencia/bildschirm.toml`
  gespeichert, je Monitor mit Name, Hersteller, Modell und Seriennummer.
  Eine `bildschirm.ini` älterer Fassungen wird weiter gelesen, bis das erste
  „Beibehalten“ die neue Datei anlegt; sie bleibt unverändert liegen.
- Bildschirm: Meldet ein Monitor keine Daten (EDID), erklärt ein Hinweis,
  warum nur Grundauflösungen wie 1024 × 768 zur Wahl stehen und dass Kabel
  ab- und wieder anstecken meist hilft.
- Bildschirm: Fußzeile „Quelle“ wie auf den anderen Seiten (Compositor,
  Datei, Befehl für die Anmeldung); der Knopf „Als Hauptmonitor verwenden“
  erklärt im Tooltip, was das unter labwc bedeutet.
- labwc-Seiten: Gibt es keine Querencia-Vorgabe für labwc, sagen die Seiten
  das und zeigen labwcs eingebaute Werte.
- Erscheinungsbild: „Dunkle Darstellung bevorzugen“ stellt beim Übernehmen
  auch den Fensterrahmen um (Querencia-Klassisch ↔ Querencia-Klassisch-Dunkel),
  wenn gerade einer der beiden eingestellt ist; ein anderer Rahmen bleibt.
  Fehlt der dunkle Rahmen, bleiben die Rahmen hell und die Seite sagt es.
  Das Fensterdesign nennt bei dunkler Darstellung den Zusammenhang.

### Geändert
- Die Rückfrage nach „Anwenden“ heißt „Einstellungen beibehalten?“, der Knopf
  „Beibehalten“.
- Die Vorgabe der labwc-Konfiguration wird jetzt auch neben dem Programm
  (`share/querencia/labwc` der Ablage bzw. unter dem Installationspräfix),
  unter `/etc/querencia/labwc` und unter `/usr/share/querencia/labwc`
  gesucht (vorher nur Checkout, Starter-Stand und `/etc`).
- Bildschirm: Statuszeile und Skalierungshinweis ohne Fachbegriffe der
  Entwicklung.
- Die Seiten mit labwc-Einstellungen lesen ihren Stand neu, wenn man sie
  öffnet und sich inzwischen etwas geändert hat (z. B. der Fensterrahmen nach
  dem Umschalten im Erscheinungsbild).

### Behoben
- Ändern und Zurückstellen der Namen der Arbeitsflächen ließ `rc.xml` nicht
  byte-gleich zurück, wenn die Datei an anderer Stelle eigene Werte hatte
  (`<desktops … />` blieb aufgeklappt). Jetzt bekommt jeder veränderte
  Behälter, der danach der Vorgabe gleicht, deren Form zurück.
- Bildschirm: Wurde ein Monitor verschoben, legte labwc einen nie
  verschobenen Monitor manchmal an eine andere Stelle als gewählt (z. B.
  beim Untereinanderlegen). Die Einstellungen setzen die Anordnung dann ein
  zweites Mal; danach liegt jeder Monitor wie gewählt.

### Bekannte Einschränkungen
- Gespeicherte Bildschirmeinstellungen gelten nach der Anmeldung erst, wenn
  der Autostart der Sitzung `querencia-einstellungen --bildschirm-anwenden`
  aufruft (folgt in einer eigenen Änderung der Sitzung).

## [0.1.5] – 2026-10-22

### Hinzugefügt
- „Informationen über Querencia“ zeigt die Ausgabe von Querencia Desktop
  (z. B. „Querencia Desktop Alpha 2.1“) aus `/usr/share/querencia/release`,
  sonst aus `/usr/share/querencia/version`; ohne beide Dateien steht nur
  „Querencia Desktop“ da. Dazu die Website, die Lizenz (GPL-3.0-or-later)
  und die Knöpfe „Online-Hilfe“ und „Fehler melden“, die den Browser öffnen.
  Die Adressen kommen aus derselben Datei, sonst gelten
  querencialinux.org, die Hilfe in der Sprache der Oberfläche und die
  Fehlerseite des Projekts auf GitHub.
- Neue Seite „Netzwerk“: erklärt, dass Verbindungen über das Netzwerksymbol
  im Panel eingerichtet werden. `querencia-einstellungen --netzwerk` und die
  Aktion „Netzwerkeinstellungen“ der Desktop-Datei öffnen das
  Kontrollzentrum auf dieser Seite; damit hat der Eintrag
  „Netzwerkeinstellungen …“ im Panel ein Ziel.
- `querencia-einstellungen --seite <name>` öffnet das Kontrollzentrum auf
  einer beliebigen Seite (z. B. `sprache`, `maus`).

### Geändert
- Die Zeile „Quelle: …“ am Ende der Seiten (wo die Werte gespeichert sind)
  ist auf allen Seiten gleich: klein, grau, letzte Zeile über den Knöpfen.
- „Informationen über Querencia“ nennt keinen Pfad der Entwicklungsumgebung
  mehr; die falsche Angabe zum Panel-Dialog ist entfernt.
- Die Quellzeilen von „Arbeitsflächen“ und „Standardanwendungen“ kommen
  ohne Fachbegriffe der Entwicklung aus.

## [0.1.4] – 2026-10-21

### Hinzugefügt
- Die Galerie „Eingebaute Bilder“ kennt die 16 Hintergründe von
  Hintergründe 0.2: Tiefblau, Blaugrün, Blauschein, Glut, Hügel,
  Küste, Dünen, See, Wüste, Berge, Strand, Glas, Tiefblau mit Logo, mit
  Wortmarke und mit Wasserzeichen, Bernstein mit Logo. Reihenfolge: Tiefblau
  zuerst, dann die übrigen neuen Bilder, danach die Bilder von 0.1. Die
  4:3-Fassung wird wie bisher nach dem Seitenverhältnis des Bildschirms
  gewählt. Lange Namen stehen in zwei Zeilen.

### Geändert
- Standardbild ist „Querencia Tiefblau“ (erste Kachel). Es bleibt beim
  leeren `File=` in `desktop.ini` („Standard wiederherstellen“, „Eingebautes
  Bild“): das zeigt ab Desktop 0.2.1 Tiefblau. Ist Tiefblau nicht
  installiert, gilt das leere `File=` wie bisher als „Querencia Blau“.
  „Querencia Blau“ wird bei installiertem Tiefblau mit vollem Pfad
  geschrieben.
- Nicht installierte Bilder erscheinen weiterhin nicht in der Galerie; ohne
  das Datenpaket von Hintergründe 0.2 sieht sie aus wie in 0.1.3.

## [0.1.3] – 2026-10-21

### Geändert
- Helle Darstellung, Kontrast nach WCAG AA: Das Kürzel hinter dem
  Namen der Tastaturbelegung ist mit 75 % statt 60 % Deckkraft gedimmt wie in
  der dunklen Darstellung (4,8:1 statt 3,3:1). Die gewählte Ausgabe im
  Bereich „Bildschirm“ ist dunkler blau (`#2a6cc4` statt `#3685e3`), die
  weiße Schrift darauf hat 5,2:1 statt 3,7:1. Alles andere bleibt, wie es
  war; die dunkle Darstellung ist unverändert.

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
- Standardanwendungen: Der Rückgriff auf die alte Terminal-ID
  `org.example.Querencia.Terminal.desktop` entfällt;
  angezeigt wird `org.querencia.Terminal.desktop`.
- Werkzeuge ohne feste Pfade der Entwicklungsumgebung (`$HOME`, `XDG_*`,
  Parameter); Integrationsskripte früherer Fassungen liegen unter
  `integration/werkzeug/`.

### Behoben
- Unter „Dunkle Darstellung bevorzugen“ startet das Fenster gleich dunkel
  (GTK 4.22 lud beim Start nur die helle Fassung des Themas).
- Die Anordnungsfläche auf der Seite „Bildschirm“ ist bei dunklem Thema dunkel
  statt hellgrau; hell unverändert.
- Dunkle Fassung: Das Kürzel der Tastaturbelegung („(de)“) ist lesbar
  (75 % statt 60 % Deckkraft, AA), der Rand der Hintergrund-Vorschau hebt sich
  vom Fenstergrund ab; hell unverändert.

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

[Unveröffentlicht]: https://github.com/querencia-desktop/querencia-einstellungen/compare/einstellungen/0.1.3...HEAD
[0.1.3]: https://github.com/querencia-desktop/querencia-einstellungen/releases/tag/einstellungen/0.1.3
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
