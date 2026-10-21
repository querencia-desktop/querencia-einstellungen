#!/bin/bash
# QD-085 (13 §5): wsdd-Prozesse aus isolierten Läufen dieser Werkzeuge
# (XDG_RUNTIME_DIR unter ~/.cache/qe01/hmp05-runtime-*), die nach <epoch>
# gestartet sind. Nur lesen, über /proc. Exit 1, wenn es welche gibt.
#   bash werkzeug/wsdd-reste.sh [epoch]
set -uo pipefail
AB=${1:-0}
bt=$(awk '/btime/{print $2}' /proc/stat); hz=$(getconf CLK_TCK); n=0
for d in /proc/[0-9]*; do
    tr "\\0" " " 2>/dev/null <"$d/cmdline" | grep -q '/usr/bin/wsdd' || continue
    tr "\\0" "\\n" 2>/dev/null <"$d/environ" | grep -q "^XDG_RUNTIME_DIR=$HOME/.cache/qe01/hmp05-runtime-" || continue
    st=$(( bt + $(awk '{print $22}' "$d/stat") / hz ))
    [ "$st" -ge "$AB" ] || continue
    echo "wsdd-Rest PID ${d#/proc/} seit $(date -d "@$st" '+%F %T')"; n=$((n+1))
done
echo "wsdd-Reste seit $(date -d "@$AB" '+%F %T'): $n"
[ "$n" = 0 ]
