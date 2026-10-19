# Galerie der eingebauten Hintergründe (G2-AF, QD-058)

Der Dialog „Querencia – Hintergrund“ zeigt im Rahmen „Hintergrundbild“
oben alle installierten eingebauten Bilder als Kacheln (Vorschaubild,
Name). Neun Kacheln: „Querencia Blau“ (Standard) und die vier Motive
Abend, Blau 2, Erde, Nacht je ohne und mit Q-Logo. Die Bilder selbst und
ihre Herkunft liegen im Runtime-Repo (`data/hintergruende/`,
`docs/g2af-hintergruende.md`).

## Verhalten

| Aktion | Wirkung |
|---|---|
| Kachel anklicken (oder mit Pfeiltasten wählen und Leertaste/Eingabe) | Typ wird „Hintergrundbild“, `File=` die Fassung für diesen Bildschirm; Vorschau und Statuszeile wie bei „Bild auswählen …“. Geschrieben wird erst bei „Übernehmen“ |
| „Querencia Blau“ | `File=` leer, wie bisher der Standard (auch „Eingebautes Bild“ und „Standard wiederherstellen“) |
| Dialog öffnen | Die Kachel der gespeicherten Einstellung ist markiert; bei einem eigenen Bild keine |
| „Vorschau für:“ wechseln | Die Vorschau zeigt die Fassung für diese Größe (1024×768: 4:3); die geschriebene Datei richtet sich weiter nach dem Bildschirm |

Fassung: Seitenverhältnis des ersten Bildschirms unter 1,555 (5:4, 4:3,
3:2) → `…-4x3.png`, sonst (16:10, 16:9, breiter) die 16:9-Datei. Fehlt die
4:3-Datei, gilt die 16:9-Datei. Unter der Galerie steht, welche Fassung
dieser Bildschirm bekommt.

Gesucht wird wie im Desktop in `querencia/hintergruende/` der
XDG-Datenordner, der des Benutzers zuerst (`~/.local/share`, dann
`/usr/local/share`, `/usr/share`). Nicht installierte Bilder erscheinen
nicht. Die Vorschaubilder (128×72) lädt ein Hintergrundfaden; das Fenster
erscheint sofort.

## Code

| Teil | Ort |
|---|---|
| Katalog: Motive, Dateinamen, Wahl der Fassung, Erkennen einer Datei (ohne GTK, Unit-Tests) | `src/hintergrund_katalog.rs` |
| Galerie, Vorschau je Fassung, Markierung | `src/hintergrund.rs` (`galerie_bauen`, `textur_fuer`, `aktualisieren`) |

`modell.rs` ist unverändert: `EINGEBAUT` bleibt „querencia-blau.png“, die
Datei `desktop.ini` hat dasselbe Format wie bisher.

## Grenzen

- Wechselt später der Bildschirm (z. B. 1920×1080 → 1024×768), bleibt die
  gespeicherte Fassung; das Bild im Dialog neu wählen.
- Mehrere Bildschirme: entscheidend ist der erste.
- Die Übersicht im Kontrollzentrum (Erscheinungsbild) nennt bei einem
  eingebauten Bild den vollen Pfad statt des Namens (`bereiche.rs`, hier
  bewusst nicht geändert, weil G2-AG dort arbeitet).
