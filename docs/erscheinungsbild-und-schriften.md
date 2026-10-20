# Erscheinungsbild, Schriften, Hintergrundbild und Panels (0.0.9, G2-AJ)

Mit 0.0.9 zeigt keine Seite des Kontrollzentrums mehr einen Kasten
„Noch nicht verfügbar“. Dieses Dokument beschreibt, was die vier
überarbeiteten Seiten lesen, schreiben und wann es wirkt.

## Gemeinsames Gerüst

Erscheinungsbild und Schriften benutzen das Seitengerüst der labwc-Bereiche
(`src/labwc_seite.rs`, G2-AG): Ändern → „Übernehmen“, „Standard
wiederherstellen“, „Neu laden“, Konfliktprüfung vor dem Schreiben. Neu ist
eine zweite Quelle neben rc.xml und Umgebung:

- `Bereich::GSETTINGS` nennt die Schlüssel aus `org.gnome.desktop.interface`,
  die der Bereich liest; das Gerüst füllt sie in `Stand::gs`.
- `Bereich::gs_aenderungen` liefert die geänderten Werte; geschrieben wird
  nach rc.xml und Umgebung (`src/gsettings.rs`, mit `Settings::sync`).
- „Standard wiederherstellen“ zeigt Querencias Standard
  (`gsettings::STANDARD`: Querencia-Klassisch, Symbolthema Querencia,
  DMZ-White 24, DejaVu Sans/Mono 10), nicht den Standard des Schemas.
- `Bereich::hinweis` ersetzt den labwc-Wirkungstext oben auf der Seite.

GTK 4.22 liest diese Werte über das Settings-Portal. Eine Änderung wirkt
deshalb sofort in allen laufenden GTK-4-Programmen, auch in Panel und
Desktop.

## Erscheinungsbild (`src/erscheinungsbild.rs`)

| Wahl | angeboten | geschrieben |
|---|---|---|
| Symbole | Ordner mit `index.theme` und `Directories=` in `~/.local/share/icons`, `~/.icons`, `/usr/share/icons`; ohne `Hidden=true` (Ausnahme Adwaita), ohne `default`/`hicolor`; Anzeigename aus `Name=` | `icon-theme` |
| Steuerelemente | Themen mit `gtk-4.0/gtk.css` in `~/.local/share/themes`, `~/.themes`, `/usr/share/themes` sowie das eingebaute Thema („GTK-Standard“, Wert `Adwaita`) | `gtk-theme` |
| Dunkle Darstellung | Umschalter | `color-scheme` `prefer-dark` bzw. `default` (ein gelesenes `prefer-light` bleibt, solange der Umschalter nicht bewegt wird) |
| Mauszeiger | Ordner mit `cursors/` (wie Bereich „Maus“), Größe 24/32/48 | `cursor-theme`, `cursor-size`; für labwc `XCURSOR_THEME`/`XCURSOR_SIZE` in `environment.d/zz-querencia-einstellungen.env` (gleich der Vorgabe → Zeile entfernt) |

Ein Wert, der nicht installiert ist, erscheint als „… (nicht installiert)“
und bleibt wählbar. Die Vorschau zeigt fünf Symbole des gewählten Themas
über ein eigenes `GtkIconTheme`, das Fenster behält sein Thema.

Wirkung:

- Symbole, Thema, Farbschema und der Zeiger in GTK-Programmen: sofort.
- Panel und Desktop (seit G2-AD) folgen dem Symbolthema sofort; bei
  „Adwaita“ (wie bei ungesetztem Wert) zeigen sie Querencia.
- Zeiger über Fensterrahmen und freier Arbeitsfläche: labwc liest die
  Umgebung beim `--reconfigure` neu (`src/server.c:115`
  `session_environment_init`, dann `cursor_load`), also sofort, wenn labwc
  die Benutzerkopie liest, sonst ab der nächsten Anmeldung.
- Andere Programme (X11, Qt) sehen `XCURSOR_*` ab der nächsten Anmeldung.
- `color-scheme` erreicht GTK (`gtk-interface-color-scheme`), schaltet aber
  Programme ohne libadwaita nicht um; das eingebaute Thema wechselt dadurch
  nicht ins Dunkle. Die Seite sagt das.

Der Bereich „Maus“ schreibt bei einer Zeigeränderung zusätzlich
`cursor-theme`/`cursor-size` (Standard → DMZ-White), damit beide Seiten
dieselben Werte setzen.

