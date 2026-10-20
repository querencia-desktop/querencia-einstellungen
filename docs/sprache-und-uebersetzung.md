# Sprache und Region, Tastaturbelegungen, Übersetzung (Einstellungen 0.1.0)

Stand 20.10.2026. Betrifft QD-056 (Bereich „Sprache und Region“), QD-057
(mehrere Tastaturbelegungen mit Umschalttaste) und QD-055 (gettext für das
ganze Programm, Muster `docs/i18n-muster.md` des Editors).

## Bereich „Sprache und Region“

Quelltext `crates/querencia-einstellungen/src/sprache.rs`; Seitengerüst und
Schreibweg wie Tastatur und Maus (`labwc_seite.rs`, Baustein
`labwc_konfig.rs`).

| Wahl | Variable(n) in `environment.d/zz-querencia-einstellungen.env` |
|---|---|
| Anzeigesprache | `LANGUAGE=<sprache>`, dazu `LANG=<Locale der Sprache>` (glibc beachtet `LANGUAGE` in der C-Locale nicht) |
| Formate | `LC_TIME`, `LC_NUMERIC`, `LC_MONETARY`, `LC_PAPER`, `LC_MEASUREMENT` = gewählte Locale, nur wenn sie von `LANG` abweicht |

- **Wirksam ab der nächsten Anmeldung.** Der Sitzungsstarter gibt die
  Variablen beim Anmelden an labwc und an die D-Bus-/systemd-Aktivierung
  weiter (QD-064, gleiche Variablenliste); laufende Programme behalten ihre
  Umgebung. Die Seite ruft deshalb kein `labwc --reconfigure` auf
  (`Bereich::NEU_LADEN = false`) und sagt das im Hinweis und in der
  Statuszeile.
- **Wirksamer Stand:** gelesen wie glibc – `LC_ALL` vor `LC_MESSAGES`/`LC_TIME`
  vor `LANG`; die Anzeigesprache ist der erste Eintrag von `LANGUAGE`, in der
  C-Locale Englisch. Die Querencia-Vorgabe (`session/querencia/labwc/environment`
  im Runtime-Repo) setzt heute `LANG=en_US.UTF-8` und `LANGUAGE=de`: Texte
  deutsch, Datum und Zahlen im US-Format (Befund QD-056). Der Hinweis oben auf
  der Seite nennt diese Vorgabe, die Zeile „Diese Sitzung“ die Umgebung des
  laufenden Programms.
- **„Standard wiederherstellen“** zeigt die Vorgabe; „Übernehmen“ entfernt dann
  alle sieben eigenen Angaben. Gleich der Vorgabe = Zeile entfernen gilt auch
  für jede einzelne Variable (`labwc_seite::env_wert`).
- **Angebot:** Anzeigesprachen = Kataloge dieses Programms (`i18n::kataloge()`,
  Englisch als Quellsprache immer) mit installierter Locale; Formate = alle
  installierten UTF-8-Locales mit Land (`locale -a`), Namen aus iso-codes
  („Deutsch (Schweiz)“). Ein Wert, den es nicht gibt, bleibt als „(nicht
  installiert)“ bzw. „(keine Übersetzung)“ wählbar.
- **Beispiel:** Datum, Uhrzeit, Zahl, Währung, Papier und Maße der gewählten
  Locale aus `locale -k` (eigenes kleines strftime für `d_fmt`/`t_fmt`; das
  `date` der VM ist uutils und folgt der Locale nicht).

## Tastatur: mehrere Belegungen

Quelltext `tastatur.rs`, Hilfen in `xkb.rs`.

- `XKB_DEFAULT_LAYOUT=de,ch`, Varianten parallel in `XKB_DEFAULT_VARIANT`
  (leer, wenn keine gesetzt ist), Umschalttaste als `grp:`-Option in
  `XKB_DEFAULT_OPTIONS`. xkbcommon wechselt selbst; labwc übernimmt alles mit
  `--reconfigure` (`keyboard.c set_layout`).
- Fremde Optionen in `XKB_DEFAULT_OPTIONS` (z. B. `compose:ralt`) bleiben
  stehen; ohne zweite Belegung wird keine Umschalttaste geschrieben.
