#!/bin/bash
# Einstellungen 0.1.5 (Informationen, Fußzeilen, Seite „Netzwerk“): isolierte
# GUI-Läufe wie werkzeug/014-gui-lauf.sh über werkzeug/g2aj-lauf.sh (eigenes
# HOME, XDG_RUNTIME_DIR, Bus und labwc – isoliert.sh; Settings-Portal-Ersatz
# color-scheme 0/1, gvfsd maskiert, GIO_USE_VFS=local). Thema: die
# Live-Ablage von Querencia-Klassisch (nur gelesen). Eingebaute Bilder fest
# 0.1 + 0.2 (QE_HINTERGRUENDE, wie live seit 21.10.).
#
# Die Versionsdatei liegt fest unter /usr/share/querencia (kein Suchpfad,
# keine Umgebungsvariable). Für „mit“ läuft die ganze isolierte Sitzung in
# einem bwrap-Namensraum mit Wegwerf-Overlay über /usr/share, darin
# /usr/share/querencia = szenarien/015/<ordner> (nur lesend); das echte
# /usr/share bleibt unberührt. /tmp/.X11-unix bekommt im Namensraum ein
# eigenes tmpfs (sonst gehört der echte Ordner dort „nobody“, und labwc
# startet Xwayland nicht).
#   d1  alle Seiten und Erscheinungsbild unten: alt (0.1.4) und neu, de
#       1024×768 und en 1920×1080, je hell und dunkel, ohne Versionsdatei
#   i1  Start mit --seite informationen: neu, de 1024 / en 1920, hell und
#       dunkel, ohne und mit release (Alpha 2.1); dazu de 1024 hell nur mit
#       der Datei version
#   n1  Start mit --netzwerk: de 1024 hell, en 1920 dunkel
# Läufe nach nachweis/015-<art>-…-<sprache>-<breite>/, nacheinander.
#
#   bash werkzeug/015-gui-lauf.sh [nur-läufe-mit-diesem-muster]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
ALT=${ALT_BIN:-$HOME/opt/querencia/einstellungen/f7e7438b84fd5403/querencia-einstellungen}
NEU=${NEU_BIN:-$P/nachweis/015/release/querencia-einstellungen}
THEMA=${THEMA:-$HOME/opt/querencia-theme/22824d05ceb49cad/Querencia-Klassisch}
BILDER01=${BILDER01:-$HOME/projects/querencia-einstellungen-014/nachweis/bilder-01}
BILDER02=${BILDER02:-$HOME/projects/querencia-einstellungen-014/nachweis/bilder-02}
MUSTER=${1:-}
Z=szenarien/015
EN=$P/szenarien/010/daten-en
NW=nachweis/015
mkdir -p "$NW"
for f in "$ALT" "$NEU" "$THEMA/gtk-4.0/gtk-dark.css" "$Z/release-alpha21/release" "$Z/nur-version/version"; do [ -e "$f" ] || { echo "fehlt: $f" >&2; exit 2; }; done
[ ! -e /usr/share/querencia ] || { echo "/usr/share/querencia existiert auf diesem Rechner – „ohne“ wäre nicht ohne" >&2; exit 2; }
AB=$(date +%s)
GS=$P/szenarien/g2aj/gsettings-live.txt
{ cat "$GS"; echo "color-scheme default"; } > "$NW/gsettings-hell.txt"
{ cat "$GS"; echo "color-scheme prefer-dark"; } > "$NW/gsettings-dunkel.txt"
echo "alt $(sha256sum "$ALT" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$ALT")" | tee "$NW/gui-binaries.txt"
echo "neu $(sha256sum "$NEU" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$NEU")" | tee -a "$NW/gui-binaries.txt"
echo "thema $(readlink -f "$THEMA") gtk.css $(sha256sum "$THEMA/gtk-4.0/gtk.css" | cut -c1-16) gtk-dark.css $(sha256sum "$THEMA/gtk-4.0/gtk-dark.css" | cut -c1-16)" | tee -a "$NW/gui-binaries.txt"
(cd "$Z" && sha256sum release-alpha21/release release-alpha21/version nur-version/version) | tee -a "$NW/gui-binaries.txt"
werkzeug/produktiv-stand.sh > "$NW/produktiv-vorher.txt"
ls -l --time-style=full-iso ~/.local/share/applications > "$NW/starter-vorher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-vorher.txt"

