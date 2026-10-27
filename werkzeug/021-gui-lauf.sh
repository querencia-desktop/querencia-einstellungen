#!/bin/bash
# Einstellungen 0.2.1: isolierte GUI-Läufe der Seite „Treiber“ (Zeile „Diese
# Sitzung“, Befund P-1 aus Alpha 3.4) über werkzeug/020-lauf.sh mit den
# Attrappen aus 0.2.0 (eigenes HOME, XDG_RUNTIME_DIR, Bus, headless labwc;
# sysfs, ubuntu-drivers, dpkg-query, querencia-sitzung, PackageKit auf
# privatem Bus; ohne die QUERENCIA_*- und Software-Variablen der Live-Sitzung).
# Fälle szenarien/021/faelle, Stubs szenarien/021/bin (querencia-sitzung
# liefert mit LIBGL_ALWAYS_SOFTWARE=1 renderer-software.txt).
#   p   Fotos: vm-llvmpipe, nvidia-nouveau (Software-Variablen gesetzt),
#       nvidia-nouveau-ohne-sw, amd, intel, nvidia-original, keine × de/en
#       1920×1080; vm-llvmpipe und nvidia-nouveau zusätzlich de 1024×768
#   h   0.2.1-2 (P-2): querencia-sitzung nur unter ~/opt/querencia/bin des
#       Profil-HOME, nicht im PATH (SITZUNG_IN_HEIM=1): nvidia-nouveau de,
#       vm-llvmpipe en, 1920×1080
#   d1  Regression: alle Seiten alt (0.2.0, Live-Ablage) ↔ neu, de 1024 und
#       en 1920
#
#   bash werkzeug/021-gui-lauf.sh [nur-läufe-mit-diesem-muster]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
ALT=${ALT_BIN:-$HOME/opt/querencia/einstellungen/bf03a73818fa2acb/querencia-einstellungen}
NEU=${NEU_BIN:-$P/nachweis/021/release/querencia-einstellungen}
MUSTER=${1:-}
export NW=${NW:-nachweis/021} FAELLE=szenarien/021/faelle STUBS=szenarien/021/bin
mkdir -p "$NW"
for f in "$ALT" "$NEU"; do [ -e "$f" ] || { echo "fehlt: $f" >&2; exit 2; }; done
AB=$(date +%s)
echo "alt $(sha256sum "$ALT" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$ALT")" | tee "$NW/gui-binaries.txt"
echo "neu $(sha256sum "$NEU" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$NEU")" | tee -a "$NW/gui-binaries.txt"
werkzeug/produktiv-stand.sh > "$NW/produktiv-vorher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-vorher.txt"
ls -l --time-style=full-iso ~/.config/querencia/ > "$NW/konfig-live-vorher.txt" 2>&1

lauf() {  # lauf <name> <bin> <hell|dunkel> <sprache> <aufl> <fall> <szenario> [app-args]
    [ -z "$MUSTER" ] || [[ $1 == *$MUSTER* ]] || return 0
    bash werkzeug/020-lauf.sh "$@"
}
export QE_OUTPUTS=1
for fall in vm-llvmpipe nvidia-nouveau nvidia-nouveau-ohne-sw amd intel nvidia-original keine; do
    for sp in de en; do
        lauf "021-p-$fall-hell-$sp-1920" "$NEU" hell $sp 1920x1080 $fall szenarien/020/foto-1920.txt
    done
done
for fall in vm-llvmpipe nvidia-nouveau; do
    lauf "021-p-$fall-hell-de-1024" "$NEU" hell de 1024x768 $fall szenarien/020/foto-1024.txt
done
SITZUNG_IN_HEIM=1 lauf 021-h-nvidia-nouveau-hell-de-1920 "$NEU" hell de 1920x1080 nvidia-nouveau szenarien/020/foto-1920.txt
SITZUNG_IN_HEIM=1 lauf 021-h-vm-llvmpipe-hell-en-1920 "$NEU" hell en 1920x1080 vm-llvmpipe szenarien/020/foto-1920.txt
for k in de-1024 en-1920; do
    sp=${k%-*} b=${k#*-}
    s=$([ "$b" = 1920 ] && echo 1920x1080 || echo 1024x768)
    lauf "021-d1-hell-alt-$k" "$ALT" hell $sp $s nvidia-nouveau szenarien/020/d1-alle-seiten-$b.txt -
    lauf "021-d1-hell-neu-$k" "$NEU" hell $sp $s nvidia-nouveau szenarien/020/d1-alle-seiten-$b.txt -
done
werkzeug/produktiv-stand.sh > "$NW/produktiv-nachher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-nachher.txt"
ls -l --time-style=full-iso ~/.config/querencia/ > "$NW/konfig-live-nachher.txt" 2>&1
if diff "$NW/produktiv-vorher.txt" "$NW/produktiv-nachher.txt" && diff "$NW/gsettings-live-vorher.txt" "$NW/gsettings-live-nachher.txt" &&
    diff "$NW/konfig-live-vorher.txt" "$NW/konfig-live-nachher.txt"; then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
bash werkzeug/wsdd-reste.sh "$AB" | tee "$NW/wsdd-nach-gui.txt"
. "$HOME/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh"
python3 werkzeug/auswerten-021.py "$NW/gui-021.tsv"; echo "auswertung exit=$?"
