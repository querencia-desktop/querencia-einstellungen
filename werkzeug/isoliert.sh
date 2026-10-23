#!/bin/bash
# Isolierte GUI-Prüfung für QE-01: eigener headless labwc 0.9.8 (pixman), eigener
# D-Bus, eigenes Profil (XDG_CONFIG_HOME/STATE_HOME), eigener XDG_RUNTIME_DIR.
# Darin laufen der ECHTE Desktop (HMP-10a) und das ECHTE Panel (QREL-04c) aus
# ~/opt (nur gelesen) sowie der Dialog aus diesem Projekt. Eingaben kommen
# ausschließlich über qe-eingabe (verweigert außerhalb von hmp05-runtime-*).
# Die produktive Sitzung wird nie berührt: HOME bleibt (Themen, Bilder lesen),
# aber alles Geschriebene landet im Profil.
#
#   isoliert.sh <name> <BREITExHÖHE> <szenario.txt> [desktop.ini-Vorlage]
#
# Ergebnis: nachweis/<name>/ mit logs/, bildern (shot …), desktop.ini nachher.
set -euo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
NAME=${1:?Name}; AUFL=${2:?BREITExHÖHE}; SZENARIO_QUELLE=$(readlink -f "${3:?Szenario}"); INI=${4:-}
# Desktop: Standard HMP-10a aus der Ablage; für die Nachprüfung mit W1s neuem
# Build: QE_DESKTOP=<pfad> QE_DESKTOP_SHA16=<sha16> isoliert.sh …
DESKTOP_BIN=${QE_DESKTOP:-$HOME/opt/querencia/desktop/2b46a7b90b3412bb/querencia-desktop}
DESKTOP_SHA16=${QE_DESKTOP_SHA16:-2b46a7b90b3412bb}
# 0.1.2 (G1a): Pfade über $HOME, umstellbar über QE_PANEL, QE_LABWC_BIN, QE_WERKSTATT_WERK.
PANEL_BIN=${QE_PANEL:-$HOME/opt/querencia/panel/696b8797a3a35eb8/querencia-panel}
LABWC=${QE_LABWC_BIN:-$HOME/opt/labwc-0.9.8/bin/labwc}
# G2-C: Release-Build über QE_APP_BIN=<pfad>; ohne Variable der Debug-Build wie bisher.
# G2-G: QE_OUTPUTS=2 gibt dem headless labwc zwei Ausgaben (die zweite 1280x720, rechts).
APP=${QE_APP_BIN:-$P/target/debug/querencia-einstellungen}
WERK=$P/werkzeug/bin
WERKSTATT_WERK=${QE_WERKSTATT_WERK:-$HOME/projects/querencia-panel-werkstatt/arbeitsflaechen-4/werkzeug/bin}
for f in "$DESKTOP_BIN" "$PANEL_BIN" "$LABWC" "$APP" "$WERK/qe-eingabe" "$WERKSTATT_WERK/qausgabe"; do
    [ -x "$f" ] || { echo "fehlt: $f" >&2; exit 2; }
done
[ "$(sha256sum "$DESKTOP_BIN" | cut -c1-16)" = "$DESKTOP_SHA16" ] || { echo "Desktop-Binary unerwartet" >&2; exit 2; }
[ "$(sha256sum "$PANEL_BIN" | cut -c1-16)" = 696b8797a3a35eb8 ] || { echo "Panel-Binary unerwartet" >&2; exit 2; }

N=$P/nachweis/$NAME; rm -rf "$N"; mkdir -p "$N/logs" "$N/profil"
PROFIL=$N/profil
cp -r "$P/werkzeug/profil-vorlage/." "$PROFIL/"
# PT-1: Ohne Settings-Portal nimmt GTK seine eingebaute Schrift (kleiner als
# live, verschiebt Layouts und Klickziele). Die Schrift der echten Sitzung
# (nur gelesen) kommt deshalb ins Profil, außer es hat eine eigene.
# G2-AG: QE_GTK_SCHRIFT="<Schrift Größe>" legt die Schrift fest (gleiche
# Klickpunkte auch nach einer Änderung der Live-Schrift, z. B. 20.10. 01:02
# Erscheinungsbild Variante A → DejaVu Sans 10).
if [ -n "${QE_GTK_SCHRIFT:-}" ]; then
    mkdir -p "$PROFIL/config/gtk-4.0"
    printf '[Settings]\ngtk-font-name=%s\n' "$QE_GTK_SCHRIFT" > "$PROFIL/config/gtk-4.0/settings.ini"
