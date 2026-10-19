#!/bin/bash
# Baut qe-eingabe (Protokolle per wayland-scanner wie das bauen.sh der Werkstatt).
set -euo pipefail
HIER="$(cd "$(dirname "$0")" && pwd)"; OUT="$HIER/bin"; mkdir -p "$OUT"
VP=$(ls -d ~/.cargo/registry/src/*/wayland-protocols-wlr-0.3.*/wlr-protocols/unstable/wlr-virtual-pointer-unstable-v1.xml | head -1)
for x in "$VP" "$HIER/virtual-keyboard-unstable-v1.xml"; do b=$(basename "$x" .xml)
  wayland-scanner client-header "$x" "$OUT/$b-client-protocol.h"; wayland-scanner private-code "$x" "$OUT/$b-protocol.c"; done
gcc -O2 -Wall -Wextra -I"$OUT" -o "$OUT/qe-eingabe" "$HIER/qe-eingabe.c" "$OUT"/virtual-*-protocol.c "$OUT"/wlr-virtual-*-protocol.c $(pkg-config --cflags --libs wayland-client xkbcommon)
echo "gebaut: $OUT/qe-eingabe"
