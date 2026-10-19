# Fenstersymbol (G2-AA, 0.0.6)

Ab 0.0.6 schickt jedes Fenster von Querencia Einstellungen – Kontrollzentrum,
die Einzeldialoge (`--hintergrund`, `--bildschirm`, `--arbeitsflaeche`,
`--standardanwendungen`) und die Unterdialoge ohne eigene Anwendung – labwc
ein eigenes Fenstersymbol (Protokoll xdg-toplevel-icon): das Zahnrad von G2-V.

## Aufbau

| Teil | Ort |
|---|---|
| Zahnrad (Quelle, byte-gleich mit `~/projects/querencia-symbole/quellen/`) | `crates/querencia-einstellungen/data/icons/hicolor/scalable/apps/org.querencia.Einstellungen.svg` |
| Fenstersymbol (dasselbe SVG, eigener Name) | `crates/querencia-einstellungen/data/resource/icons/scalable/apps/org.querencia.Einstellungen-fenster.svg` |
| GResource (eingecheckt, neu erzeugen mit `werkzeug/ressourcen.sh`) | `crates/querencia-einstellungen/data/resource/einstellungen.gresource` |
| Code | `src/main.rs`, `fenstersymbol_einrichten()` beim `startup` der Anwendung |

`fenstersymbol_einrichten()` registriert die Ressource, ergänzt das
Symbolthema des Displays um `/org/querencia/Einstellungen/icons` und setzt
`gtk::Window::set_default_icon_name("org.querencia.Einstellungen-fenster")`.
Die Vorgabe gilt für alle Fenster, auch für Dialoge ohne `application`
(App-ID `querencia-einstellungen`). Ein Unit-Test prüft, dass die
eingecheckte Ressource das SVG byte-gleich enthält.

## Regeln

- **Nur SVG, eigener Name `…-fenster`.** Mit den PNG-Symbolen der App-ID
  gibt GTK 4.22 beim Abbau eines Fensters einen Symbolpuffer doppelt frei
  (Editor, G2-V). Nie ein Symbol in `hicolor/48x48`.
- **Starter bleibt `Icon=org.querencia.Einstellungen`** (G2-V-Skript 0.2
  prüft diese Zeile). Menü und Fensterliste lesen weiter den Starter.

## Was labwc 0.9.8 in der Titelleiste zeigt

labwc wählt das Titelleistensymbol in `src/scaled-buffer/scaled-icon-buffer.c`:

1. **Ohne Fensterregel** zuerst das Symbol des Starters zur App-ID
   (`loaded icon by app_id`), erst wenn das fehlt, das vom Programm
   geschickte (`loaded icon from client buffer`).
2. **Mit `<windowRule identifier="…" iconPriority="client" />`** zuerst das
   vom Programm geschickte.

Starter und Symbolthema liest labwc beim Start und bei `labwc --reconfigure`
(`src/server.c`, `desktop_entry_finish/init`), nicht bei Änderungen danach.

Folgen für Querencia Einstellungen:

- Hauptfenster (App-ID `org.querencia.Einstellungen`): Es gilt der Starter in
  dem Stand, den labwc zuletzt eingelesen hat. Das eingebettete Symbol wirkt
  dort nur mit der Fensterregel `iconPriority="client"`.
- Unterdialoge (App-ID `querencia-einstellungen`): Zu dieser App-ID gibt es
  keinen Starter, deshalb gilt immer das geschickte Symbol. 0.0.5 hatte dort
  das Symbol der App-ID aus `~/.local/share/icons` (PNG). 0.0.6 schickt das
  SVG aus der Ressource.

Belege: `nachweis/g2aa/fenstersymbol-g2aa.tsv` (Lauf `werkzeug/g2aa-gui-lauf.sh`).
