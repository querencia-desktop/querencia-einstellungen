#!/bin/bash
# Einstellungen 0.1.6 (QD-063), Fassung von werkzeug/vorgabe-pruefen.sh
# (0.1.2): Wo findet das Programm die Vorgabe der labwc-Konfiguration? Prüft
# `--labwc-zeigen` (endet vor jedem GTK-Start) isoliert: leere Umgebung,
# Wegwerf-HOME/XDG_CONFIG_HOME/XDG_RUNTIME_DIR, kein WAYLAND_DISPLAY, kein
# Sitzungsbus. /etc/querencia/labwc und /usr/share/querencia/labwc entstehen
# nur im eigenen Namensraum (bwrap, Wegwerf-Overlay); die Ablage ist eine
# Kopie der Binary in einem Wegwerfordner.
#
#   bash werkzeug/vorgabe-pruefen-016.sh [binary] [binary-0.1.5]
# Mit der 0.1.5-Binary vergleicht Fall 7 die Ausgabe am Stand der
# Entwicklungs-VM (gleiche Vorgabe, gleiche Werte).
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
B=$(readlink -f "${1:-$P/nachweis/016-2/release/querencia-einstellungen}")
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

ETC="--overlay-src /etc --tmp-overlay /etc --dir /etc/querencia/labwc --ro-bind $PAKET /etc/querencia/labwc"
USR="--overlay-src /usr/share --tmp-overlay /usr/share --dir /usr/share/querencia/labwc --ro-bind $PAKET /usr/share/querencia/labwc"
[ ! -e /etc/querencia ] && [ ! -e /usr/share/querencia ] || { echo "/etc/querencia oder /usr/share/querencia gibt es hier – „ohne“ wäre nicht ohne" >&2; exit 2; }