## Schriften (`src/schriften.rs`)

- Anwendungen, Dokumente, Dicktengleich: `FontDialogButton` →
  `font-name`, `document-font-name`, `monospace-font-name` (Pango-Text,
  z. B. „DejaVu Serif 12“). Wird dieselbe Schrift wieder gewählt, bleibt der
  gelesene Text stehen (Pango schreibt manche Beschreibungen anders zurück).
- Fenstertitel: `<theme><font place="ActiveWindow">` und
  `"InactiveWindow"`: `name`, `size`, `weight` (`bold`/`normal`), über
  `labwc_xml::setzen` auf dem aktuellen Dateistand; Kursiv wird nicht
  geschrieben. `weight` steht deshalb in `labwc_xml::VERWALTET`
  (Dreiwegevergleich mit neuen Vorgaben). Derselbe Wert (ohne Fett) steht im
  Bereich „Fensterdesign“.

## Hintergrundbild (`src/hintergrund_seite.rs`)

Die Galerie der eingebauten Bilder (G2-AF, `hintergrund::galerie_bauen`)
steht direkt auf der Seite: Kachel anklicken → Vorschau (`vorschau.rs`,
Fassung 4:3/16:9 nach Bildschirm) → „Übernehmen“. Geschrieben wird nur
`[Background]` (`File=` der Fassung, `Type=image`), Darstellung und Farbe
bleiben; Konfliktprüfung, Sicherung und Fähigkeiten des laufenden Desktops
wie im Dialog. „Hintergrund …“ öffnet weiter den Dialog für eigene Bilder,
Farbe und Darstellung. Ein `GFileMonitor` zeigt Änderungen an
`desktop.ini` (Dialog, Desktop), solange keine Auswahl offen ist.

## Panels (`src/panels.rs`)

Die Seite schreibt nichts. Das laufende Panel hat keine Schnittstelle zum
Neuladen; eine Änderung an `panel.ini` von außen würde erst beim nächsten
Start sichtbar und bei der nächsten Änderung im Panel verweigert (das Panel
schreibt nie über eine geänderte Datei). Die Seite erklärt die Bedienung im
Panel (Rechtsklick: „Zum Panel hinzufügen …“, Verschieben, „Aus dem Panel
entfernen“; mittlere Taste ziehen) und zeigt den gespeicherten Aufbau mit
den Namen des Panels (Starter mit dem Namen der Anwendung). Die Anzeige
folgt Änderungen an der Datei. Lage und Höhe haben im Panel noch keinen
Menüpunkt.

## Bildschirm

Unverändert seit G2-G: Werte und Setzen über `zwlr_output_manager_v1`,
dauerhaft über `~/.config/querencia/bildschirm.ini` und
`querencia-einstellungen --bildschirm-anwenden` im Autostart. labwc 0.9.8
hat keine `<output>`-Regeln in rc.xml (Ausgaben stellen laut
labwc-config(5) externe Werkzeuge wie kanshi oder wlr-randr ein).

## Zeichenfehler

Siehe [`zeichenfehler-gtk.md`](zeichenfehler-gtk.md).

## Prüfung

`werkzeug/g2aj-gui-lauf.sh` (Szenarien aus `werkzeug/g2aj-szenarien.py`,
Auswertung `werkzeug/auswerten-g2aj.py`). Neue Schalter in
`werkzeug/isoliert.sh`:

| Schalter | Wirkung |
|---|---|
| `QE_GTK_THEMA="<GTK>[:<Symbole>]"` | Thema und Symbole im `settings.ini` des Profils |
| `QE_GSETTINGS=<datei>` | Zeilen `schlüssel wert` in den Profil-dconf, vor dem Start von Desktop und Panel; `gsettings-start.txt`/`-ende.txt` im Laufordner |
| `QE_PORTAL=settings` | Settings-Portal-Ersatz (`werkzeug/g2aj-settings-portal.py`) statt `GDK_DEBUG=no-portals`: GTK-Programme folgen gsettings sofort |
| `QE_WLR_RENDERER=gles2` | headless labwc zeichnet auf der Grafikkarte (dmabuf, GTK nimmt Vulkan) wie die echte Sitzung |
| `QE_GSK_RENDERER` | `GSK_RENDERER` der App |
