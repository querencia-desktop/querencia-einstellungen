# Bereich „Bildschirm“ (ab 0.0.2, Datei und Grenzen ab 0.1.6)

Auflösung, Bildwiederholrate, Drehung, Skalierung und Anordnung mehrerer
Monitore, im Stil der GNOME-2-Monitoreinstellungen. Seite im
Kontrollzentrum (`querencia-einstellungen --seite bildschirm`) und eigenes
Fenster (`querencia-einstellungen --bildschirm`).

## Bedienung

| Element | Wirkung |
|---|---|
| Monitor | Wahl des Ausgangs (Name – Hersteller Modell; „Hauptmonitor“, „(aus)“) |
| Eingeschaltet | Ein/Aus je Ausgang; der letzte eingeschaltete lässt sich nicht ausschalten |
| Auflösung | Modi des Monitors, größte zuerst, bevorzugter markiert; Ausgänge ohne Modusliste (virtuell) bekommen freie Größen 1024 × 768 … 2560 × 1440 |
| Wiederholrate | die Raten der gewählten Größe (nur aktiv, wenn es mehr als eine gibt) |
| Drehung | Normal, 90°, 180°, 270° |
| Skalierung | 1, 1,25, 1,5, 2 (gebrochene mit Hinweis) |
| Als Hauptmonitor verwenden | legt den Monitor nach links oben (0, 0), die anderen daneben; `primaer = true` in der Datei |
| Anordnung | Rechtecke ziehen; nach dem Loslassen rastet der Monitor an Kanten der Nachbarn ein (60 px), Überlappungen werden aufgelöst, die kleinste Ecke liegt bei 0, 0 |
| Anwenden | erst `test`, dann `apply` über `zwlr_output_manager_v1` (wayland-client, kein Aufruf von `wlr-randr`); danach Rückfrage „Einstellungen beibehalten?“ mit 15-s-Countdown |
| Beibehalten | speichert nach `~/.config/querencia/bildschirm.toml` (atomar) |
| Vorherige Einstellungen wiederherstellen, Schließen des Dialogs, Ablauf | setzen den Stand vor „Anwenden“ wieder, nichts wird gespeichert |
| Zurücksetzen | liest den Compositor neu (verwirft nicht angewendete Änderungen) |

Die Fußzeile nennt Quelle und Datei: im Kontrollzentrum als letzte Zeile
des Seiteninhalts, im eigenen Fenster fest über der Statuszeile (dort rollt
der Inhalt).

## Datei `~/.config/querencia/bildschirm.toml` (ab 0.1.6)

```toml
# Querencia Einstellungen – Bildschirm. …
version = 1

[[ausgabe]]
name = "HDMI-A-1"
hersteller = "Ancor Communications Inc"
modell = "ASUS VS247"
seriennummer = "G7LMTF159704"
aktiv = true
breite = 1920
hoehe = 1080
wiederholrate_mhz = 60000
x = 0
y = 0
drehung = 0
skalierung = 1.0
primaer = true
```

- Eine Tabelle `[[ausgabe]]` je Ausgang. Zuordnung beim Anwenden: zuerst
  Hersteller|Modell|Seriennummer (wenn nicht leer), sonst der Name.
  Ausgänge ohne Eintrag bleiben, wie sie sind.
- Gelesen wird mit dem `toml`-Crate (`toml::Table`, ohne serde-derive),
  geschrieben in fester Reihenfolge mit Kommentarkopf. Unbekannte Schlüssel
  werden gemeldet, ein eingeschalteter Eintrag ohne `breite`/`hoehe` wird
  übergangen, ein Text, der kein TOML ist, ergibt keinen Eintrag.
- **Alte Datei:** Bis 0.1.5 hieß sie `bildschirm.ini` (Gruppen
  `[Ausgabe <Name>]`, `Geraet=`, `Modus=1920x1080@60000`, `Haupt=`). 0.1.6
  liest sie, solange es keine `bildschirm.toml` gibt, schreibt und löscht
  sie aber nie. Das erste „Beibehalten“ legt `bildschirm.toml` an; danach
  gilt nur noch diese.

## `querencia-einstellungen --bildschirm-anwenden`

