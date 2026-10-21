#!/bin/bash
# cargo test isoliert (Projektregel vom 20.10.2026): eigenes
# HOME, XDG_CONFIG_HOME/DATA_HOME/CACHE_HOME/RUNTIME_DIR in einem Wegwerfordner,
# eigener Session-Bus (dbus-run-session), ohne WAYLAND_DISPLAY und DISPLAY –
# kein Test erreicht die Live-Sitzung, dconf oder den Papierkorb. Cargo und
# rustup bleiben über CARGO_HOME/RUSTUP_HOME beim echten Benutzer (nur lesend,
# gebaut wird in target/ des Worktrees).
#
#   bash werkzeug/test-isoliert.sh [cargo-test-argumente]
set -euo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
mkdir -p nachweis/profile
T=$(mktemp -d "$P/nachweis/profile/test-XXXXXX")
mkdir -p "$T/heim" "$T/konfig" "$T/daten" "$T/cache" "$T/laufzeit"
chmod 0700 "$T/laufzeit"
ECHT_HOME=$HOME
rc=0
env -u WAYLAND_DISPLAY -u DISPLAY -u DBUS_SESSION_BUS_ADDRESS \
    HOME="$T/heim" XDG_CONFIG_HOME="$T/konfig" XDG_DATA_HOME="$T/daten" \
    XDG_CACHE_HOME="$T/cache" XDG_RUNTIME_DIR="$T/laufzeit" \
    CARGO_HOME="${CARGO_HOME:-$ECHT_HOME/.cargo}" RUSTUP_HOME="${RUSTUP_HOME:-$ECHT_HOME/.rustup}" \
    PATH="$ECHT_HOME/.cargo/bin:$PATH" ECHT_HOME="$ECHT_HOME" ECHT_UID="$(id -u)" \
    dbus-run-session -- bash -c '
        [ "$XDG_RUNTIME_DIR" != "/run/user/$ECHT_UID" ] && [ "$HOME" != "$ECHT_HOME" ] && [ -z "${WAYLAND_DISPLAY:-}" ] ||
            { echo "test-isoliert: nicht isoliert – Abbruch" >&2; exit 2; }
        echo "test-isoliert: HOME=$HOME XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR"
        exec cargo test --offline --locked "$@"' _ "$@" || rc=$?
rm -rf "$T"
exit $rc