- Umschalttasten zur Auswahl: `xkb::UMSCHALTER` (13 gängige `grp:`-Optionen
  mit deutschen Namen). `grp:toggle` (rechte Alt-Taste) fehlt absichtlich: auf
  deutschen Tastaturen ist sie AltGr. Ein anderer Wert aus der Datei erscheint
  mit der Beschreibung aus `evdev.xml`.
- Oberfläche: der Kasten „Tastaturbelegung“ oben zeigt die erste Belegung wie
  bisher; „Weitere Belegungen und Umschalten“ steht unten (Nach oben, Nach
  unten, Entfernen, Belegung hinzufügen …, Umschalten mit). So behalten die
  übrigen Bedienelemente ihre Lage. Mit der zweiten Belegung wird Alt+Umschalt
  vorgeschlagen.
- Das Panel hat noch keinen Belegungsanzeiger (Folgepunkt Runtime).

## Übersetzung (gettext)

- msgid englisch, Deutsch aus `po/de.po`; `po/en.po` ist der Katalog der
  Quellsprache (msgstr = msgid). Er sorgt dafür, dass `LANGUAGE=en:de`
  englisch bleibt – ohne `en.mo` fiele gettext auf `de.mo` zurück.
- Kataloge: `QUERENCIA_EINSTELLUNGEN_LOCALEDIR`, sonst neben der Binary
  (`<ablage>/share/locale`), sonst im Präfix (`../share/locale`), sonst
  `/usr/share/locale` (`i18n.rs`).
- Nicht übersetzt: Protokoll (`eprintln!`), die Textausgaben ohne Fenster
  (`--labwc-zeigen`, `--bildschirm-zeigen`, `--standardanwendungen-zeigen`,
  `--faehigkeiten-zeigen`), Dateiinhalte (Kommentare in `bildschirm.ini` und
  der Umgebungsdatei). Fehlermeldungen, die in der Statuszeile erscheinen,
  sind übersetzt; in den Textausgaben ohne Fenster erscheinen sie englisch,
  weil GTK dort die Locale nicht setzt.
- Feste Tabellen (Namen der Bereiche, Paletten, Kategorien, Umschalttasten)
  markiert `gettext_noop`; übersetzt wird beim Anzeigen.
- Dezimaltrenner der Oberfläche (Zeigergeschwindigkeit, Bildwiederholrate,
  Skalierung) über `pgettext("decimal separator", ".")`.
- Bildschirm: „Anwenden“ ist `pgettext("screen settings", "Apply")`, überall
  sonst heißt „Apply“ „Übernehmen“.
- Desktop-Datei: `data/org.querencia.Einstellungen.desktop.in` (englisch) →
  `werkzeug/i18n.py desktop …` → `data/org.querencia.Einstellungen.desktop`
  (byte-gleich zu `msgfmt --desktop`).

Pflege (im Worktree):

```bash
python3 werkzeug/i18n.py pot
python3 werkzeug/i18n.py update
python3 werkzeug/i18n.py quelle en
python3 werkzeug/i18n.py desktop crates/querencia-einstellungen/data/org.querencia.Einstellungen.desktop.in crates/querencia-einstellungen/data/org.querencia.Einstellungen.desktop
python3 werkzeug/i18n.py pruefen
```

`de.po` von Hand übersetzen (fuzzy entfernen); `cargo test` scheitert, wenn
ein Eintrag in `de.po` oder `en.po` fehlt oder leer ist.

## Prüfung

- Unit-Tests: Lesen/Schreiben der Variablen (`sprache.rs`, `tastatur.rs`),
  Beispiele aus `locale -k`, Katalogsuche, Vollständigkeit beider Kataloge.
- GUI isoliert (`werkzeug/010-gui-lauf.sh`, Szenarien aus
  `werkzeug/010-szenarien.py`, Auswertung `werkzeug/auswerten-010.py`): alle
  Seiten deutsch und englisch, Sprache und Region in beiden Oberflächen, ohne
  Benutzerkopie, Neuanmeldung mit der geschriebenen Datei (Umgebung der App,
  eingeblendeter Katalog in `/proc/<pid>/maps`), mehrere Tastaturbelegungen.
  Englisch kommt wie im Betrieb über die eigene Umgebungsdatei der
  Benutzerkopie (`szenarien/010/daten-en`): labwc setzt im isolierten Profil
  `LANGUAGE=de` aus der kopierten Vorgabe für seine Kinder.