lauf() {  # lauf <name> <bin> <hell|dunkel> <sprache> <aufl> <szenario> <ohne|ordner unter szenarien/015> [app-args]
    local name=$1 bin=$2 art=$3 sp=$4 aufl=$5 sz=$6 datei=$7 args=${8:--}
    [ -z "$MUSTER" ] || [[ $name == *$MUSTER* ]] || return 0
    local en=; [ "$sp" = en ] && en=$EN
    local wrap=()
    if [ "$datei" != ohne ]; then
        wrap=(bwrap --dev-bind / / --overlay-src /usr/share --tmp-overlay /usr/share
              --ro-bind "$P/$Z/$datei" /usr/share/querencia --tmpfs /tmp/.X11-unix)
    fi
    QE_PROFIL_STABIL=$HOME/.cache/qe01/einstellungen-015-profil QE_APP_BIN=$bin QE_PORTAL=settings QE_GSETTINGS=$NW/gsettings-$art.txt \
        QE_THEMA_ORDNER=$THEMA QE_LABWC_DATEIEN=$en QE_HINTERGRUENDE=$BILDER01:$BILDER02 QE_APP_ARGS=$args \
        "${wrap[@]}" bash werkzeug/g2aj-lauf.sh "$name" "$aufl" "$sz"
    echo "$datei" > "nachweis/$name/versionsdatei.txt"
    # Belege ohne dconf-Datenbank des Wegwerfprofils
    rm -rf "nachweis/$name/profil/heim/.config/dconf"
}
for k in de-1024 en-1920; do
    sp=${k%-*} b=${k#*-}
    s=$([ "$b" = 1920 ] && echo 1920x1080 || echo 1024x768)
    for art in hell dunkel; do
        lauf "015-d1-$art-alt-$k" "$ALT" $art $sp $s szenarien/014/d1-alle-seiten-galerie-$b.txt ohne
        lauf "015-d1-$art-neu-$k" "$NEU" $art $sp $s $Z/d1-alle-seiten-$b.txt ohne
        lauf "015-i1-$art-ohne-$k" "$NEU" $art $sp $s $Z/i1-info-$b.txt ohne "--seite informationen"
        lauf "015-i1-$art-mit-$k" "$NEU" $art $sp $s $Z/i1-info-$b.txt release-alpha21 "--seite informationen"
    done
done
lauf "015-i1-hell-nurversion-de-1024" "$NEU" hell de 1024x768 $Z/i1-info-1024.txt nur-version "--seite informationen"
lauf "015-n1-hell-neu-de-1024" "$NEU" hell de 1024x768 $Z/n1-netzwerk-1024.txt ohne --netzwerk
lauf "015-n1-dunkel-neu-en-1920" "$NEU" dunkel en 1920x1080 $Z/n1-netzwerk-1920.txt ohne --netzwerk
werkzeug/produktiv-stand.sh > "$NW/produktiv-nachher.txt"
ls -l --time-style=full-iso ~/.local/share/applications > "$NW/starter-nachher.txt"
# Starter ändern auch andere Sessions (Live-Aktivierungen); welche, steht hier.
diff "$NW/starter-vorher.txt" "$NW/starter-nachher.txt" > "$NW/starter-diff.txt" && echo "Starter unverändert" || { echo "Starter geändert (andere Sessions?):"; cat "$NW/starter-diff.txt"; }
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-nachher.txt"
if diff "$NW/produktiv-vorher.txt" "$NW/produktiv-nachher.txt" && diff "$NW/gsettings-live-vorher.txt" "$NW/gsettings-live-nachher.txt" && [ ! -e /usr/share/querencia ] &&
    cmp -s <(grep org.querencia.Einstellungen "$NW/starter-vorher.txt") <(grep org.querencia.Einstellungen "$NW/starter-nachher.txt"); then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
bash werkzeug/wsdd-reste.sh "$AB" | tee "$NW/wsdd-nach-gui.txt"
. "$HOME/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh"
python3 werkzeug/auswerten-015.py "$NW/gui-015.tsv"; echo "auswertung exit=$?"
