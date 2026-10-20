#!/bin/bash
# G2-AG: differenzielle Regression der Vorgänger-GUI-Prüfungen (G2-C, G2-G,
# G2-X, G2-Z, G2-AA). Die Klickpunkte dieser Szenarien wurden mit der Schrift
# „Adwaita Sans 11“ ermittelt; die ist seit 20.10. 01:02 nicht mehr installiert
# (Erscheinungsbild Variante A, DejaVu Sans 10). Deshalb läuft jedes Skript
# zweimal unter gleichen Bedingungen – mit 0.0.6 (Live-Ablage, Ausgangsstand)
# und mit 0.0.7 – und verglichen werden die Ergebnistabellen: gleiche
# Ergebnisse = keine Regression durch 0.0.7. Ergebnisse nach
# nachweis/g2ag/regression/<skript>-{006,007}/; danach werden die
# eingecheckten Nachweise der Vorgänger wiederhergestellt (git checkout).
#
#   bash werkzeug/g2ag-regression.sh
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
ALT=~/opt/querencia/einstellungen/05ac8f96c28e4434/querencia-einstellungen
NEU=$P/nachweis/g2ag/release/querencia-einstellungen
R=nachweis/g2ag/regression
mkdir -p "$R"
TSVS="nachweis/g2aa/fenstersymbol-g2aa.tsv nachweis/g2c-direktstart.tsv nachweis/g2x/arbeitsflaeche-g2x.tsv nachweis/g2z/standardanwendungen-g2z.tsv nachweis/nachpruefung-release-g2c.tsv nachweis/pruefungen.tsv nachweis/regression-release-g2c.tsv"
for s in g2c g2g g2x g2z g2aa; do
    for v in 006 007; do
        bin=$ALT; [ $v = 007 ] && bin=$NEU
        rm -f $TSVS
        echo "=== $s mit $v ($(sha256sum "$bin" | cut -c1-16)) $(date +%T)"
        QE_APP_BIN=$bin bash werkzeug/$s-gui-lauf.sh > "$R/$s-$v.log" 2>&1
        echo "exit=$?" >> "$R/$s-$v.log"
        mkdir -p "$R/$s-$v"
        for t in $TSVS; do [ -f "$t" ] && cp "$t" "$R/$s-$v/"; done
        tail -3 "$R/$s-$v.log"
    done
    # Vergleich: Fall und Ergebnis (ohne Zeit-/Pfadspalten)
    for t in "$R/$s-006"/*.tsv; do
        n=$(basename "$t")
        if diff <(cut -f1-3 "$t") <(cut -f1-3 "$R/$s-007/$n") > "$R/$s-diff-$n.txt"; then
            echo "GLEICH $s $n ($(grep -c PASS "$t") PASS-Zeilen)"
        else
            echo "UNTERSCHIED $s $n (siehe $R/$s-diff-$n.txt)"
        fi
    done
done
git checkout -- nachweis/ 2>/dev/null
git status --short nachweis | head
