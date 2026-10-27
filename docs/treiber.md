# Seite „Treiber“ (Einstellungen 0.2.0, QD-170; Zeile „Diese Sitzung“ 0.2.1)

Vorbild ist „Zusätzliche Treiber“ von Ubuntu (software-properties-gtk mit
ubuntu-drivers-common): je Gerät die Treiber, die Ubuntu dafür kennt, zur
Wahl. Querencia nimmt den freien Treiber als Vorgabe; der
NVIDIA-Originaltreiber (in Ubuntu 26.04 `nvidia-driver-615-open`) ist eine
Wahl. Ein ISO für alle Karten (Entscheidung 27.10.2026).

Aufruf: `querencia-einstellungen --seite treiber`; im Menü System →
Systemverwaltung → „Treiber“ über den eigenen Starter
`org.querencia.Einstellungen.Treiber.desktop` (`Categories=Settings;System;HardwareSettings;`,
eigenes Symbol `org.querencia.Einstellungen.Treiber`). Code:
`src/treiber_modell.rs` (Lesen und Zerlegen, Tests), `src/treiber.rs` (Seite,
PackageKit).

## Was die Seite liest

Erst wenn sie zum ersten Mal sichtbar wird, in einem Hintergrund-Thread
(`ubuntu-drivers` braucht einige Sekunden); „Erneut suchen“ liest neu.

| Angabe | Quelle |
|---|---|
| Grafikkarten | `/sys/bus/pci/devices/*`: PCI-Klasse `0x03xxxx`, `vendor`, `device`, `boot_vga` |
| Kerneltreiber | Name des Links `driver` im Geräteordner (`nouveau`, `nvidia`, `amdgpu`, `i915`, `xe` …), wie `render_treiber` des Sitzungsstarters; bei `virtio-pci` der Treiber des Kindgeräts `virtio<n>` (`virtio_gpu`, 0.2.1) |
| Gerätename | Zeile `model` aus `ubuntu-drivers devices`, sonst `lspci -vmm -s <adresse>` |
| Treiberliste | `ubuntu-drivers devices`: je Gerät `driver : <paket> - distro|third-party free|non-free [recommended] [builtin]` |
| Kernelmodul-Paket | `ubuntu-drivers list`: `<paket>, (kernel modules provided by <modulpaket>)` |
| installiert | `dpkg-query -W -f '${Package}\t${db:Status-Abbrev}\n'`, nur Status `ii` |
| OpenGL-Renderer | `querencia-sitzung --grafik-pruefen` (EGL ohne Fenster, Zeitgrenze 5 s) **ohne** die drei Software-Variablen, also der Hardware-Weg; sind die Variablen gesetzt, ein zweites Mal **mit** ihnen (was die Programme der Sitzung bekommen, 0.2.1). Die Zeile „OpenGL-Renderer“ zeigt diese Sitzungsmessung, „OpenGL der Karte“ dann die Hardware-Messung. `querencia-sitzung` wird wie in der Sitzung aufgelöst (0.2.1-2, P-2): `$QUERENCIA_SITZUNG` (setzt der Sitzungsstarter), `~/opt/querencia/bin/querencia-sitzung`, `/usr/bin/querencia-sitzung`, zuletzt `PATH` |
| Software-OpenGL der Sitzung | die eigene Umgebung: `LIBGL_ALWAYS_SOFTWARE=1`, `GALLIUM_DRIVER=llvmpipe` oder `MESA_LOADER_DRIVER_OVERRIDE=llvmpipe` (vom Sitzungsstarter für Autostart und Aktivierung gesetzt) |
| Übersteuerung | `~/.config/querencia/grafik.ini` `[Grafik] Software-OpenGL = automatisch | an | aus` (wie der Starter: letzter Wert, unbekannt = automatisch) |

Fehlt `ubuntu-drivers` im `PATH`, sagt die Seite das und bietet „ubuntu-drivers-common
installieren …“ an (gleicher PackageKit-Weg wie unten). Grafik, Kerneltreiber
und Renderer zeigt sie trotzdem.

## Zeile „Diese Sitzung“ (0.2.1)

Quelle sind die gemessenen Renderer und die Kernelmodule, nicht die
Paketliste (`treiber_modell::darstellung`). Befund P-1 aus Alpha 3.4: 0.2.0
leitete die Zeile nur aus den Variablen ab und zeigte in einer VM mit
llvmpipe „OpenGL auf der Grafikkarte“.

