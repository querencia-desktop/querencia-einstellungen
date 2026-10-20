#!/bin/bash
# Einstellungen 0.1.2 (G1a): Wo findet das Programm die Vorgabe der
# labwc-Konfiguration? Prüft `--labwc-zeigen` (endet vor jedem GTK-Start)
# isoliert: leere Umgebung, Wegwerf-HOME/XDG_CONFIG_HOME/XDG_RUNTIME_DIR,
# kein WAYLAND_DISPLAY, kein Sitzungsbus. Die Paketlage /etc/querencia/labwc
# entsteht nur im eigenen Namensraum (bwrap, /etc als Wegwerf-Overlay).
#
#   bash werkzeug/vorgabe-pruefen.sh [binary] [binary-0.1.1]
# Mit der 0.1.1-Binary vergleicht Fall 6 die Ausgabe am Stand der
# Entwicklungs-VM (gleiche Vorgabe, gleiche Werte).
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
B=$(readlink -f "${1:-$P/nachweis/012/release/querencia-einstellungen}")
ALT=${2:-}
VQ=$P/szenarien/011/vorgabe-548483c
T=$(mktemp -d "$P/nachweis/vorgabe-pruefen-XXXXXX")
ok=0; bad=0
pruef() { if eval "$2"; then ok=$((ok+1)); echo "PASS   $1"; else bad=$((bad+1)); echo "FAIL   $1"; fi; }
# zeigen <fall> <bin> [VAR=wert …]: Ausgabe nach $T/<fall>.txt
zeigen() {
    local f=$1 bin=$2; shift 2
    mkdir -p "$T/$f/heim" "$T/$f/config" "$T/$f/laufzeit"; chmod 0700 "$T/$f/laufzeit"
    env -i HOME="$T/$f/heim" XDG_CONFIG_HOME="$T/$f/config" XDG_RUNTIME_DIR="$T/$f/laufzeit" \
        PATH=/usr/bin:/bin LANG=C.UTF-8 "$@" ${WRAP:-} "$bin" --labwc-zeigen >"$T/$f.txt" 2>&1
}
wert() { grep -m1 "^$2=" "$T/$1.txt" | cut -d= -f2-; }
kopie() {  # Benutzerkopie wie nach dem Sitzungsstarter, Stand mit quelle $2
    local u=$T/$1/config/querencia/labwc
    mkdir -p "$u/.vorgabe"; cp "$VQ/rc.xml" "$VQ/environment" "$u/"; cp "$VQ/rc.xml" "$VQ/environment" "$u/.vorgabe/"
    printf '# Vorgaben in .vorgabe/, zuletzt übernommen von querencia-session\nzeit 2026-10-21 08:00:00\nquelle %s (commit 548483c)\n' "$2" >"$u/.vorgabe/stand"
}
CHECKOUT=$T/checkout/session/querencia/labwc
mkdir -p "$CHECKOUT"; cp "$VQ/rc.xml" "$VQ/environment" "$CHECKOUT/"
PAKET=$T/paket; mkdir -p "$PAKET"; cp "$VQ/rc.xml" "$VQ/environment" "$PAKET/"
NR=$(grep -o 'number="[0-9]*"' "$VQ/rc.xml" | head -1 | tr -dc 0-9)
echo "Binary: $B $(sha256sum "$B" | cut -c1-16) $(bash "$P/werkzeug/version-isoliert.sh" "$B")"

