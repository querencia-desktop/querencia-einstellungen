#!/bin/bash
# Einstellungen 0.2.0: privater Bus mit der PackageKit-Attrappe
# (pk-attrappe-020.py) für die isolierten Läufe der Seite „Treiber“. Nie der
# Systembus: der Bus hat eine eigene Konfiguration und einen eigenen Socket.
#
#   020-pk.sh start <zustand> <bestand.tsv>   → druckt QS_PK_BUS=…
#   020-pk.sh stop <zustand>
# Der Socket liegt unter ~/.cache/qe01 (Pfade ≤ 107 Zeichen, nicht /tmp).
set -euo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
ART=${1:?start|stop}; Z=$(readlink -f "${2:?zustand}")
case $ART in
start)
    mkdir -p "$Z"; cp "${3:?bestand.tsv}" "$Z/bestand.tsv"; : > "$Z/pk.log"
    mkdir -p "$HOME/.cache/qe01"; S=$(mktemp -d "$HOME/.cache/qe01/pk-XXXXXX")
    echo "$S" > "$Z/socket-ordner"
    cat > "$S/bus.conf" <<KONF
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <listen>unix:path=$S/s</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
KONF
    dbus-daemon --config-file="$S/bus.conf" --nofork --nopidfile > "$Z/bus.log" 2>&1 &
    echo $! > "$Z/bus.pid"
    for _ in $(seq 50); do [ -S "$S/s" ] && break; sleep 0.1; done
    ADR="unix:path=$S/s"
    QS_PK_BUS=$ADR python3 -I "$P/werkzeug/pk-attrappe-020.py" "$Z" > "$Z/attrappe.log" 2>&1 &
    echo $! > "$Z/attrappe.pid"
    for _ in $(seq 50); do grep -q bereit "$Z/attrappe.log" 2>/dev/null && break; sleep 0.1; done
    grep -q bereit "$Z/attrappe.log" || { echo "Attrappe startet nicht" >&2; cat "$Z/attrappe.log" >&2; exit 2; }
    echo "QS_PK_BUS=$ADR"
    ;;
stop)
    for f in attrappe bus; do
        [ -f "$Z/$f.pid" ] && kill "$(cat "$Z/$f.pid")" 2>/dev/null || true
    done
    sleep 0.3
    [ -f "$Z/socket-ordner" ] && rm -rf "$(cat "$Z/socket-ordner")"
    ;;
esac
