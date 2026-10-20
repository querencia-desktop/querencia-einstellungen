#!/bin/bash
# G2-AG: differenzielle Regression der Vorgänger-GUI-Prüfungen (G2-C, G2-G,
# G2-X, G2-Z, G2-AA). Die Klickpunkte dieser Szenarien wurden mit der Schrift
# „Adwaita Sans 11“ ermittelt; die ist seit 20.10. 01:02 nicht mehr installiert
# (Erscheinungsbild Variante A, DejaVu Sans 10). Deshalb läuft jedes Skript
# zweimal unter gleichen Bedingungen – mit dem Live-Stand (Ausgang) und mit
# der neuen Version (REG_ALT/REG_NEU) – und verglichen werden die Ergebnistabellen: gleiche
# Ergebnisse = keine Regression durch 0.0.7. Ergebnisse nach
# nachweis/g2ag/regression/<skript>-{006,007}/; danach werden die
# eingecheckten Nachweise der Vorgänger wiederhergestellt (git checkout).
#
#   bash werkzeug/g2ag-regression.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
# Ausgang (live) und neu; Standard seit 20.10. 03:37: 0.0.7 (G2-AF) gegen 0.0.8
ALT=${REG_ALT:-~/opt/querencia/einstellungen/a4b3f4fd8f1f0df5/querencia-einstellungen}
NEU=${REG_NEU:-$P/nachweis/g2ag/release/querencia-einstellungen}
VA=${REG_ALT_NAME:-007}; VN=${REG_NEU_NAME:-008}
R=nachweis/g2ag/regression
mkdir -p "$R"
TSVS="nachweis/g2aa/fenstersymbol-g2aa.tsv nachweis/g2c-direktstart.tsv nachweis/g2x/arbeitsflaeche-g2x.tsv nachweis/g2z/standardanwendungen-g2z.tsv nachweis/nachpruefung-release-g2c.tsv nachweis/pruefungen.tsv nachweis/regression-release-g2c.tsv"
TSVS="$TSVS ${REG_TSV_ZUSATZ:-}"
for s in ${REG_SKRIPTE:-g2c g2g g2x g2z g2aa}; do
    for v in $VA $VN; do
        bin=$ALT; [ $v = $VN ] && bin=$NEU
        rm -f $TSVS
        echo "=== $s mit $v ($(sha256sum "$bin" | cut -c1-16)) $(date +%T)"
        QE_APP_BIN=$bin bash werkzeug/$s-gui-lauf.sh > "$R/$s-$v.log" 2>&1
        echo "exit=$?" >> "$R/$s-$v.log"
        mkdir -p "$R/$s-$v"
        for t in $TSVS; do [ -f "$t" ] && cp "$t" "$R/$s-$v/"; done
        tail -3 "$R/$s-$v.log"
    done
    # Vergleich: Fall und Ergebnis (ohne Zeit-/Pfadspalten)
    for t in "$R/$s-$VA"/*.tsv; do
        [ -f "$t" ] || { grep -h 'ERGEBNIS\|PASS-Zähler' "$R/$s-$VA.log" "$R/$s-$VN.log" | sort | uniq -c | sed "s#^#VERGLEICH LOG $s: #"; continue; }
        n=$(basename "$t")
        if diff <(cut -f1-3 "$t") <(cut -f1-3 "$R/$s-$VN/$n") > "$R/$s-diff-$n.txt"; then
            echo "GLEICH $s $n ($(grep -c PASS "$t") PASS-Zeilen)"
        else
            echo "UNTERSCHIED $s $n (siehe $R/$s-diff-$n.txt)"
        fi
    done
done
git checkout -- nachweis/ 2>/dev/null
git status --short nachweis | head