fi
if [ ! -e "$PROFIL/config/gtk-4.0/settings.ini" ]; then
    SCHRIFT=$(gsettings get org.gnome.desktop.interface font-name 2>/dev/null | tr -d "'" || true)
    if [ -n "$SCHRIFT" ]; then
        mkdir -p "$PROFIL/config/gtk-4.0"
        printf '[Settings]\ngtk-font-name=%s\n' "$SCHRIFT" > "$PROFIL/config/gtk-4.0/settings.ini"
    fi
fi
# G2-AJ: QE_GTK_THEMA="<GTK-Thema>[:<Symbolthema>]" legt Thema und Symbole des
# Fensters fest (ohne Portal liest GTK nur settings.ini), z. B.
# "Querencia-Klassisch:Querencia" wie live seit 20.10.
if [ -n "${QE_GTK_THEMA:-}" ]; then
    mkdir -p "$PROFIL/config/gtk-4.0"
    [ -e "$PROFIL/config/gtk-4.0/settings.ini" ] || printf '[Settings]\n' > "$PROFIL/config/gtk-4.0/settings.ini"
    printf 'gtk-theme-name=%s\n' "${QE_GTK_THEMA%%:*}" >> "$PROFIL/config/gtk-4.0/settings.ini"
    case $QE_GTK_THEMA in *:*) printf 'gtk-icon-theme-name=%s\n' "${QE_GTK_THEMA#*:}" >> "$PROFIL/config/gtk-4.0/settings.ini";; esac
