# Galerie der eingebauten Hintergründe (G2-AF, QD-058)

Der Dialog „Querencia – Hintergrund“ zeigt im Rahmen „Hintergrundbild“
oben alle installierten eingebauten Bilder als Kacheln (Vorschaubild,
Name). Seit 0.1.4 bis zu 25 Kacheln in dieser Reihenfolge:

1. „Querencia Tiefblau“ (Standard der Einstellungen),
2. die übrigen Motive von Hintergründe 0.2 (QD-093): Blaugrün, Blauschein,
   Glut, Hügel, Küste, Dünen, See, Wüste, Berge, Strand, Glas,
3. ihre Logo-Fassungen: Tiefblau mit Logo, mit Wortmarke, mit Wasserzeichen,
   Bernstein mit Logo (je ein eigenes Motiv, Stamm z. B. `tiefblau-logo`),
4. die Bilder von 0.1: „Querencia Blau“ (das eingebaute Bild des Desktops)
   und die vier Motive Abend, Blau 2, Erde, Nacht je ohne und mit Q-Logo.

Die Bilder selbst und ihre Herkunft liegen im Runtime-Repo
(`data/hintergruende/`, `docs/g2af-hintergruende.md`,
`docs/hintergruende-02.md`). Live liegen sie ohne Paket als Verweise in
`~/.local/share/querencia/hintergruende/` (Ziel
`~/opt/querencia/hintergruende/<sha16>/`); erkannt wird der Pfad des
Verweises. Von 0.2 kennt der Katalog nur
`querencia-<stamm>.png` (2560×1440) und `querencia-<stamm>-4x3.png`; die
Größen `-3840`/`-1920` erkennt er nicht (gewählt über „Bild auswählen …“
gelten sie als eigenes Bild). Lange Namen stehen in zwei Zeilen, der volle
Name im Tooltip.

## Verhalten

| Aktion | Wirkung |
|---|---|
| Kachel anklicken (oder mit Pfeiltasten wählen und Leertaste/Eingabe) | Typ wird „Hintergrundbild“, `File=` die Fassung für diesen Bildschirm; Vorschau und Statuszeile wie bei „Bild auswählen …“. Geschrieben wird erst bei „Übernehmen“ |
| Kachel des eingebauten Bilds, „Standard wiederherstellen“, „Eingebautes Bild“ | `File=` leer: das eingebaute Bild des Desktops (`BUILT_IN` in `querencia-desktop/src/config.rs`, 16:9). 0.1.4: das ist „Querencia Tiefblau“, sobald es installiert ist (Desktop ab 0.2.1), sonst „Querencia Blau“ wie bis 0.1.3 |
| die andere der beiden Kacheln | mit Pfad wie jedes andere Motiv (bei installiertem Tiefblau also „Querencia Blau“ mit `…/querencia-blau.png`) |
| Dialog öffnen | Die Kachel der gespeicherten Einstellung ist markiert; bei einem eigenen Bild keine |
| „Vorschau für:“ wechseln | Die Vorschau zeigt die Fassung für diese Größe (1024×768: 4:3); die geschriebene Datei richtet sich weiter nach dem Bildschirm |

Fassung: Seitenverhältnis des ersten Bildschirms unter 1,555 (5:4, 4:3,
3:2) → `…-4x3.png`, sonst (16:10, 16:9, breiter) die 16:9-Datei. Fehlt die
4:3-Datei, gilt die 16:9-Datei. Unter der Galerie steht, welche Fassung
dieser Bildschirm bekommt.

Gesucht wird wie im Desktop in `querencia/hintergruende/` der
XDG-Datenordner, der des Benutzers zuerst (`~/.local/share`, dann
`/usr/local/share`, `/usr/share`). Nicht installierte Bilder erscheinen
nicht (maßgeblich ist die 16:9-Datei; `katalog::verfuegbar`). So kann
0.1.4 vor dem Datenpaket von Hintergründe 0.2 live gehen: ohne die neuen
Dateien zeigt die Galerie dieselben neun Kacheln wie 0.1.3. Steht in
`desktop.ini` ein Bild, das fehlt, meldet der Dialog es wie bisher als
nicht lesbar. Die Vorschaubilder (128×72) lädt ein Hintergrundfaden; das Fenster
erscheint sofort.

## Code

| Teil | Ort |
|---|---|
| Katalog: Motive, Dateinamen, Wahl der Fassung, Erkennen einer Datei, verfügbare Kacheln, `Wahl::STANDARD` (Tiefblau) und `Wahl::BLAU` (ohne GTK, Unit-Tests) | `src/hintergrund_katalog.rs` |
| Galerie, Vorschau je Fassung, Markierung, eingebautes Bild (`eingebaut_wahl`, `eingebautes_bild`) | `src/hintergrund.rs` (`galerie_bauen`, `textur_fuer`, `aktualisieren`) |

`modell.rs`: `EINGEBAUT` ist seit 0.1.4 „querencia-tiefblau.png“ wie
`BUILT_IN` ab Desktop 0.2.1, `EINGEBAUT_0_1` „querencia-blau.png“ (Desktop
bis 0.2.0). Die Datei `desktop.ini` hat dasselbe Format wie bisher.

Grenze der Übergangszeit: Die Einstellungen sehen nur, ob Tiefblau
installiert ist, nicht welcher Desktop läuft. Liegen die Bilder von
Hintergründe 0.2 schon da und läuft noch Desktop 0.2.0, zeigt der Desktop
bei leerem `File=` Blau, die Einstellungen nennen Tiefblau. Datenpaket und
Desktop 0.2.1 gehören deshalb zusammen live. Der Standard schreibt immer
das leere `File=` (16:9-Vorgabe des Desktops); eine 4:3-Fassung für den
Standard müsste der Desktop nach dem Seitenverhältnis wählen.

## Grenzen

- Wechselt später der Bildschirm (z. B. 1920×1080 → 1024×768), bleibt die
  gespeicherte Fassung; das Bild im Dialog neu wählen.
- Mehrere Bildschirme: entscheidend ist der erste.
- Die Übersicht im Kontrollzentrum (Erscheinungsbild) nennt bei einem
  eingebauten Bild den vollen Pfad statt des Namens (`bereiche.rs`, hier
  bewusst nicht geändert, weil G2-AG dort arbeitet).

## Fähigkeiten des Desktops (Nebenbefund, 0.0.7)

`faehigkeiten.rs` ruft `querencia-desktop --faehigkeiten` nur auf, wenn das
Programm ein Kennzeichen enthält (Schutz vor HMP-10a, das die Option als
zweiten Start missverstünde). Bis 0.0.6 war das nur die Zeichenkette
`--faehigkeiten`. Im Release-Bau des Desktops G2-AD (`9d622162`) ist sie
wegoptimiert; die Einstellungen hielten ihn deshalb für HMP-10a und sperrten
Farbe, Zentrieren, Kacheln und die Symbole der Arbeitsfläche. Ab 0.0.7 gilt
auch die Antwortzeile `type-color mode-centered` als Kennzeichen; sie steht in
jedem Desktop mit der Option im Programm und nicht in HMP-10a (Unit-Test
`kennzeichen_option_oder_antwortzeile`).
