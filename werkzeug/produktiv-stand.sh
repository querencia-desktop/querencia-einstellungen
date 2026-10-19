#!/bin/bash
# Ausgangsstand der produktiven Sitzung (nur lesend): PIDs über /proc/*/exe, nicht über die Kommandozeile.
# G2-G: nur Prozesse der echten Sitzung (XDG_RUNTIME_DIR=/run/user/1001); isolierte
# Testsitzungen (eigene und anderer Worker) benutzen dieselben Binaries aus ~/opt.
echte() { tr '\0' '\n' <"$1/environ" 2>/dev/null | grep -qx 'XDG_RUNTIME_DIR=/run/user/1001'; }
pids() { for p in /proc/[0-9]*; do e=$(readlink "$p/exe" 2>/dev/null) || continue; [[ "$e" == $1 ]] && echte "$p" && echo "${p#/proc/}"; done | sort -n | tr '\n' ,; }
lw=""; for p in $(pgrep -x labwc); do echte /proc/$p && lw="$lw$p,"; done
echo "labwc_sitzung=$lw panel=$(pids '~/opt/querencia/*querencia-panel') desktop=$(pids '~/opt/querencia/*querencia-desktop') desktop_link=$(sha256sum "$(readlink -f ~/opt/querencia/bin/querencia-desktop)" | cut -c1-16) panel_link=$(sha256sum "$(readlink -f ~/opt/querencia/bin/querencia-panel)" | cut -c1-16)"
sha256sum ~/.config/querencia/panel.ini ~/.local/state/querencia/desktop-symbole ~/.config/querencia/desktop.ini 2>/dev/null | cut -c1-16 | tr '\n' ' '; ls ~/.config/querencia/ | tr '\n' ' '; echo
# G2-Z: Zuordnungen und Starter der echten Sitzung (nur gelesen)
echo "mimeapps=$(sha256sum ~/.config/mimeapps.list 2>/dev/null | cut -c1-16) x-querencia-mimeapps=$(ls ~/.config/*-mimeapps.list 2>/dev/null | tr '\n' ,) starter=$(ls -l --time-style=+%s ~/.local/share/applications | sha256sum | cut -c1-16)"
