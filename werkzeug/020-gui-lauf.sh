#!/bin/bash
# Einstellungen 0.2.0: isolierte GUI-Läufe der Seite „Treiber“ über
# werkzeug/020-lauf.sh (eigenes HOME, XDG_RUNTIME_DIR, Bus, headless labwc;
# Attrappen für sysfs, ubuntu-drivers, dpkg-query, querencia-sitzung und
# PackageKit auf einem privaten Bus; ohne die QUERENCIA_*-Variablen der
# Live-Sitzung). Kein echter Treiberwechsel, kein Systembus.
#   p   Fotos: Fälle nvidia-nouveau, nvidia-original, amd, intel, keine,
#       werkzeug-fehlt × de/en × 1024×768/1920×1080, hell; nvidia-nouveau
#       zusätzlich dunkel
#   w1  frei → NVIDIA 615 (offen): Auswahl, Rückfrage, Fortschritt,
#       Neustart-Hinweis, Knopf „Neu starten …“ (Stub, QS_TROCKENLAUF)
#   w2  NVIDIA → frei: der ganze 615-Zweig wird entfernt
#   w3  polkit lehnt ab · w4 ubuntu-drivers-common installieren · w5 Paketsperre
#   w6  Auswahl zurücksetzen · w7 wie w1 auf Englisch
#   d1  Regression: alle Seiten alt (0.1.6-2, Live-Ablage) ↔ neu, de 1024 und
#       en 1920, hell
#
#   bash werkzeug/020-gui-lauf.sh [nur-läufe-mit-diesem-muster]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
ALT=${ALT_BIN:-$HOME/opt/querencia/einstellungen/1313696c2dcb7e2a/querencia-einstellungen}
NEU=${NEU_BIN:-$P/nachweis/020/release/querencia-einstellungen}
MUSTER=${1:-}
Z=szenarien/020
export NW=${NW:-nachweis/020}
mkdir -p "$NW"
for f in "$ALT" "$NEU"; do [ -e "$f" ] || { echo "fehlt: $f" >&2; exit 2; }; done
AB=$(date +%s)
echo "alt $(sha256sum "$ALT" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$ALT")" | tee "$NW/gui-binaries.txt"
echo "neu $(sha256sum "$NEU" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$NEU")" | tee -a "$NW/gui-binaries.txt"
werkzeug/produktiv-stand.sh > "$NW/produktiv-vorher.txt"
ls -l --time-style=full-iso ~/.local/share/applications > "$NW/starter-vorher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-vorher.txt"
ls -l --time-style=full-iso ~/.config/querencia/ > "$NW/konfig-live-vorher.txt" 2>&1

lauf() {  # lauf <name> <bin> <hell|dunkel> <sprache> <aufl> <fall> <szenario> [app-args]
    [ -z "$MUSTER" ] || [[ $1 == *$MUSTER* ]] || return 0
    bash werkzeug/020-lauf.sh "$@"
}
export QE_OUTPUTS=1
# Fotos
for fall in nvidia-nouveau nvidia-original amd intel keine werkzeug-fehlt; do
    for sp in de en; do for b in 1024x768 1920x1080; do
        lauf "020-p-$fall-hell-$sp-${b%x*}" "$NEU" hell $sp $b $fall "$Z/foto-${b%x*}.txt"
    done; done
done
for sp in de en; do lauf "020-p-nvidia-nouveau-dunkel-$sp-1920" "$NEU" dunkel $sp 1920x1080 nvidia-nouveau "$Z/foto-1920.txt"; done
# Funktionsläufe
lauf 020-w1-installieren-de-1920 "$NEU" hell de 1920x1080 nvidia-nouveau $Z/w1-installieren.txt
lauf 020-w2-entfernen-de-1920 "$NEU" hell de 1920x1080 nvidia-original $Z/w2-entfernen.txt
QE_PK_POLKIT=verweigern lauf 020-w3-polkit-de-1920 "$NEU" hell de 1920x1080 nvidia-nouveau $Z/w3-polkit.txt
lauf 020-w4-werkzeug-de-1920 "$NEU" hell de 1920x1080 werkzeug-fehlt $Z/w4-werkzeug.txt
QE_PK_SPERRE=1 lauf 020-w5-sperre-de-1920 "$NEU" hell de 1920x1080 nvidia-nouveau $Z/w3-polkit.txt
lauf 020-w6-zuruecksetzen-de-1920 "$NEU" hell de 1920x1080 nvidia-nouveau $Z/w6-zuruecksetzen.txt
lauf 020-w7-installieren-en-1920 "$NEU" hell en 1920x1080 nvidia-nouveau $Z/w7-installieren-en.txt
# Regression der übrigen Seiten (Szenarien wie 0.1.5/0.1.6; die neue Seite
# steht vor „Über Querencia“, das neue Szenario klickt dort eine Zeile tiefer)
for k in de-1024 en-1920; do
    sp=${k%-*} b=${k#*-}
    s=$([ "$b" = 1920 ] && echo 1920x1080 || echo 1024x768)
    lauf "020-d1-hell-alt-$k" "$ALT" hell $sp $s nvidia-nouveau szenarien/015/d1-alle-seiten-$b.txt -
    lauf "020-d1-hell-neu-$k" "$NEU" hell $sp $s nvidia-nouveau $Z/d1-alle-seiten-$b.txt -
done
werkzeug/produktiv-stand.sh > "$NW/produktiv-nachher.txt"
ls -l --time-style=full-iso ~/.local/share/applications > "$NW/starter-nachher.txt"
diff "$NW/starter-vorher.txt" "$NW/starter-nachher.txt" > "$NW/starter-diff.txt" && echo "Starter unverändert" || { echo "Starter geändert (andere Sessions?):"; cat "$NW/starter-diff.txt"; }
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-nachher.txt"
ls -l --time-style=full-iso ~/.config/querencia/ > "$NW/konfig-live-nachher.txt" 2>&1
if diff "$NW/produktiv-vorher.txt" "$NW/produktiv-nachher.txt" && diff "$NW/gsettings-live-vorher.txt" "$NW/gsettings-live-nachher.txt" &&
    diff "$NW/konfig-live-vorher.txt" "$NW/konfig-live-nachher.txt" &&
    cmp -s <(grep org.querencia.Einstellungen "$NW/starter-vorher.txt") <(grep org.querencia.Einstellungen "$NW/starter-nachher.txt"); then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
bash werkzeug/wsdd-reste.sh "$AB" | tee "$NW/wsdd-nach-gui.txt"
. "$HOME/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh"
python3 werkzeug/auswerten-020.py "$NW/gui-020.tsv"; echo "auswertung exit=$?"
