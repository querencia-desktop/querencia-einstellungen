#!/bin/bash
# Versionszeile eines Einstellungen-Binarys, ohne es in der Live-Umgebung
# aufzurufen (Regel der Status-Session vom 21.10.2026, 13 §5): leere
# Umgebung, Wegwerf-HOME und -XDG_RUNTIME_DIR, kein WAYLAND_DISPLAY/DISPLAY,
# kein Sitzungsbus. `--version` endet vor jedem GTK-Start (main.rs).
#   bash werkzeug/version-isoliert.sh <binary>
set -euo pipefail
B=$(readlink -f "${1:?Binary}")
T=$(mktemp -d)
mkdir -m 0700 "$T/laufzeit"
env -i HOME="$T" XDG_RUNTIME_DIR="$T/laufzeit" PATH=/usr/bin:/bin "$B" --version || true
rm -rf "${T:?}"
