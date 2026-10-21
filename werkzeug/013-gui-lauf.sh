#!/bin/bash
# Einstellungen 0.1.3 (Kontrast hell, QD-089): isolierte GUI-Läufe 0.1.2 ↔
# 0.1.3 in der gekürzten Matrix, wie werkzeug/012-gui-lauf.sh über
# werkzeug/g2aj-lauf.sh (eigenes HOME, XDG_RUNTIME_DIR, Bus und labwc –
# isoliert.sh; Settings-Portal-Ersatz color-scheme 0/1, gvfsd maskiert,
# GIO_USE_VFS=local). Thema: die Live-Ablage von Querencia-Klassisch 0.2.1
# (nur gelesen, über QE_THEMA_ORDNER fest auf die Ablage, nicht auf den Link).
#   d1  alle Seiten: de 1024×768 und en 1920×1080, je hell und dunkel ab dem
#       Start, je alt (0.1.2) und neu (0.1.3)
#   w1  Wechsel im Lauf hell → dunkel → hell: de 1024×768, alt und neu
# Läufe nach nachweis/013-<art>-<alt|neu>-<sprache>-<breite>/; Läufe
# nacheinander (QE_PROFIL_STABIL zeigt auf den letzten).
#
#   bash werkzeug/013-gui-lauf.sh [nur-läufe-mit-diesem-muster]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
ALT=${ALT_BIN:-$HOME/opt/querencia/einstellungen/f822dd0ad28df2d5/querencia-einstellungen}
NEU=${NEU_BIN:-$P/nachweis/013/release/querencia-einstellungen}
THEMA=${THEMA:-$HOME/opt/querencia-theme/22824d05ceb49cad/Querencia-Klassisch}
MUSTER=${1:-}
Z=szenarien/012
EN=$P/szenarien/010/daten-en
NW=nachweis/013
mkdir -p "$NW"
for f in "$ALT" "$NEU" "$THEMA/gtk-4.0/gtk-dark.css"; do [ -e "$f" ] || { echo "fehlt: $f" >&2; exit 2; }; done
AB=$(date +%s)
GS=$P/szenarien/g2aj/gsettings-live.txt
{ cat "$GS"; echo "color-scheme default"; } > "$NW/gsettings-hell.txt"
{ cat "$GS"; echo "color-scheme prefer-dark"; } > "$NW/gsettings-dunkel.txt"
echo "alt $(sha256sum "$ALT" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$ALT")" | tee "$NW/gui-binaries.txt"
echo "neu $(sha256sum "$NEU" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$NEU")" | tee -a "$NW/gui-binaries.txt"
echo "thema $(readlink -f "$THEMA") gtk.css $(sha256sum "$THEMA/gtk-4.0/gtk.css" | cut -c1-16) gtk-dark.css $(sha256sum "$THEMA/gtk-4.0/gtk-dark.css" | cut -c1-16)" | tee -a "$NW/gui-binaries.txt"
werkzeug/produktiv-stand.sh > "$NW/produktiv-vorher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-vorher.txt"

lauf() {  # lauf <name> <bin> <hell|dunkel> <sprache> <aufl> <szenario>
    local name=$1 bin=$2 art=$3 sp=$4 aufl=$5 sz=$6
    [ -z "$MUSTER" ] || [[ $name == *$MUSTER* ]] || return 0
    local en=; [ "$sp" = en ] && en=$EN
    QE_PROFIL_STABIL=$HOME/.cache/qe01/einstellungen-013-profil QE_APP_BIN=$bin QE_PORTAL=settings QE_GSETTINGS=$NW/gsettings-$art.txt \
        QE_THEMA_ORDNER=$THEMA QE_LABWC_DATEIEN=$en \
        bash werkzeug/g2aj-lauf.sh "$name" "$aufl" "$sz"
    # Belege ohne dconf-Datenbank des Wegwerfprofils
    rm -rf "nachweis/$name/profil/heim/.config/dconf"
}
for k in de-1024 en-1920; do
    sp=${k%-*} b=${k#*-}
    s=$([ "$b" = 1920 ] && echo 1920x1080 || echo 1024x768)
    for art in hell dunkel; do
        lauf "013-$art-alt-$k" "$ALT" $art $sp $s $Z/d1-alle-seiten-$b.txt
        lauf "013-$art-neu-$k" "$NEU" $art $sp $s $Z/d1-alle-seiten-$b.txt
    done
done
lauf "013-wechsel-alt-de-1024" "$ALT" hell de 1024x768 $Z/w1-wechsel-1024.txt
lauf "013-wechsel-neu-de-1024" "$NEU" hell de 1024x768 $Z/w1-wechsel-1024.txt
werkzeug/produktiv-stand.sh > "$NW/produktiv-nachher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-nachher.txt"
if diff "$NW/produktiv-vorher.txt" "$NW/produktiv-nachher.txt" && diff "$NW/gsettings-live-vorher.txt" "$NW/gsettings-live-nachher.txt"; then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
bash werkzeug/wsdd-reste.sh "$AB" | tee "$NW/wsdd-nach-gui.txt"
. "$HOME/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh"
python3 werkzeug/auswerten-013.py "$NW/gui-013.tsv"; echo "auswertung exit=$?"