echo "=== 1 nichts installiert: keine Vorgabe, Hinweis statt Absturz"
zeigen ohne "$B"
pruef "Herkunft Keine, Vorgabe /usr/share/querencia/labwc, quelle=Keine, Ausgabe vollständig, kein Absturz" "[ \"\$(wert ohne vorgabe-herkunft)\" = Keine ] && [ \"\$(wert ohne vorgabe)\" = /usr/share/querencia/labwc ] && [ \"\$(wert ohne quelle)\" = Keine ] && grep -q '^desktops/names=' $T/ohne.txt && ! grep -qi panicked $T/ohne.txt"
echo "=== 2 /usr/share/querencia/labwc (System, bwrap)"
WRAP="bwrap --dev-bind / / $USR" zeigen system "$B"
pruef "Herkunft System, Werte aus der Vorgabe (Arbeitsflächen $NR)" "[ \"\$(wert system vorgabe)\" = /usr/share/querencia/labwc ] && [ \"\$(wert system vorgabe-herkunft)\" = System ] && [ \"\$(wert system quelle)\" = Vorgabe ] && grep -q \"^desktops/number=$NR\$\" $T/system.txt"
echo "=== 3 /etc/querencia/labwc vor /usr/share (Paket bis 0.1.5, Verwaltung)"
WRAP="bwrap --dev-bind / / $USR $ETC" zeigen paket "$B"
pruef "Herkunft Paket" "[ \"\$(wert paket vorgabe)\" = /etc/querencia/labwc ] && [ \"\$(wert paket vorgabe-herkunft)\" = Paket ]"
echo "=== 4 Ablage des Programms: <ablage>/share/querencia/labwc neben der Binary"
AB=$T/opt/querencia/einstellungen/0123456789abcdef; mkdir -p "$AB/share/querencia/labwc"; cp "$B" "$AB/"; cp "$VQ/rc.xml" "$VQ/environment" "$AB/share/querencia/labwc/"
WRAP="bwrap --dev-bind / / $USR $ETC" zeigen ablage "$AB/querencia-einstellungen"
pruef "Herkunft Ablage vor /etc und /usr/share" "[ \"\$(wert ablage vorgabe)\" = $AB/share/querencia/labwc ] && [ \"\$(wert ablage vorgabe-herkunft)\" = Ablage ]"
mkdir -p "$T/bin"; ln -s "$AB/querencia-einstellungen" "$T/bin/querencia-einstellungen"
zeigen ablage-link "$T/bin/querencia-einstellungen"
pruef "über einen Link wie ~/opt/querencia/bin (Link aufgelöst): Ablage" "[ \"\$(wert ablage-link vorgabe-herkunft)\" = Ablage ] && [ \"\$(wert ablage-link vorgabe)\" = $AB/share/querencia/labwc ]"
mkdir -p "$T/usr/bin" "$T/usr/share/querencia/labwc"; cp "$B" "$T/usr/bin/"; cp "$VQ/rc.xml" "$T/usr/share/querencia/labwc/"
zeigen praefix "$T/usr/bin/querencia-einstellungen"
pruef "Präfix: <präfix>/bin/.. /share/querencia/labwc" "[ \"\$(wert praefix vorgabe)\" = $T/usr/share/querencia/labwc ] && [ \"\$(wert praefix vorgabe-herkunft)\" = Ablage ]"
echo "=== 5 Entwicklungs-VM: Stand des Sitzungsstarters vor der Ablage"
kopie stand "$CHECKOUT"; zeigen stand "$AB/querencia-einstellungen"
pruef "Vorgabe = Checkout aus .vorgabe/stand, Herkunft Stand" "[ \"\$(wert stand vorgabe)\" = $CHECKOUT ] && [ \"\$(wert stand vorgabe-herkunft)\" = Stand ] && [ \"\$(wert stand quelle)\" = Benutzer ]"
kopie stand-weg "$T/gibt-es-nicht"; zeigen stand-weg "$B"
pruef "Stand nennt einen Ordner ohne rc.xml, sonst nichts: Keine" "[ \"\$(wert stand-weg vorgabe-herkunft)\" = Keine ]"
echo "=== 6 QUERENCIA_REPO und QE_LABWC_VORGABE vor allem"
zeigen repo "$AB/querencia-einstellungen" QUERENCIA_REPO="$T/checkout"
pruef "Herkunft Repo vor Ablage" "[ \"\$(wert repo vorgabe)\" = $CHECKOUT ] && [ \"\$(wert repo vorgabe-herkunft)\" = Repo ]"
zeigen repo-rel "$B" QUERENCIA_REPO=checkout
pruef "relatives QUERENCIA_REPO zählt nicht" "[ \"\$(wert repo-rel vorgabe-herkunft)\" = Keine ]"
kopie umgebung "$CHECKOUT"; zeigen umgebung "$AB/querencia-einstellungen" QE_LABWC_VORGABE="$PAKET" QUERENCIA_REPO="$T/anderes"
pruef "Herkunft Umgebung" "[ \"\$(wert umgebung vorgabe)\" = $PAKET ] && [ \"\$(wert umgebung vorgabe-herkunft)\" = Umgebung ]"
if [ -n "$ALT" ]; then
    echo "=== 7 Live-Stand der Entwicklungs-VM nachgebildet: echter .vorgabe/stand (nur gelesen), 0.1.5 und 0.1.6"
    L=${XDG_CONFIG_HOME:-$HOME/.config}/querencia/labwc
    for f in live-neu live-alt; do mkdir -p "$T/$f/config/querencia"; cp -a "$L" "$T/$f/config/querencia/"; done
    zeigen live-neu "$B"; zeigen live-alt "$(readlink -f "$ALT")"
    Q=$(sed -n 's/^quelle \(.*\) (commit .*/\1/p' "$L/.vorgabe/stand")
    pruef "0.1.6 nimmt die Quelle aus dem echten Stand ($Q), Herkunft Stand" "[ \"\$(wert live-neu vorgabe)\" = \"$Q\" ] && [ \"\$(wert live-neu vorgabe-herkunft)\" = Stand ]"
    pruef "0.1.6 und 0.1.5 lesen dieselbe Vorgabe und dieselben Werte (bis auf den Wegwerf-Pfad der Benutzerkopie)" "diff <(grep -v '^benutzerkopie=' $T/live-neu.txt) <(grep -v '^benutzerkopie=' $T/live-alt.txt)"
fi
mkdir -p "$P/${NW:-nachweis/016-2}/vorgabe-faelle"; cp "$T"/*.txt "$P/${NW:-nachweis/016-2}/vorgabe-faelle/" 2>/dev/null
sed -i "s#$T#<T>#g" "$P"/${NW:-nachweis/016-2}/vorgabe-faelle/*.txt
chmod -R u+w "$T"; rm -rf "${T:?}"
echo "ERGEBNIS vorgabe-pruefen-016: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