| Fall | Bedingung | Text (de) |
|---|---|---|
| frei | Renderer der Karte (z. B. „AMD …“, „Mesa Intel …“, „zink … NVK“ ohne Variablen) | OpenGL auf der Grafikkarte mit dem freien Treiber |
| proprietär | Renderer von NVIDIA (Hersteller „NVIDIA Corporation“) und Modul `nvidia` geladen | OpenGL auf der Grafikkarte mit dem proprietären NVIDIA-Treiber |
| umgeschaltet | Variablen gesetzt, Sitzungsmessung llvmpipe/softpipe, die Karte selbst kann OpenGL | Software-Darstellung: Die Sitzung hat bei der Anmeldung umgeschaltet, weil es mit dieser Karte ein Grafikproblem gibt (bei `grafik.ini` `an`: „eingeschaltet in ~/.config/querencia/grafik.ini“) |
| ohne 3D | schon die Messung ohne Variablen liefert llvmpipe/softpipe (VM ohne 3D, Karte ohne Treiber) | Software-Darstellung: keine 3D-Beschleunigung, zum Beispiel in einer virtuellen Maschine ohne 3D-Grafik |
| unbekannt | keine Messung und keine Variablen | unbekannt |

Die Messung geht den Variablen vor (der NVIDIA-Treiber beachtet sie nicht);
fehlt die Sitzungsmessung, zählen die Variablen allein. Der Hinweis
„Software-OpenGL“ unten erscheint nicht mehr, wenn der NVIDIA-Treiber zeichnet.

## Zustände und Wahl

Je Treiberzeile: Name (NVIDIA-Treiber mit Nummer, „mit offenen
Kernelmodulen“ bei `-open`, Server-Treiber bei `-server`; „Freier Treiber
nouveau (Querencia-Vorgabe)“), dazu Paket, frei/proprietär, „von Ubuntu
empfohlen“, Fremdquelle und Zustand:

| Zustand | Bedeutung |
|---|---|
| in Gebrauch | Paket installiert (oder `builtin`) und sein Kernelmodul geladen |
| installiert, in Gebrauch nach einem Neustart | Paket installiert, Modul noch nicht geladen |
| verfügbar | `builtin`, aber ein anderer Treiber ist geladen |
| nicht installiert | |

Gewählt ist ein installiertes eigenes Paket (auch vor dem Neustart), sonst
der geladene eingebaute, sonst der erste freie eingebaute Treiber. Modul je
Paket: `nvidia-driver-*` → `nvidia`, `xserver-xorg-video-nouveau|amdgpu|radeon|intel`
→ `nouveau|amdgpu|radeon|i915`.

## Wechsel

„Änderungen anwenden“ plant (`treiber_modell::wechsel`) und fragt nach
(„Treiber wechseln?“ mit den Paketlisten):

- **frei → NVIDIA:** Treiberpaket und das fertige, von Canonical signierte
  Kernelmodul-Paket aus `ubuntu-drivers list` installieren (wie
  `ubuntu-drivers install`; ohne Modulpaket zöge apt DKMS nach).
- **NVIDIA → frei:** alle installierten Pakete desselben Zweigs entfernen:
  Namen mit `nvidia-`, `libnvidia-`, `xserver-xorg-video-nvidia-`,
  `linux-modules-nvidia-`, `linux-objects-nvidia-`, die die Zweignummer als
  eigenes Glied tragen (`libnvidia-gl-615`, `nvidia-firmware-615-615.48.03`).
  Allgemeine Pakete ohne Nummer (`libnvidia-egl-wayland1`, `nvidia-settings`,
  `nvidia-prime`) bleiben. Entfernt wird mit `allow_deps=false`,
  `autoremove=false`; hängt ein anderes Paket daran, meldet PackageKit einen
  Konflikt und nichts geschieht.
- **NVIDIA-Zweig A → B:** erst A entfernen, dann B installieren (zwei
  Transaktionen; polkit fragt wegen `auth_admin_keep` meist nur einmal).

Ausgeführt wird über **`qs_paketkit`** aus Querencia Software
(`../querencia-software-paketkit`, Tag `paketkit/0.1.0`): `paket_nach_name`
(Resolve) bzw. `suchen(name, installiert)` für die installierte Fassung,
dann `ausfuehren_kanal` mit Fortschrittsbalken und „Abbrechen“. Die Rechte
holt PackageKit über polkit; die Passwortfrage zeigt der polkit-Agent der
Sitzung. Fehler (abgelehnt, kein Agent, Paketsperre, kein Netz, Konflikt,
PackageKit beendet) stehen als Satz unter der Liste. Danach liest die Seite
neu.

## Neustart

