#!/bin/bash
# G2-AF: Fähigkeiten-Erkennung der Einstellungen gegen drei Desktops im
# isolierten labwc (logs/faehigkeiten.log aus isoliert.sh):
#   f4879567 QE-04a (Debug, live bis zur nächsten Anmeldung) → alles an
#   9d622162 G2-AD (Release, ab nächster Anmeldung)            → alles an
#   2b46a7b9 HMP-10a (ohne --faehigkeiten)                     → alles aus
#   QE_APP_BIN=<release> bash werkzeug/g2af-faehigkeiten.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
export QE_APP_BIN=${QE_APP_BIN:-$P/nachweis/g2af/release/querencia-einstellungen}
AUS=nachweis/g2af/faehigkeiten.tsv
echo -e "desktop\terwartet\tlog\turteil" >"$AUS"
ok=0; bad=0
for paar in f48795676f867b82:an 9d622162efd66f93:an 2b46a7b90b3412bb:aus; do
    d=${paar%%:*}; soll=${paar#*:}
    QE_DESKTOP=$HOME/opt/querencia/desktop/$d/querencia-desktop QE_DESKTOP_SHA16=$d QE_APP_ARGS= \
        bash werkzeug/isoliert.sh "g2af-faehigkeiten-$d" 1024x768 szenarien/hintergruende/l-desktop.txt >/dev/null 2>&1
    log=$(grep -m1 'Farbe' "nachweis/g2af-faehigkeiten-$d/logs/faehigkeiten.log" | sed 's/.* – //')
    if [ "$soll" = an ]; then muster='^Farbe true, Zentrieren true, Kacheln true, Laufwerkstypen true, Symbole true, Symbolgröße true$'
    else muster='^Farbe false, Zentrieren false, Kacheln false, Laufwerkstypen false, Symbole false, Symbolgröße false$'; fi
    if echo "$log" | grep -q "$muster"; then u=PASS; ok=$((ok+1)); else u=FAIL; bad=$((bad+1)); fi
    echo -e "$d\t$soll\t$log\t$u" >>"$AUS"; echo "$u $d ($soll): $log"
done
echo "$ok/$((ok+bad)) PASS"
[ "$bad" = 0 ]
