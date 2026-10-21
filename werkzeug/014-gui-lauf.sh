#!/bin/bash
# Einstellungen 0.1.4 (Hintergründe 0.2, Standard Tiefblau): isolierte
# GUI-Läufe 0.1.3 ↔ 0.1.4 wie werkzeug/013-gui-lauf.sh über
# werkzeug/g2aj-lauf.sh (eigenes HOME, XDG_RUNTIME_DIR, Bus und labwc –
# isoliert.sh; Settings-Portal-Ersatz color-scheme 0/1, gvfsd maskiert,
# GIO_USE_VFS=local). Thema: die Live-Ablage von Querencia-Klassisch 0.2.1
# (nur gelesen, fest auf die Ablage). Die eingebauten Bilder setzt
# QE_HINTERGRUENDE fest, unabhängig vom Live-Stand (Hintergründe 0.2 ist seit
# 21.10. 20:09 live): „ohne“ nur die 17 Dateien von 0.1 (BILDER01, Verweise
# auf data/hintergruende des Runtime-Worktrees, SHA256SUMS 0.1), „mit“ dazu
# die 32 von Hintergründe 0.2 (BILDER02, Verweise ins Datenrepo
# ~/projects/querencia-hintergruende, Tag hintergruende/0.2); im Profil
# liegen sie als Verweise wie live ohne Paket.
#   d1  alle Seiten und die Galerie gescrollt: de 1024×768 und en 1920×1080,
#       je hell und dunkel, je alt (0.1.3), neu-ohne und neu-mit
#   s1  Dialog „Hintergrund“ ab „Querencia Erde“, „Standard wiederherstellen“,
#       „Übernehmen“: de 1024 hell, alt, neu-ohne, neu-mit
#   e1  Seite „Hintergrundbild“ mit File= auf den Verweis
#       ~/.local/share/querencia/hintergruende/querencia-glas-4x3.png (wie
#       live ohne Paket): de 1024 hell, neu-mit
#   s2  Dialog „Hintergrund“ mit den neuen Bildern: de 1024 dunkel, en 1920
#       hell und dunkel (nur neu-mit)
# Läufe nach nachweis/014-<art>-<fassung>-<sprache>-<breite>/; Läufe
# nacheinander (QE_PROFIL_STABIL zeigt auf den letzten).
#
#   bash werkzeug/014-gui-lauf.sh [nur-läufe-mit-diesem-muster]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
ALT=${ALT_BIN:-$HOME/opt/querencia/einstellungen/950f769f17f47c02/querencia-einstellungen}
NEU=${NEU_BIN:-$P/nachweis/014/release/querencia-einstellungen}
THEMA=${THEMA:-$HOME/opt/querencia-theme/22824d05ceb49cad/Querencia-Klassisch}
BILDER01=${BILDER01:-$HOME/projects/querencia-einstellungen-014/nachweis/bilder-01}
BILDER02=${BILDER02:-$HOME/projects/querencia-einstellungen-014/nachweis/bilder-02}
MUSTER=${1:-}
Z=szenarien/014
EN=$P/szenarien/010/daten-en
NW=nachweis/014
mkdir -p "$NW"
for f in "$ALT" "$NEU" "$THEMA/gtk-4.0/gtk-dark.css"; do [ -e "$f" ] || { echo "fehlt: $f" >&2; exit 2; }; done
[ "$(find -L "$BILDER01" -maxdepth 1 -name 'querencia-*.png' -type f | wc -l)" = 17 ] || { echo "nicht 17 Bilder in $BILDER01" >&2; exit 2; }
[ "$(find -L "$BILDER02" -maxdepth 1 -name 'querencia-*.png' -type f | wc -l)" = 32 ] || { echo "nicht 32 Bilder in $BILDER02" >&2; exit 2; }
AB=$(date +%s)
GS=$P/szenarien/g2aj/gsettings-live.txt
{ cat "$GS"; echo "color-scheme default"; } > "$NW/gsettings-hell.txt"
{ cat "$GS"; echo "color-scheme prefer-dark"; } > "$NW/gsettings-dunkel.txt"
echo "alt $(sha256sum "$ALT" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$ALT")" | tee "$NW/gui-binaries.txt"
echo "neu $(sha256sum "$NEU" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$NEU")" | tee -a "$NW/gui-binaries.txt"
echo "thema $(readlink -f "$THEMA") gtk.css $(sha256sum "$THEMA/gtk-4.0/gtk.css" | cut -c1-16) gtk-dark.css $(sha256sum "$THEMA/gtk-4.0/gtk-dark.css" | cut -c1-16)" | tee -a "$NW/gui-binaries.txt"
(cd "$BILDER01" && for f in querencia-*.png; do printf '%s  %s\n' "$(sha256sum < "$f" | cut -d' ' -f1)" "$f"; done) > "$NW/bilder-01.sha256"
(cd "$BILDER02" && for f in querencia-*.png; do printf '%s  %s\n' "$(sha256sum < "$f" | cut -d' ' -f1)" "$f"; done) > "$NW/bilder-02.sha256"
echo "bilder-01 $(wc -l < "$NW/bilder-01.sha256") Dateien, Liste $(sha256sum "$NW/bilder-01.sha256" | cut -c1-16)" | tee -a "$NW/gui-binaries.txt"
echo "bilder-02 $(wc -l < "$NW/bilder-02.sha256") Dateien, Liste $(sha256sum "$NW/bilder-02.sha256" | cut -c1-16)" | tee -a "$NW/gui-binaries.txt"
werkzeug/produktiv-stand.sh > "$NW/produktiv-vorher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-vorher.txt"

