# Informationen über Querencia, Seite „Netzwerk“, Fußzeilen (0.1.5)

## Informationen über Querencia

Die Seite zeigt, welche Ausgabe von Querencia Desktop installiert ist, und
führt zur Website, zur Online-Hilfe und zur Fehlermeldung.

| Zeile | Quelle | ohne Versionsdatei |
|---|---|---|
| Titel „Querencia Desktop Alpha 2.1“ | `NAME` und `VERSION` aus `/usr/share/querencia/release`; fehlt `VERSION`, die erste Zeile von `/usr/share/querencia/version` | „Querencia Desktop“ (keine Ausgabe) |
| Querencia Einstellungen 0.1.5 | Programmversion (`CARGO_PKG_VERSION`) | gleich |
| System, Compositor, Desktop, GTK | `/etc/os-release` `PRETTY_NAME`, fest labwc 0.9.8, `XDG_CURRENT_DESKTOP`, GTK zur Laufzeit | gleich |
| Lizenz | `GPL-3.0-or-later` | gleich |
| Webseite (Link) | `HOME_URL` | `https://querencialinux.org` |
| Knopf „Online-Hilfe“ | `DOCUMENTATION_URL` | Übersetzung der Adresse `https://querencialinux.org/en/docs/` (de: `…/de/docs/`) |
| Knopf „Fehler melden“ | `BUG_REPORT_URL` | `https://github.com/querencia-desktop/querencia/issues/new/choose` |

Format der Dateien (festgelegt vom Paketbau, ab Alpha 2.1, Metapaket
`querencia-desktop`): `release` wie os-release(5), `SCHLÜSSEL="Wert"`;
leere Zeilen, `#`-Zeilen und unbekannte Schlüssel werden übergangen; Werte in
doppelten, einfachen oder ohne Anführungszeichen; Escapes `\"`, `\\`,
`` \` ``, `\$`. `version` ist eine Zeile. Der Ordner ist fest
(`produkt::ORDNER`), es gibt keine Umgebungsvariable und keinen anderen
Suchpfad; Tests lesen über `Produkt::lesen_aus(<ordner>)`. Eine fehlende
Datei ist kein Fehler.

Knöpfe und Link öffnen die Adresse mit `GtkUriLauncher` (Portal, sonst die
Standardanwendung über GIO). Klappt das nicht, steht der Grund unter den
Knöpfen. Die Adresse steht als Tooltip am Knopf.

Die Hilfe-Adresse aus `release` wird so genommen, wie sie dort steht (für
Alpha 2.1 `…/de/docs/`, auch bei englischer Oberfläche). Nach Sprache wählt
nur der Rückfall.

## Seite „Netzwerk“ und Startseite

Querencia Einstellungen richtet keine Netzwerkverbindungen ein. Die Seite
„Netzwerk“ erklärt, dass das über das Netzwerksymbol im Panel geht (Netz
wählen, WLAN und Flugmodus, „Verbindungsinformationen …“, Symbol
hinzufügen).

| Aufruf | Wirkung |
|---|---|
| `querencia-einstellungen --netzwerk` | Kontrollzentrum auf der Seite „Netzwerk“ |
| `querencia-einstellungen --seite <name>` / `--seite=<name>` | Kontrollzentrum auf der Seite `<name>` (Namen der Stapelseiten: `erscheinungsbild`, `hintergrundbild`, `symbole`, `fensterdesign`, `schriften`, `panels`, `tastatur`, `maus`, `bildschirm`, `arbeitsflaechen`, `standardanwendungen`, `netzwerk`, `sprache`, `informationen`); unbekannte Namen: Meldung auf stderr, erste Seite |

Die Desktop-Datei hat die Aktion `netzwerk` („Netzwerkeinstellungen“,
`Exec=querencia-einstellungen --netzwerk`). Das Netzwerksymbol des Panels
prüft mit `DesktopAppInfo::list_actions()`, ob es diese Aktion gibt, und
startet sie über „Netzwerkeinstellungen …“.

## Fußzeilen „Quelle: …“

Seiten mit gespeicherten Werten nennen in der letzten Zeile ihres Inhalts,
wo die Werte stehen (`bereiche::quelle`): klein (`<small>`), grau
(`dim-label`), umbrechend, höchstens 70 Zeichen breit, 6 px Abstand nach
oben, über der Knopfleiste. Pfade der Entwicklungsumgebung und Fachbegriffe
der Entwicklung gehören nicht hinein. Kontrast im hellen Thema 4,85:1.
