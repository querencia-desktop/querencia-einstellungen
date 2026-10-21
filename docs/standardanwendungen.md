# Bereich „Standardanwendungen“ (QD-046 ①, G2-Z)

Stand 19.10.2026, Querencia Einstellungen 0.0.5. Quelle der Tabelle ist
`crates/querencia-einstellungen/src/standardanwendungen_modell.rs`
(`KATEGORIEN`). Ändert sich dort etwas, wird dieses Dokument gleichzeitig
angepasst.

## Wofür

Der Bereich entspricht den „Bevorzugten Anwendungen“ von GNOME 2. Er zeigt für
jede Kategorie die Anwendung, die GIO heute für den Haupttyp liefert. Über
„Ändern …“ wählt man eine andere aus den installierten Anwendungen. Querencia
Files, Desktop, Panel und alle GTK-Programme öffnen über GIO, deshalb wirkt
eine Änderung sofort und ohne Neustart.

Aufruf: im Kontrollzentrum (Seitenleiste „Standardanwendungen“) oder einzeln
mit `querencia-einstellungen --standardanwendungen`. Den Ist-Stand als Text
gibt `querencia-einstellungen --standardanwendungen-zeigen` aus; dieser Aufruf
schreibt nichts.

## Kategorien

| Kategorie | Haupttyp (erster) und weitere Typen | Vorschlag |
|---|---|---|
| Webbrowser | `x-scheme-handler/http`, `x-scheme-handler/https`, `text/html`, `application/xhtml+xml` | – |
| E-Mail | `x-scheme-handler/mailto` | – |
| Texteditor | `text/plain`, `text/markdown`, `text/x-markdown` | `org.querencia.Editor.desktop` |
| Terminal | – (nur Anzeige, siehe unten) | `org.querencia.Terminal.desktop` |
| Bilder | `image/png`, `image/jpeg`, `image/gif`, `image/webp`, `image/bmp`, `image/svg+xml`, `image/tiff` | `org.querencia.Bilder.desktop` |
| Musik | `audio/mpeg`, `audio/ogg`, `audio/x-vorbis+ogg`, `audio/flac`, `audio/x-wav`, `audio/mp4` | – |
| Video | `video/mp4`, `video/webm`, `video/x-matroska`, `video/ogg`, `video/mpeg` | – |
| Dateimanager | `inode/directory` | `io.github.lgse.Strata.desktop` (Querencia Files) |
| PDF-Dokumente | `application/pdf` | – |
| Archive | `application/zip`, `application/x-tar`, `application/x-compressed-tar`, `application/gzip`, `application/x-xz-compressed-tar`, `application/x-7z-compressed` | – |

## Lesen

- **Zuständig** ist `g_app_info_get_default_for_type(Haupttyp)`. GIO wertet
  dabei die Benutzerdatei, die desktop-spezifische Liste und die
  Systemstandards aus.
- **Zur Auswahl** stehen alle Anwendungen aus
  `g_app_info_get_all_for_type(Haupttyp)`, die im Menü erscheinen
  (`should_show`), dazu die aktuelle. Sortiert wird nach Name, Doppelte
  entfallen.
- GIO nimmt die registrierten Anwendungen aus `mimeinfo.cache`
  (`update-desktop-database`), nicht direkt aus den Startern. Ein Starter ohne
  Eintrag im Cache erscheint deshalb nicht.
- **Ohne passende Anwendung** steht in der Zeile „Keine passende Anwendung
  installiert“, und es gibt keinen Knopf. Am 19.10. betrifft das Musik, Video
  und Archive.

## Schreiben

- Es wird nur über `g_app_info_set_as_default_for_type` geschrieben. Ziel ist
  `$XDG_CONFIG_HOME/mimeapps.list`, also `~/.config/mimeapps.list`. Nie wird
  systemweit geschrieben, nie in eine andere Datei.
- Der **Haupttyp** wird immer gesetzt. Die **weiteren Typen** werden nur
  gesetzt, wenn die gewählte Anwendung sie in `MimeType=` nennt (Regel wie im
  GNOME-Kontrollzentrum). Beispiel: Wählt man Vim, setzt die Seite nur
  `text/plain`; wählt man den Querencia Editor, setzt sie alle drei Texttypen.
- GIO führt außerdem `[Added Associations]` nach (die gewählte Anwendung steht
  dort zuerst). Andere Einträge und Gruppen der Datei bleiben erhalten.
- Ist die Wahl schon die aktuelle Anwendung, wird nichts geschrieben.
  Dasselbe gilt für „Abbrechen“ und Escape.
- **Vorschläge** sind in der Auswahl nur mit „(Vorschlag)“ markiert.
  Vorausgewählt ist immer die aktuelle Anwendung, nie der Vorschlag. Gesetzt
  wird nichts von selbst.
- Nach dem Schreiben fragt die Seite GIO erneut. Kommt eine andere Anwendung
  zurück, weil eine vorrangige Liste wie `~/.config/x-querencia-mimeapps.list`
  etwas anderes festlegt, meldet die Seite das statt eines Erfolgs.

## Terminal

Für Terminals gibt es keinen MIME-Typ und keinen Schreibweg über GIO. Panel
und Desktop starten Querencia Terminal fest (`TERMINAL_APP` im Runtime-Repo);
labwc nutzt `lab-sensible-terminal`. Angezeigt wird der Starter
`org.querencia.Terminal.desktop` (seit 20.10.2026, QD-066). Den Rückgriff
auf die alte ID älterer Installationen gibt es seit 0.1.2 nicht mehr; fehlt
der Starter, zeigt die Zeile ihn als nicht installiert. Die Zeile zeigt nur
an. Eine Auswahl ohne Wirkung wäre ein Knopf, der
nichts tut. Eine echte Auswahl braucht einen eigenen Baustein, etwa eine
gemeinsame Einstellung, die Panel und Desktop lesen. `xdg-terminals.list`
wäre eine weitere Benutzerdatei und gehört nicht zu diesem Bereich.

## Auswahl im Dialog statt Klappliste

Die Seite nimmt keine Klappliste, weil nach oben geklappte Listen unter labwc
0.9.8 keine Klicks bekommen (G2-G). Die untersten Zeilen der Seite lägen genau
dort. Der Auswahldialog ist modal und hat eine feste Größe von 460×380.