lauf() {  # lauf <name> <bin> <hell|dunkel> <sprache> <aufl> <szenario> <mit|ohne> [app-args] [desktop.ini]
    local name=$1 bin=$2 art=$3 sp=$4 aufl=$5 sz=$6 bilder=$7 args=${8:--} ini=${9:-}
    [ -z "$MUSTER" ] || [[ $name == *$MUSTER* ]] || return 0
    local en=; [ "$sp" = en ] && en=$EN
    local hg=$BILDER01; [ "$bilder" = mit ] && hg=$BILDER01:$BILDER02
    QE_PROFIL_STABIL=$HOME/.cache/qe01/einstellungen-014-profil QE_APP_BIN=$bin QE_PORTAL=settings QE_GSETTINGS=$NW/gsettings-$art.txt \
        QE_THEMA_ORDNER=$THEMA QE_LABWC_DATEIEN=$en QE_HINTERGRUENDE=$hg QE_APP_ARGS=$args \
        bash werkzeug/g2aj-lauf.sh "$name" "$aufl" "$sz" $ini
    # Belege ohne dconf-Datenbank des Wegwerfprofils
    rm -rf "nachweis/$name/profil/heim/.config/dconf"
}
for k in de-1024 en-1920; do
    sp=${k%-*} b=${k#*-}
    s=$([ "$b" = 1920 ] && echo 1920x1080 || echo 1024x768)
    for art in hell dunkel; do
        lauf "014-$art-alt-$k" "$ALT" $art $sp $s $Z/d1-alle-seiten-galerie-$b.txt ohne
        lauf "014-$art-ohne-$k" "$NEU" $art $sp $s $Z/d1-alle-seiten-galerie-$b.txt ohne
        lauf "014-$art-mit-$k" "$NEU" $art $sp $s $Z/d1-alle-seiten-galerie-$b.txt mit
    done
done
lauf "014-standard-alt-de-1024" "$ALT" hell de 1024x768 $Z/s1-standard-1024.txt ohne --hintergrund $Z/ini-erde.ini
lauf "014-standard-ohne-de-1024" "$NEU" hell de 1024x768 $Z/s1-standard-1024.txt ohne --hintergrund $Z/ini-erde.ini
lauf "014-standard-mit-de-1024" "$NEU" hell de 1024x768 $Z/s1-standard-1024.txt mit --hintergrund $Z/ini-erde.ini
lauf "014-erkennen-mit-de-1024" "$NEU" hell de 1024x768 $Z/e1-erkennen-1024.txt mit - $Z/ini-glas-4x3.ini
lauf "014-dialog-dunkel-mit-de-1024" "$NEU" dunkel de 1024x768 $Z/s2-dialog-1024.txt mit --hintergrund
lauf "014-dialog-hell-mit-en-1920" "$NEU" hell en 1920x1080 $Z/s2-dialog-1920.txt mit --hintergrund
lauf "014-dialog-dunkel-mit-en-1920" "$NEU" dunkel en 1920x1080 $Z/s2-dialog-1920.txt mit --hintergrund
werkzeug/produktiv-stand.sh > "$NW/produktiv-nachher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-nachher.txt"
if diff "$NW/produktiv-vorher.txt" "$NW/produktiv-nachher.txt" && diff "$NW/gsettings-live-vorher.txt" "$NW/gsettings-live-nachher.txt"; then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
bash werkzeug/wsdd-reste.sh "$AB" | tee "$NW/wsdd-nach-gui.txt"
. "$HOME/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh"
python3 werkzeug/auswerten-014.py "$NW/gui-014.tsv"; echo "auswertung exit=$?"
