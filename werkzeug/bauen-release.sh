#!/bin/bash
# Release-Bau von Einstellungen ab 0.1.2: wie
# bisher über den Bau-Platz, dazu --remap-path-prefix, damit keine Pfade des
# Bau-Benutzers (Cargo-Registry, Worktree) im Binary stehen – Prüfung:
# strings target/release/querencia-einstellungen | grep -c /home/ → 0.
#   bash werkzeug/bauen-release.sh [weitere cargo-build-argumente]
set -euo pipefail
cd "$(dirname "$(dirname "$(readlink -f "$0")")")"
mem=$(awk '/MemAvailable/ {print int($2/1024)}' /proc/meminfo)
[ "$mem" -ge 2500 ] || { echo "bauen-release: nur $mem MB frei (mindestens 2500) – später" >&2; exit 3; }
BAU_PLATZ=${BAU_PLATZ:-$HOME/.local/state/querencia/bau-platz.sh}
export RUSTFLAGS="--remap-path-prefix=$HOME=~${RUSTFLAGS:+ $RUSTFLAGS}"
exec bash "$BAU_PLATZ" cargo build --release --offline --locked "$@"