Nach einem Wechsel, und immer wenn ein installierter Treiber noch nicht
geladen ist oder das geladene Modul keinem installierten Treiber mehr
gehört, zeigt die Seite „Den Rechner neu starten, damit der neue Treiber
geladen wird“ mit „Neu starten …“. Der Knopf ruft `querencia-sitzung
--neustart`, also die Rückfrage der Sitzung; ohne Bestätigung startet nichts
neu.

## Hinweis Software-OpenGL

Bei einer NVIDIA-Karte (nicht mehr, wenn der NVIDIA-Treiber zeichnet, 0.2.1)
oder wenn die Sitzung Software-OpenGL nutzt, erklärt
die Seite den Rückfall (QD-169): zink auf NVK ließ WebKit und Firefox
abstürzen, deshalb schaltet der Sitzungsstarter OpenGL bei der Anmeldung auf
llvmpipe, Vulkan bleibt auf der Karte; Preis: OpenGL auf dem Prozessor. Mit
dem NVIDIA-Treiber meldet die Prüfung keinen zink-Renderer mehr, und die
Sitzung lässt OpenGL von selbst unverändert. Die Seite ändert an der Sitzung
nichts; sie nennt nur `grafik.ini`.

## Prüfung

- Unit-Tests (`treiber_modell::tests`): Ausgaben von `ubuntu-drivers devices`
  für NVIDIA mit nouveau, NVIDIA mit Originaltreiber, AMD, Intel, keine Karte
  (Fixtures unter `testdaten/treiber/`), Rauschen und doppelte Zeilen,
  `ubuntu-drivers list`, dpkg-Status, Zweig-Satz, Wechselplan, Neustart,
  Renderer, Umgebung, `grafik.ini`, sysfs-Attrappen, fehlendes Werkzeug.
- 0.2.1: `darstellung_aus_messung_und_modul`, `befund_nimmt_den_sitzungsrenderer`,
  `virtio_grafiktreiber_am_kindgeraet` mit den Fixtures `testdaten/treiber/grafik-*.txt`
  (llvmpipe VM und umgeschaltet, softpipe, zink/NVK, radeonsi, NVIDIA). GUI
  `werkzeug/021-gui-lauf.sh` (Fälle `szenarien/021/faelle/`, darunter
  `vm-llvmpipe` und `nvidia-nouveau-ohne-sw`; Stub `querencia-sitzung` mit
  `renderer-software.txt` bei `LIBGL_ALWAYS_SOFTWARE=1`; `020-lauf.sh` nimmt
  `FAELLE`/`STUBS` und Modul `a/b` für ein virtio-Kindgerät), Auswertung
  `werkzeug/auswerten-021.py`. 0.2.1-2: `sitzung_programm_wie_die_sitzung_aufloesen`;
  GUI-Läufe `021-h-*` mit `SITZUNG_IN_HEIM=1` (Stub nur unter
  `~/opt/querencia/bin` des Profil-HOME, nicht im `PATH`); die Protokollzeile
  nennt `programm=…`.
- GUI: `werkzeug/020-gui-lauf.sh` mit `werkzeug/020-lauf.sh`. Prüfschalter
  der Seite: `QE_TREIBER_SYSFS=<ordner>` (sysfs-Attrappe; dann kein `lspci`).
  Die Stubs `ubuntu-drivers`, `dpkg-query`, `querencia-sitzung` liegen in
  `szenarien/020/bin/`, die Fälle in `szenarien/020/faelle/`. PackageKit ist
  `werkzeug/pk-attrappe-020.py` auf einem privaten Bus (`werkzeug/020-pk.sh`,
  `QS_PK_BUS`); sie führt einen Bestand, aus dem der dpkg-Stub liest.
- Die Ausgabeformate von `ubuntu-drivers` sind nach ubuntu-drivers-common
  nachgebildet; auf der Entwicklungs-VM ist das Paket nicht installiert. Die
  echte Ausgabe prüft pottrauschen (Prüfliste der Übergabe).

## Grenzen

- Secure Boot: Mit dem signierten Modulpaket aus `ubuntu-drivers list` ist
  keine MOK-Registrierung nötig. Fehlt es (DKMS), fragt die Seite nicht nach
  einem MOK-Passwort; das bleibt Ubuntus Weg auf der Konsole.
- Nur was `ubuntu-drivers` nennt; Treiber anderer Geräte (z. B. WLAN) stehen
  unter der Grafik, sind aber nur im Format geprüft.
- Kein Wechsel ohne Netz (PackageKit lädt die Pakete).
- `lspci` nur für den Namen, wenn `ubuntu-drivers` fehlt.