Für die Autostart-Datei der Sitzung (ohne Fenster): liest
`bildschirm.toml`, sonst die alte `bildschirm.ini`, ordnet die Einträge
den vorhandenen Ausgängen zu, testet und setzt. Exit 0 = gesetzt oder
nichts zu tun, 1 = Fehler (z. B. der Compositor lehnt den Test ab, die
Datei würde alle Ausgänge ausschalten), 2 = keine Datei. Meldungen auf
stderr. Alle Aufrufe haben eine Zeitgrenze von 5 s.

Die Zeile in `session/querencia/labwc/autostart` des Runtime-Repos gehört
nicht zu diesem Repo; sie wird nach der Welle Alpha 2.2 eingebunden. Bis
dahin gilt eine gespeicherte Konfiguration erst, wenn jemand den Befehl
ausführt oder in der App „Beibehalten“ drückt.

`--bildschirm-zeigen` druckt den Ist-Zustand als Text (Prüfwerkzeuge).

## Grenzen von labwc 0.9.8

- **Keine eigene Speicherung.** labwc übernimmt beim Start nichts aus einer
  früheren wlr-output-management-Konfiguration (`labwc-config(5)`: nur
  `autoEnableOutputs`, `reuseOutputMode`). Deshalb die Datei und
  `--bildschirm-anwenden`.
- **Kein Hauptmonitor.** labwc kennt keinen primären Ausgang. „Als
  Hauptmonitor verwenden“ legt ihn nach 0, 0 und vermerkt `primaer = true`
  für Panel und Desktop, die es noch nicht lesen.
- **Automatisch platzierte Ausgänge (0.1.6).** labwc verschiebt einen
  Ausgang nur, wenn die verlangte Position von der jetzigen abweicht
  (`output_config_apply`). Ein Ausgang, der nie verschoben wurde, bleibt in
  wlroots „automatisch platziert“ und wird neu gelegt, sobald ein anderer
  verschoben wird (rechts neben den äußersten). Gemessen: HEADLESS-1 unter
  HEADLESS-2 gezogen, labwc legte HEADLESS-2 danach auf 1920, 720 statt
  0, 0. Die App liest nach dem Setzen die Lage nach und setzt bei
  Abweichung dieselbe Wahl ein zweites Mal (`Verbindung::setzen`); danach
  liegt jeder Ausgang fest. Bleibt etwas anders, steht es im Protokoll.
- **Keine EDID bei durchgereichter Grafikkarte (QD-027).** Auf der
  Entwicklungs-VM mit durchgereichter RTX 3060 meldet der Monitor nach dem
  Start manchmal keine EDID. labwc bietet dann nur Grundmodi wie
  1024 × 768 an, ohne Hersteller und Modell. Die Seite zeigt dafür einen
  Hinweis (echter Anschluss ohne Hersteller und Modell); Abhilfe ist das
  Kabel ab- und wieder anstecken, kein Neustart. `--bildschirm-anwenden`
  testet zuerst und setzt nichts, wenn der gespeicherte Modus fehlt (Exit
  1); die Zuordnung fällt dann auf den Namen (`HDMI-A-1`) zurück.
- **Bildwiederholraten** lassen sich isoliert nicht prüfen: virtuelle
  Ausgänge melden nur ihren aktuellen Modus (Modell und Unit-Tests decken
  die Auswahl ab).
- **Abbruch während des Countdowns:** Endet das Programm, bevor die Frage
  beantwortet ist (Absturz, Abmelden), bleibt der neue Zustand bis zur
  nächsten Anmeldung aktiv; gespeichert ist er nicht.

## Prüfschalter

`QE_BILDSCHIRM_OHNE_EDID=<name>[,<name>…]` lässt die genannten Ausgänge als
„ohne EDID“ gelten (Foto des Hinweises in den isolierten Läufen).

## Prüfung 0.1.6

`werkzeug/016-gui-lauf.sh` (isolierter headless labwc mit zwei virtuellen
Ausgängen, `WLR_BACKENDS=headless`, `WLR_HEADLESS_OUTPUTS=2`), Auswertung
`werkzeug/auswerten-016.py`: Countdown-Rückfall, Beibehalten, Skalierung
1,25 und 2, Hauptmonitor, Ziehen, `--bildschirm-anwenden` mit TOML, mit
alter INI und ohne Datei, EDID-Hinweis, Fotos de/en, 1024 × 768 und
1920 × 1080, hell und dunkel. Die Ausgänge der laufenden Sitzung werden
dabei nie berührt.
