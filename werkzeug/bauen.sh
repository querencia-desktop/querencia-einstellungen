#!/bin/bash
# Baut den Prototyp wie die anderen Worker: unter der gemeinsamen Build-Sperre,
# 2 Jobs, nice 10, eigenes target/, offline (keine Downloads).
set -euo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
exec flock ~/.local/state/querencia/bau.lock nice -n 10 env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR="$P/target" cargo build --offline --locked "$@"