fi
mkdir -p "$PROFIL/state" "$PROFIL/schreibtisch" "$PROFIL/bilder" "$PROFIL/bin"
# A3: querencia-einstellungen über PATH (nur in dieser isolierten Sitzung)
ln -s "$APP" "$PROFIL/bin/querencia-einstellungen"
echo "Desktop: $DESKTOP_BIN ($DESKTOP_SHA16)" > "$N/desktop-binary.txt"
# Testbilder ins Profil (der Dateidialog startet dort)
cp "$P"/nachweis/testbilder/*.png "$P"/nachweis/testbilder/*.jpg "$P"/nachweis/testbilder/*.webp "$PROFIL/bilder/"
sed -i "s#\$HOME/schreibtisch#$PROFIL/schreibtisch#; s#\$HOME/bilder#$PROFIL/bilder#" "$PROFIL/config/user-dirs.dirs"
# 0.1.2 (G1a): @WURZEL@ in Szenario und desktop.ini-Vorlage steht für diesen
# Checkout (statt eines festen Pfads); das Szenario läuft als Kopie im Lauf.
SZENARIO=$N/szenario.txt
sed "s#@WURZEL@#$P#g" "$SZENARIO_QUELLE" > "$SZENARIO"
if [ -n "$INI" ]; then sed "s#@WURZEL@#$P#g" "$INI" > "$PROFIL/config/querencia/desktop.ini"; fi
# G2-G: gespeicherte Bildschirmkonfiguration ins Profil (für --bildschirm-anwenden)
if [ -n "${QE_BILDSCHIRM_INI:-}" ]; then cp "$QE_BILDSCHIRM_INI" "$PROFIL/config/querencia/bildschirm.ini"; fi
# 0.1.6: dasselbe für die neue Datei bildschirm.toml
if [ -n "${QE_BILDSCHIRM_TOML:-}" ]; then cp "$QE_BILDSCHIRM_TOML" "$PROFIL/config/querencia/bildschirm.toml"; fi
# G2-Z: mimeapps.list ins Profil (GIO liest und schreibt $XDG_CONFIG_HOME/mimeapps.list);
# ohne Variable gibt es dort keine Datei. Die Starter liest GIO weiter (nur lesend)
# aus ~/.local/share/applications, weil XDG_DATA_HOME ungesetzt bleibt.
if [ -n "${QE_MIMEAPPS:-}" ]; then cp "$QE_MIMEAPPS" "$PROFIL/config/mimeapps.list"; cp "$QE_MIMEAPPS" "$N/mimeapps.list.vorher"; fi
# G2-Z: vorrangige Liste des Desktops (XDG_CURRENT_DESKTOP=X-Querencia), Prüfung der Warnung
if [ -n "${QE_MIMEAPPS_VORRANG:-}" ]; then cp "$QE_MIMEAPPS_VORRANG" "$PROFIL/config/x-querencia-mimeapps.list"; fi
# G2-AA: eigene labwc-rc.xml (z. B. Symbolthema gnome-human wie live, Fensterregel
# iconPreferClient) und eigenes XDG_DATA_HOME (Starter/Symbole, die labwc sieht –
# simuliert einen labwc, der Starter und Symbole vor G2-V eingelesen hat).
if [ -n "${QE_LABWC_RC:-}" ]; then cp "$QE_LABWC_RC" "$PROFIL/labwc/rc.xml"; fi
if [ -n "${QE_DATA_HOME:-}" ]; then mkdir -p "$PROFIL/share"; cp -r "$QE_DATA_HOME/." "$PROFIL/share/"; fi
# 0.1.2 (G1a): @HOME@ in den kopierten Startern steht für das echte HOME (Ablage unter ~/opt)
if [ -d "$PROFIL/share/applications" ]; then sed -i "s#@HOME@#$HOME#g" "$PROFIL"/share/applications/*.desktop; fi
mkdir -p "$HOME/.cache/qe01"; RT=$(mktemp -d "$HOME/.cache/qe01/hmp05-runtime-XXXXXX"); chmod 0700 "$RT"; echo "$RT" > "$N/laufzeitordner.txt"
# PT-1 (nach SW-6, QD-037): keine Portale im Wegwerf-Bus. Sonst startet
# xdg-document-portal und sein FUSE-Mount $RT/doc erscheint im echten Querencia
# Files unter „Geräte“. $XDG_RUNTIME_DIR/dbus-1/services hat beim Sitzungsbus
# Vorrang (verlangt Modus 0700); XDG_DATA_HOME bleibt so wie bisher ungesetzt.
mkdir -m 0700 "$RT/dbus-1" "$RT/dbus-1/services"
# 0.1.2 (QD-085, 13 §5): auch gvfsd maskieren – ein Dateidialog im Wegwerf-Bus
# startet sonst gvfsd und darüber wsdd (Netzwerksuche, läuft nach dem Lauf weiter).
for n in org.freedesktop.portal.Documents org.freedesktop.portal.Desktop org.freedesktop.impl.portal.PermissionStore org.gtk.vfs.Daemon; do
    printf '[D-BUS Service]\nName=%s\nExec=/bin/false\n' "$n" > "$RT/dbus-1/services/$n.service"
done
cp "$PROFIL/config/querencia/desktop.ini" "$N/desktop.ini.vorher" 2>/dev/null || true
# G2-AG: labwc-Konfiguration. Die Vorgabe (rc.xml, environment) kommt als KOPIE
# ins Profil (QE_LABWC_VORGABE, die App liest nie das Runtime-Repo); autostart
# wird nie kopiert (labwc würde ihn im Wegwerfprofil ausführen).
#   QE_LABWC=benutzer  Benutzerkopie im Profil wie nach dem Sitzungsstarter
#                      (Teil A), labwc läuft mit -C darauf → „sofort“
#   QE_LABWC=ohne      keine Benutzerkopie, labwc liest das Profil-labwc → „ab
#                      der nächsten Anmeldung“
#   QE_LABWC_DATEIEN=<ordner>  danach dessen Inhalt in die Benutzerkopie (Szenarien)
VQ=${QE_LABWC_VORGABE_QUELLE:-${QUERENCIA_REPO:-$HOME/projects/querencia-runtime}/session/querencia/labwc}
mkdir -p "$PROFIL/vorgabe"; cp "$VQ/rc.xml" "$VQ/environment" "$PROFIL/vorgabe/"
LABWC_DIR=$PROFIL/labwc
if [ "${QE_LABWC:-}" = benutzer ]; then
    LABWC_DIR=$PROFIL/config/querencia/labwc
    mkdir -p "$LABWC_DIR/.vorgabe"
    cp "$PROFIL/vorgabe/rc.xml" "$PROFIL/vorgabe/environment" "$LABWC_DIR/"
    cp "$PROFIL/vorgabe/rc.xml" "$PROFIL/vorgabe/environment" "$LABWC_DIR/.vorgabe/"
fi
if [ -n "${QE_LABWC_DATEIEN:-}" ]; then mkdir -p "$PROFIL/config/querencia/labwc"; cp -a "$QE_LABWC_DATEIEN/." "$PROFIL/config/querencia/labwc/"; fi
if [ -d "$PROFIL/config/querencia/labwc" ]; then cp -a "$PROFIL/config/querencia/labwc" "$N/labwc-vorher"; fi

cat > "$N/start.sh" <<EOF
#!/bin/bash
# läuft innerhalb des isolierten labwc (-S), WAYLAND_DISPLAY gesetzt
set -u
cd "$N"
# G2-AJ: gsettings im Profil-dconf vorbelegen (QE_GSETTINGS: Zeilen
# "schlüssel wert" für org.gnome.desktop.interface); Stand am Anfang und Ende.
if [ -n "\$QE_GSETTINGS" ]; then
    while read -r k v; do [ -n "\$k" ] && gsettings set org.gnome.desktop.interface "\$k" "\$v"; done < "\$QE_GSETTINGS"
fi
gsettings list-recursively org.gnome.desktop.interface > gsettings-start.txt 2>&1
# G2-AJ: QE_PORTAL=settings startet den Settings-Portal-Ersatz (nur
# org.freedesktop.portal.Settings aus dem Profil-dconf); GTK-Programme folgen
# dann gsettings-Änderungen sofort wie in der echten Sitzung.
PORTAL=
if [ "\$QE_PORTAL" = settings ]; then
    python3 -I "$P/werkzeug/g2aj-settings-portal.py" >> logs/portal.log 2>&1 &
    PORTAL=\$!
    sleep 0.8
fi
"$WERKSTATT_WERK/qausgabe" "$AUFL" >> logs/qausgabe.log 2>&1; sleep 0.5
"$DESKTOP_BIN" >> logs/desktop.log 2>&1 &
DPID=\$!
sleep 1.5
"$PANEL_BIN" >> logs/panel.log 2>&1 &
PPID_=\$!
sleep 2
cp "\$XDG_STATE_HOME/querencia/desktop-symbole" symbole-start 2>/dev/null || echo fehlt > symbole-start
"$APP" --faehigkeiten-zeigen > logs/faehigkeiten.log 2>&1 || true
# G2-G: Ausgaben am Anfang und am Ende (wlr-output-management, nur lesend).
# Nur ab 0.0.2: 0.0.1 kennt die Option nicht, ignoriert sie und öffnet das
# Kontrollzentrum (blockiert bis zur Zeitgrenze des Laufs). Die Version kommt
# aus --version (0.0.1 antwortet ohne Fenster); ein grep im Binary versagt,
# weil der Release-Build kurze Konstanten als Direktwerte vergleicht.
if [ "\$("$APP" --version 2>/dev/null)" != "querencia-einstellungen 0.0.1" ]; then
    timeout 10 "$APP" --bildschirm-zeigen > logs/ausgaben-start.txt 2>>logs/ausgaben.log || true
fi
# G2-C: mit QE_APP_ARGS wird die App vor dem Szenario direkt gestartet (z. B.
# "--hintergrund" oder "-" für das Kontrollzentrum ohne Argument).
if [ -n "\$QE_APP_ARGS" ]; then
    # G2-AA: QE_APP_WAYLAND_DEBUG=1 protokolliert die Wayland-Anfragen der App
    # (xdg_toplevel_icon) nach logs/app-direkt.log.
    WD=; [ -n "\$QE_APP_WAYLAND_DEBUG" ] && WD="WAYLAND_DEBUG=client"
    if [ "\$QE_APP_ARGS" = "-" ]; then env \$WD "$APP" >> logs/app-direkt.log 2>&1 &
    else env \$WD "$APP" \$QE_APP_ARGS >> logs/app-direkt.log 2>&1 & fi
    echo "app direkt gestartet: \$QE_APP_ARGS" >> logs/app-direkt.log
fi
"$WERK/qe-eingabe" < "$SZENARIO" >> logs/eingabe.log 2>&1
echo "szenario exit=\$?" >> logs/eingabe.log
sleep 0.5
# nur Prozesse DIESER isolierten Sitzung (gleiches XDG_RUNTIME_DIR) beenden
# 0.1.2: Prozesse nur über /proc/<pid>/exe, nicht über die Kommandozeile
APP_EXE=\$(readlink -f "$APP")
for d in /proc/[0-9]*; do [ "\$(readlink "\$d/exe" 2>/dev/null)" = "\$APP_EXE" ] && grep -qz "^XDG_RUNTIME_DIR=\$XDG_RUNTIME_DIR\$" "\$d/environ" 2>/dev/null && kill -TERM "\${d#/proc/}"; done
if [ "\$("$APP" --version 2>/dev/null)" != "querencia-einstellungen 0.0.1" ]; then
    timeout 10 "$APP" --bildschirm-zeigen > logs/ausgaben-ende.txt 2>>logs/ausgaben.log || true
fi
cp "\$XDG_CONFIG_HOME/querencia/bildschirm.ini" bildschirm.ini.nachher 2>/dev/null || true
cp "\$XDG_CONFIG_HOME/querencia/bildschirm.toml" bildschirm.toml.nachher 2>/dev/null || true
gsettings list-recursively org.gnome.desktop.interface > gsettings-ende.txt 2>&1
kill -TERM \$PPID_ \$DPID \$PORTAL 2>/dev/null || true
sleep 1
cp "\$XDG_STATE_HOME/querencia/desktop-symbole" symbole-ende 2>/dev/null || echo fehlt > symbole-ende
EOF
chmod +x "$N/start.sh"
echo "Profil $PROFIL, Laufzeit $RT, Auflösung $AUFL"
HOME_REAL=$HOME
# 0.1.2: eigenes HOME im Profil (heim/). Themen, Symbole, Starter und die
# Galerie der Hintergründe kommen als Verweisbaum (cp -rs, nur gelesen) bzw.
# Kopie (Starter) dazu, damit das Fenster aussieht wie live.
#   QE_THEMA_ORDNER=<ordner>  statt des installierten Querencia-Klassisch
#                             (z. B. Thema 0.2 aus dem Thema-Worktree)
HEIM=$PROFIL/heim
mkdir -p "$HEIM/.local/share"
for d in icons themes querencia; do
    [ -d "$HOME_REAL/.local/share/$d" ] && cp -rs "$HOME_REAL/.local/share/$d" "$HEIM/.local/share/$d"
done
[ -d "$HOME_REAL/.local/share/applications" ] && cp -a "$HOME_REAL/.local/share/applications" "$HEIM/.local/share/"
# 0.1.4: QE_HINTERGRUENDE=<ordner>[:<ordner>…] ersetzt die eingebauten Bilder
# im Profil durch Verweise auf die querencia-*.png dieser Ordner (unabhängig
# davon, was live in ~/.local/share/querencia/hintergruende liegt).
if [ -n "${QE_HINTERGRUENDE:-}" ]; then
    rm -rf "${HEIM:?}/.local/share/querencia/hintergruende"
    mkdir -p "$HEIM/.local/share/querencia/hintergruende"
    IFS=: read -ra hg_ordner <<<"$QE_HINTERGRUENDE"
    for o in "${hg_ordner[@]}"; do
        for f in "$o"/querencia-*.png; do ln -s "$(readlink -f "$f")" "$HEIM/.local/share/querencia/hintergruende/"; done
    done
    ls "$HEIM/.local/share/querencia/hintergruende" > "$N/hintergruende.txt"
fi
if [ -n "${QE_THEMA_ORDNER:-}" ]; then
    rm -rf "${HEIM:?}/.local/share/themes/Querencia-Klassisch"
    mkdir -p "$HEIM/.local/share/themes"
    ln -s "$(readlink -f "$QE_THEMA_ORDNER")" "$HEIM/.local/share/themes/Querencia-Klassisch"
fi
# 0.1.5: QE_HEIM_DATEIEN=<ordner> – dessen Inhalt zusätzlich ins Profil-HOME
# (z. B. ein Starter unter .local/share/applications, der Adressen nur
# protokolliert, statt einen Browser zu öffnen).
if [ -n "${QE_HEIM_DATEIEN:-}" ]; then cp -a "$QE_HEIM_DATEIEN/." "$HEIM/"; fi
# 0.1.6: QE_THEMEN_WEG=<name>[:<name>…] – diese Themen fehlen im Profil-HOME
# (z. B. der dunkle Fensterrahmen, um „nicht installiert“ zu prüfen).
if [ -n "${QE_THEMEN_WEG:-}" ]; then
    IFS=: read -ra weg_themen <<<"$QE_THEMEN_WEG"
    for t in "${weg_themen[@]}"; do rm -rf "${HEIM:?}/.local/share/themes/${t:?}"; done
fi
echo "HOME=$HEIM Thema=$(readlink -f "$HEIM/.local/share/themes/Querencia-Klassisch")" > "$N/heim.txt"
# 0.1.2: QE_PROFIL_STABIL=<verweis> – die Sitzung sieht das Profil unter einem
# festen Verweis statt unter nachweis/<lauf>/profil, damit angezeigte Pfade
# (z. B. „Quelle: …/panel.ini“) in Bildvergleichen zweier Läufe gleich sind.
# Nur für nacheinander laufende Läufe (der Verweis zeigt auf den letzten).
if [ -n "${QE_PROFIL_STABIL:-}" ]; then
    mkdir -p "$(dirname "$QE_PROFIL_STABIL")"
    ln -sfn "$PROFIL" "$QE_PROFIL_STABIL"
    echo "Profil sichtbar als $QE_PROFIL_STABIL" >> "$N/heim.txt"
fi
(
  if [ -n "${QE_PROFIL_STABIL:-}" ]; then
      LABWC_DIR=$QE_PROFIL_STABIL${LABWC_DIR#"$PROFIL"}; PROFIL=$QE_PROFIL_STABIL; HEIM=$PROFIL/heim
  fi
  export XDG_RUNTIME_DIR="$RT" XDG_CONFIG_HOME="$PROFIL/config" XDG_STATE_HOME="$PROFIL/state" XDG_CACHE_HOME="$PROFIL/cache"
  # G2-AJ: QE_WLR_RENDERER=gles2 lässt den headless labwc auf der Grafikkarte
  # zeichnen (Render-Node, dmabuf) wie die echte Sitzung; Standard bleibt pixman.
  # QE_GSK_RENDERER setzt GSK_RENDERER der App (Vergleich der GTK-Renderer).
  export WLR_BACKENDS=headless WLR_RENDERER=${QE_WLR_RENDERER:-pixman} WLR_HEADLESS_OUTPUTS="${QE_OUTPUTS:-1}" WLR_LIBINPUT_NO_DEVICES=1
  if [ -n "${QE_GSK_RENDERER:-}" ]; then export GSK_RENDERER=$QE_GSK_RENDERER; fi
  export XDG_CURRENT_DESKTOP=X-Querencia XDG_SESSION_TYPE=wayland LANG=en_US.UTF-8 LANGUAGE=de XKB_DEFAULT_LAYOUT=de
  if [ "${QE_PORTAL:-}" = settings ]; then export GTK_USE_PORTAL=0; else export GDK_DEBUG=no-portals GTK_USE_PORTAL=0; fi
  export QUERENCIA_STATE_DIR="$PROFIL/state/querencia" QE_APP="$APP" QE_APP_ARGS="${QE_APP_ARGS:-}" QE_GSETTINGS="${QE_GSETTINGS:+$(readlink -f "$QE_GSETTINGS")}" QE_PORTAL="${QE_PORTAL:-}" QE_APP_WAYLAND_DEBUG="${QE_APP_WAYLAND_DEBUG:-}" PATH="$PROFIL/bin:$PATH"
  export QE_LABWC_VORGABE="$PROFIL/vorgabe" HOME="$HEIM" GIO_USE_VFS=local
  unset WAYLAND_DISPLAY DISPLAY DBUS_SESSION_BUS_ADDRESS
  if [ -n "${QE_DATA_HOME:-}" ]; then export XDG_DATA_HOME="$PROFIL/share"; fi
  mkdir -p "$PROFIL/state/querencia" "$PROFIL/cache"
  timeout ${QE_ZEIT:-180} dbus-run-session -- "$LABWC" ${QE_LABWC_DEBUG:+-d} -C "$LABWC_DIR" -S "$N/start.sh" > "$N/logs/labwc.log" 2>&1 || echo "labwc exit=$?" >> "$N/logs/labwc.log"
)
cp "$PROFIL/config/querencia/desktop.ini" "$N/desktop.ini.nachher" 2>/dev/null || true
cp "$PROFIL/config/mimeapps.list" "$N/mimeapps.list.nachher" 2>/dev/null || true
ls "$PROFIL/config/querencia/" > "$N/konfigordner-nachher.txt"
if [ -d "$PROFIL/config/querencia/labwc" ]; then cp -a "$PROFIL/config/querencia/labwc" "$N/labwc-nachher"; fi
ls -A "$PROFIL/config/" > "$N/konfig-wurzel-nachher.txt"
echo "--- fertig: $N"; ls "$N" | tr '\n' ' '; echo; tail -3 "$N/logs/eingabe.log"
# PT-1: falls doch ein Portal lief, nur den eigenen doc-Mount aushängen
# (über /proc/self/mounts, ein toter FUSE-Mount lässt sich nicht stat'en).
DOC=$(readlink -f "$RT")/doc
if awk -v d="$DOC" '$2 == d { f = 1 } END { exit !f }' /proc/self/mounts; then
    fusermount3 -u "$DOC" 2>/dev/null || fusermount3 -u -z "$DOC" || true
fi
rm -rf "$RT"