echo "=== 1 Paket: keine Angabe, /etc/querencia/labwc aus dem Paket (bwrap)"
WRAP="bwrap --dev-bind / / --overlay-src /etc --tmp-overlay /etc --dir /etc/querencia/labwc --ro-bind $PAKET /etc/querencia/labwc" zeigen paket "$B"
pruef "Vorgabe /etc/querencia/labwc, Herkunft Paket" "[ \"\$(wert paket vorgabe)\" = /etc/querencia/labwc ] && [ \"\$(wert paket vorgabe-herkunft)\" = Paket ]"
pruef "ohne Benutzerkopie gilt die Paket-Vorgabe (quelle=Vorgabe, Arbeitsflächen $NR, kein Fehler)" "[ \"\$(wert paket quelle)\" = Vorgabe ] && grep -q \"^desktops/number=$NR\$\" $T/paket.txt && ! grep -q '^fehler=' $T/paket.txt"
WRAP="bwrap --dev-bind / / --overlay-src /etc --tmp-overlay /etc --dir /etc/querencia/labwc --ro-bind $PAKET /etc/querencia/labwc" zeigen paket-kopie "$B"
kopie paket-kopie /etc/querencia/labwc
WRAP="bwrap --dev-bind / / --overlay-src /etc --tmp-overlay /etc --dir /etc/querencia/labwc --ro-bind $PAKET /etc/querencia/labwc" zeigen paket-kopie "$B"
pruef "Paket mit Benutzerkopie (Stand quelle /etc/querencia/labwc): Herkunft Stand, quelle=Benutzer" "[ \"\$(wert paket-kopie vorgabe)\" = /etc/querencia/labwc ] && [ \"\$(wert paket-kopie vorgabe-herkunft)\" = Stand ] && [ \"\$(wert paket-kopie quelle)\" = Benutzer ]"
echo "=== 2 Paket nicht installiert: Vorgabe bleibt /etc/querencia/labwc, Programm meldet das ohne Absturz"
zeigen ohne "$B"
pruef "Herkunft Paket, Vorgabe fehlt → quelle=Keine, Ausgabe vollständig, kein Absturz" "[ \"\$(wert ohne vorgabe-herkunft)\" = Paket ] && [ \"\$(wert ohne quelle)\" = Keine ] && grep -q '^desktops/names=' $T/ohne.txt && ! grep -qi panicked $T/ohne.txt"
echo "=== 3 Entwicklungs-VM: Stand des Sitzungsstarters nennt den Runtime-Checkout"
kopie stand "$CHECKOUT"; zeigen stand "$B"
pruef "Vorgabe = Checkout aus .vorgabe/stand, Herkunft Stand" "[ \"\$(wert stand vorgabe)\" = $CHECKOUT ] && [ \"\$(wert stand vorgabe-herkunft)\" = Stand ] && [ \"\$(wert stand quelle)\" = Benutzer ]"
kopie stand-weg "$T/gibt-es-nicht"; zeigen stand-weg "$B"
pruef "Stand nennt einen Ordner ohne rc.xml: Paket" "[ \"\$(wert stand-weg vorgabe-herkunft)\" = Paket ]"
echo "=== 4 QUERENCIA_REPO (Checkout, gleicher Name wie im Starter)"
zeigen repo "$B" QUERENCIA_REPO="$T/checkout"
pruef "Vorgabe = \$QUERENCIA_REPO/session/querencia/labwc, Herkunft Repo, Werte aus der Vorgabe" "[ \"\$(wert repo vorgabe)\" = $CHECKOUT ] && [ \"\$(wert repo vorgabe-herkunft)\" = Repo ] && grep -q \"^desktops/number=$NR\$\" $T/repo.txt"
kopie repo-stand "$T/gibt-es-nicht"; mkdir -p "$T/anderes/session/querencia/labwc"; cp "$VQ/rc.xml" "$T/anderes/session/querencia/labwc/"
zeigen repo-stand "$B" QUERENCIA_REPO="$T/anderes"
pruef "QUERENCIA_REPO vor dem Stand" "[ \"\$(wert repo-stand vorgabe-herkunft)\" = Repo ] && [ \"\$(wert repo-stand vorgabe)\" = $T/anderes/session/querencia/labwc ]"
zeigen repo-rel "$B" QUERENCIA_REPO=checkout
pruef "relatives QUERENCIA_REPO zählt nicht" "[ \"\$(wert repo-rel vorgabe-herkunft)\" = Paket ]"
echo "=== 5 QE_LABWC_VORGABE (Prüfungen) vor allem anderen"
kopie umgebung "$CHECKOUT"; zeigen umgebung "$B" QE_LABWC_VORGABE="$PAKET" QUERENCIA_REPO="$T/anderes"
pruef "Herkunft Umgebung" "[ \"\$(wert umgebung vorgabe)\" = $PAKET ] && [ \"\$(wert umgebung vorgabe-herkunft)\" = Umgebung ]"
if [ -n "$ALT" ]; then
    echo "=== 6 Live-Stand der Entwicklungs-VM nachgebildet: echter .vorgabe/stand (nur gelesen), 0.1.1 und 0.1.2"
    L=${XDG_CONFIG_HOME:-$HOME/.config}/querencia/labwc
    for f in live-neu live-alt; do mkdir -p "$T/$f/config/querencia"; cp -a "$L" "$T/$f/config/querencia/"; done
    zeigen live-neu "$B"; zeigen live-alt "$(readlink -f "$ALT")"
    Q=$(sed -n 's/^quelle \(.*\) (commit .*/\1/p' "$L/.vorgabe/stand")
    pruef "0.1.2 nimmt die Quelle aus dem echten Stand ($Q), Herkunft Stand" "[ \"\$(wert live-neu vorgabe)\" = \"$Q\" ] && [ \"\$(wert live-neu vorgabe-herkunft)\" = Stand ]"
    pruef "0.1.2 und 0.1.1 lesen dieselbe Vorgabe und dieselben Werte (Ausgabe gleich bis auf vorgabe-herkunft und den Wegwerf-Pfad der Benutzerkopie)" "diff <(grep -v '^vorgabe-herkunft=\\|^benutzerkopie=' $T/live-neu.txt) <(grep -v '^benutzerkopie=' $T/live-alt.txt)"
fi
cp "$T"/*.txt "$P/nachweis/012/" 2>/dev/null; mkdir -p "$P/nachweis/012/vorgabe-faelle"; mv "$P"/nachweis/012/{paket,paket-kopie,ohne,stand,stand-weg,repo,repo-stand,repo-rel,umgebung,live-neu,live-alt}.txt "$P/nachweis/012/vorgabe-faelle/" 2>/dev/null
rm -rf "${T:?}"
echo "ERGEBNIS vorgabe-pruefen: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
