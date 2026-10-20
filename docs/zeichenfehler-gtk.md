# Zeichenfehler: Textreste nach dem Seitenwechsel (G2-AJ)

## Bild

pottrauschens Bildschirmfotos vom 20.10.2026 (Seiten Hintergrundbild und
Panels): unter dem Seiteninhalt stehen blasse, halbe Textzeilen und Punkte
der zuvor gezeigten Seite.

## Ursache

GTK 4 zeichnet nach einer Änderung nur den Bereich neu, in dem sich die
Darstellung geändert hat. Den Bereich bestimmt es aus den Grenzen der
alten und neuen Knoten (Render-Node-Diff). Für Text sind das die
Tintengrenzen ohne Hinting. Mit Hinting (Querencia: `font-hinting`
`slight`) ragen gerasterte Glyphen bis zu einem Pixel darüber hinaus. Diese
Reihe gehört zu keinem Schadensbereich und bleibt aus dem alten Bild
stehen. Die Reste sind blass, weil es meist graue Beschriftungen
(`dim-label`) oder Glyphenkanten sind.

Messung (`werkzeug/g2aj-reste.py`): Ein Szenario wechselt zwischen langen
und kurzen Seiten und fotografiert. Danach maximiert und verkleinert es das
Fenster; die neue Puffergröße erzwingt ein vollständiges Neuzeichnen. Das
zweite Foto ist die Referenz aus demselben Lauf, also mit demselben Inhalt.
Jede abweichende Pixelzeile ist ein Rest.

| Lauf (0.0.8, 1920×1080) | Seitenwechsel ohne Reste |
|---|---|
| GPU-Compositor (gles2, GTK nimmt Vulkan auf NVK) | 0/4 |
| pixman-Compositor (GTK nimmt GL bzw. Cairo) | 0/4 |
| wie oben mit `GSK_DEBUG=full-redraw` | 4/4 |
| ohne Hinting (`gtk-xft-hintstyle=hintnone`) | 4/4 |
| `gtk-hint-font-metrics=1` (Hinting bleibt) | 3/4 |

Damit sind Stack-Übergang (steht seit QE-01 auf `None`), Thema und
Render-Backend als Ursache ausgeschlossen. Dieselben Reste zeigt der
Schriftwähler von GTK beim Filtern der Liste.

## Behebung

`main.rs::vollstaendig_zeichnen` setzt beim Start `GSK_DEBUG=full-redraw`,
wenn die Variable nicht gesetzt ist. Das ist eine dokumentierte
GTK-Variable; GTK zeichnet dann jedes Bild ganz. Für ein Fenster dieser
Größe kostet das nichts Spürbares. Die App startet keine GTK-Programme als
Kindprozesse, die Variable wirkt also nur hier. Wer `GSK_DEBUG` selbst
setzt (auch leer), behält seinen Wert. So entsteht auch der Vergleich mit
derselben Fassung:

| 0.0.9, Läufe `g2aj-z-*` | mit Behebung | `GSK_DEBUG=` (aus) |
|---|---|---|
| gles2 1920×1080 | 4/4 | 2/4 |
| gles2 1024×768 | 4/4 | 2/4 |
| pixman 1920×1080 | 4/4 | 2/4 |
| pixman 1024×768 | 4/4 | 2/4 |

Seite „Symbole der Arbeitsfläche“ (pottrauschens fünftes Bild: Reste zwischen
der Quelle-Zeile unter dem Laufwerke-Rahmen und dem Hinweis „Änderungen
gelten erst nach „Übernehmen““), Szenario `z2-symbole`, gles2:

| Lauf | 1920×1080 | 1024×768 |
|---|---|---|
| 0.0.8 (Querencia-Klassisch) | 0/4 | 0/4 |
| 0.0.8 mit eingebautem GTK-Thema | 0/4 | 0/4 |
| 0.0.9, `GSK_DEBUG=` (Behebung aus) | 0/4 | 0/4 |
| 0.0.9 | 4/4 | 4/4 |

Geprüft und ausgeschlossen: Die Seiten werden einmal gebaut
(`bereiche::alle`), ein Seitenwechsel baut nichts neu, und jeder Neuaufbau
(`labwc_seite::optionen`, Panels-Raster, Symbolvorschau) entfernt zuerst
die alten Widgets. Die Labels überlappen nicht (die Referenzbilder nach
vollem Neuzeichnen zeigen sie sauber getrennt). Das Thema ist nicht die
Ursache (gleiches Bild mit dem eingebauten GTK-Thema).

Andere GTK-4-Programme von Querencia haben denselben Fehler, sobald sich
Text an einer Stelle ändert (Files, Systemmonitor, Dialoge). Eine
gemeinsame Lösung gehört in die Sitzung (Umgebung) oder zu GTK selbst.
