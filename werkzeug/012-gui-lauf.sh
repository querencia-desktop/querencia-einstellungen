#!/bin/bash
# Einstellungen 0.1.2 (CSS-Welle dunkel): isolierte GUI-Läufe mit dem
# Settings-Portal-Ersatz (color-scheme 0 = default, 1 = prefer-dark) über
# werkzeug/g2aj-lauf.sh (eigenes HOME, XDG_RUNTIME_DIR, Bus und labwc –
# isoliert.sh). Je Breite 1920/1024 und Sprache de/en:
#   t01  installiertes Querencia-Klassisch (live, ohne dunkle Fassung):
#        hell und dunkel, je 0.1.1 und 0.1.2 – muss bildgleich sein
#   t02  Thema 0.2 (QE_THEMA_ORDNER, Thema-Worktree, nur gelesen):
#        hell (0.1.1/0.1.2: gleich bis auf den Hinweis), dunkel ab dem Start
#        (0.1.1 bleibt hell, QD-081; 0.1.2 dunkel), Wechsel im Lauf (0.1.2,
#        in 1920 de zum Vergleich auch 0.1.1)
# Läufe nach nachweis/012-<thema>-<art>-<alt|neu>-<sprache>-<breite>/.
#
#   bash werkzeug/012-gui-lauf.sh [nur-läufe-mit-diesem-muster]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
ALT=${ALT_BIN:-$HOME/projects/querencia-einstellungen-wt-011/nachweis/011/release/querencia-einstellungen}
NEU=${NEU_BIN:-$P/nachweis/012/release/querencia-einstellungen}
THEMA02=${THEMA02:-$HOME/projects/querencia-thema-wt-02/Querencia-Klassisch}
MUSTER=${1:-}
Z=szenarien/012
EN=$P/szenarien/010/daten-en
NW=nachweis/012
mkdir -p "$NW"
for f in "$ALT" "$NEU" "$THEMA02/gtk-4.0/gtk-dark.css"; do [ -e "$f" ] || { echo "fehlt: $f" >&2; exit 2; }; done
GS=$P/szenarien/g2aj/gsettings-live.txt
{ cat "$GS"; echo "color-scheme default"; } > "$NW/gsettings-hell.txt"
{ cat "$GS"; echo "color-scheme prefer-dark"; } > "$NW/gsettings-dunkel.txt"
echo "alt $(sha256sum "$ALT" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$ALT")" | tee "$NW/gui-binaries.txt"
echo "neu $(sha256sum "$NEU" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$NEU")" | tee -a "$NW/gui-binaries.txt"
echo "thema02 $(readlink -f "$THEMA02") gtk-dark.css $(sha256sum "$THEMA02/gtk-4.0/gtk-dark.css" | cut -c1-16)" | tee -a "$NW/gui-binaries.txt"
werkzeug/produktiv-stand.sh > "$NW/produktiv-vorher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-vorher.txt"

lauf() {  # lauf <name> <bin> <thema|-> <hell|dunkel> <sprache> <aufl> <szenario>
    local name=$1 bin=$2 thema=$3 art=$4 sp=$5 aufl=$6 sz=$7
    [ -z "$MUSTER" ] || [[ $name == *$MUSTER* ]] || return 0
    local en=; [ "$sp" = en ] && en=$EN
    QE_PROFIL_STABIL=$HOME/.cache/qe01/einstellungen-012-profil QE_APP_BIN=$bin QE_PORTAL=settings QE_GSETTINGS=$NW/gsettings-$art.txt \
        QE_THEMA_ORDNER=$([ "$thema" = - ] || echo "$thema") QE_LABWC_DATEIEN=$en \
        bash werkzeug/g2aj-lauf.sh "$name" "$aufl" "$sz"
}
for b in 1920 1024; do
    s=$([ $b = 1920 ] && echo 1920x1080 || echo 1024x768)
    for sp in de en; do
        for v in alt neu; do
            bin=$ALT; [ $v = neu ] && bin=$NEU
            lauf "012-t01-hell-$v-$sp-$b" "$bin" - hell $sp $s $Z/d1-alle-seiten-$b.txt
            lauf "012-t01-dunkel-$v-$sp-$b" "$bin" - dunkel $sp $s $Z/d1-alle-seiten-$b.txt
            lauf "012-t02-hell-$v-$sp-$b" "$bin" "$THEMA02" hell $sp $s $Z/d1-alle-seiten-$b.txt
            lauf "012-t02-dunkel-$v-$sp-$b" "$bin" "$THEMA02" dunkel $sp $s $Z/d1-alle-seiten-$b.txt
        done
        lauf "012-t02-wechsel-neu-$sp-$b" "$NEU" "$THEMA02" hell $sp $s $Z/w1-wechsel-$b.txt
    done
done
lauf "012-t02-wechsel-alt-de-1920" "$ALT" "$THEMA02" hell de 1920x1080 $Z/w1-wechsel-1920.txt
werkzeug/produktiv-stand.sh > "$NW/produktiv-nachher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-nachher.txt"
if diff "$NW/produktiv-vorher.txt" "$NW/produktiv-nachher.txt" && diff "$NW/gsettings-live-vorher.txt" "$NW/gsettings-live-nachher.txt"; then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
. "$HOME/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh"
python3 werkzeug/auswerten-012.py "$NW/gui-012.tsv"; echo "auswertung exit=$?"
