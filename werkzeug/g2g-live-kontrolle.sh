#!/bin/bash
# G2-G: Live-Kontrolle von 0.0.2 in der ECHTEN Sitzung, ohne eine
# Monitorkonfiguration anzuwenden und ohne Eingaben (qe-eingabe bleibt auf
# Testsitzungen beschränkt):
#  1. --bildschirm öffnen (liest nur über wlr-output-management), Fenster per
#     grim-Ausschnitt prüfen, Ausgaben vorher/nachher gleich, keine bildschirm.ini;
#  2. --hintergrund öffnen (Hintergrund-Bereich weiter funktionsfähig, erkennt QE-01);
#  3. Kontrollzentrum über den Menüeintrag (gtk-launch, wie das Panel).
# Fenster werden per SIGTERM geschlossen. Gespeichert werden nur Ausschnitte der
# App-Fenster (labwc legt neue Fenster mittig an); das Vollbild wird sofort gelöscht.
set -uo pipefail
P=~/projects/querencia-werkstatt-w6-einstellungen
N=$P/nachweis/live-g2g; mkdir -p "$N"; cd "$N"
S=/tmp/werkstatt
export XDG_RUNTIME_DIR=/run/user/1001 WAYLAND_DISPLAY=wayland-0 XDG_CURRENT_DESKTOP=X-Querencia LANG=en_US.UTF-8 LANGUAGE=de
. ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh
APP=~/opt/querencia/bin/querencia-einstellungen
INI=~/.config/querencia/bildschirm.ini
ok=0; bad=0
pruef() { if eval "$2"; then ok=$((ok+1)); echo "PASS   $1"; else bad=$((bad+1)); echo "FAIL   $1"; fi; }
app_pids() { for p in /proc/[0-9]*; do e=$(readlink $p/exe 2>/dev/null) || continue; [[ "$e" == */querencia-einstellungen ]] && grep -qz '^XDG_RUNTIME_DIR=/run/user/1001$' $p/environ 2>/dev/null && echo "${p#/proc/}"; done; }
beenden() { for p in $(app_pids); do kill -TERM "$p"; done; sleep 1; }
# Ausschnitt x0 y0 x1 y1 ziel prüfart
aufnahme() { grim "$S/voll.png" && python3 - "$S/voll.png" "$@" <<'EOF'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("RGB")
x0, y0, x1, y1 = map(int, sys.argv[2:6]); ziel, art = sys.argv[6], sys.argv[7]
px = im.load()
if art == "hintergrund":
    zeilen = {}
    for y in range(140, 560):
        xs = [x for x in range(560, 1360) if px[x, y] == (64, 64, 64)]
        if len(xs) > 200: zeilen[y] = (min(xs), max(xs))
    r = None
    if len(zeilen) >= 2:
        t = min(zeilen); r = (zeilen[t][0] + 2, t + 2)
    print("rahmen", r)
    ok = bool(r and abs(r[0] - 748) < 40 and abs(r[1] - 217) < 40)
    if ok:
        im.crop((x0, y0, x1, y1)).save(ziel)
    sys.exit(0 if ok else 1)
# Fenster über seinen roten Schließen-Knopf (labwc-Theme Querencia-Klassisch)
# finden: helle Pixel allein beweisen nichts (am 18.10. war dort ein Fenster
# des Besitzers). Ausschnitt nur, wenn der Knopf gefunden ist; sonst nichts speichern.
breite, hoehe = x1 - x0, y1 - y0
reds = [(x, y) for y in range(120, 320) for x in range(1100, 1500)
        if px[x, y][0] > 180 and px[x, y][1] < 40 and px[x, y][2] < 40]
if not reds:
    print("kein Schließen-Knopf gefunden – Fenster nicht sichtbar")
    sys.exit(1)
rx = max(x for x, _ in reds); ty = min(y for _, y in reds)
L, T = rx - breite + 6, ty - 8
if abs(L - x0) > 40 or abs(T - y0) > 60:
    print("Knopf an unerwarteter Stelle", rx, ty)
    sys.exit(1)
im.crop((L, T, L + breite, min(im.height, T + hoehe))).save(ziel)
print("fenster", L, T)
sys.exit(0)
EOF
rc=$?; rm -f "$S/voll.png"; return $rc; }

echo "=== Ausgangsstand"
"$P/werkzeug/produktiv-stand.sh" | tee kontrolle2-produktiv-vor.txt
timeout 10 "$APP" --bildschirm-zeigen > ausgaben-vor.txt 2>/dev/null
cat ausgaben-vor.txt | cut -c1-120
pruef "0.0.2 über den Link, keine App offen, keine bildschirm.ini" "[ \"\$($APP --version)\" = 'querencia-einstellungen 0.0.2' ] && [ -z \"\$(app_pids)\" ] && [ ! -e $INI ]"
pruef "Ausgabe HDMI-A-1 an, 1920x1080@60000 bei 0,0, Skalierung 1" "grep -q '^HDMI-A-1 an modus=1920x1080@60000 position=0,0 drehung=0 skalierung=1 ' ausgaben-vor.txt"

echo "=== 1 Bereich Bildschirm (--bildschirm, nur lesen)"
( cd ~ && "$APP" --bildschirm > "$N/app-bildschirm.log" 2>&1 & )
sleep 7
aufnahme 572 195 1334 885 "$N/live-bildschirm.png" fenster; r=$?
pruef "Fenster „Querencia – Bildschirm“ offen" "[ $r = 0 ] && [ -n \"\$(app_pids)\" ]"
beenden
timeout 10 "$APP" --bildschirm-zeigen > ausgaben-nach-bildschirm.txt 2>/dev/null
pruef "nichts angewendet: Ausgaben unverändert, keine bildschirm.ini, Log ohne „angewendet“" "cmp -s ausgaben-vor.txt ausgaben-nach-bildschirm.txt && [ ! -e $INI ] && ! grep -q 'angewendet\|zurückgesetzt' app-bildschirm.log"
pruef "kein Absturz" "! grep -q 'panicked\|CRITICAL' app-bildschirm.log && [ -z \"\$(app_pids)\" ]"

echo "=== 2 Bereich Hintergrund (--hintergrund, wie der Desktop-Menüpunkt)"
( cd ~ && "$APP" --hintergrund > "$N/app-hintergrund.log" 2>&1 & )
sleep 5
aufnahme 598 145 1322 935 "$N/live-hintergrund.png" hintergrund; r=$?
pruef "Dialog „Querencia – Hintergrund“ offen (Vorschau-Rahmen wie isoliert)" "[ $r = 0 ]"
pruef "erkennt QE-01: Farbe, Zentrieren, Kacheln true" "grep -q 'Farbe true, Zentrieren true, Kacheln true' app-hintergrund.log"
beenden
pruef "kein Absturz, keine desktop.ini geschrieben" "! grep -q 'panicked\|CRITICAL' app-hintergrund.log && [ ! -e ~/.config/querencia/desktop.ini ] && [ -z \"\$(app_pids)\" ]"

echo "=== 3 Kontrollzentrum über den Menüeintrag (gtk-launch)"
( cd ~ && gtk-launch org.querencia.Einstellungen > "$N/app-menue.log" 2>&1 & )
sleep 6
aufnahme 522 230 1398 860 "$N/live-kontrollzentrum.png" fenster; r=$?
pruef "Kontrollzentrum offen, aus der Ablage 0.0.2" "[ $r = 0 ] && for p in \$(app_pids); do readlink /proc/\$p/exe; done | grep -q 03c6dec5e63966b9"
beenden
pruef "kein Absturz" "! grep -q 'panicked\|CRITICAL' app-menue.log && [ -z \"\$(app_pids)\" ]"

echo "=== Schluss"
"$P/werkzeug/produktiv-stand.sh" | tee kontrolle2-produktiv-nach.txt
timeout 10 "$APP" --bildschirm-zeigen > ausgaben-nach.txt 2>/dev/null
pruef "produktiver Stand und Ausgaben wie vorher" "cmp -s kontrolle2-produktiv-vor.txt kontrolle2-produktiv-nach.txt && cmp -s ausgaben-vor.txt ausgaben-nach.txt"
echo "ERGEBNIS live-kontrolle-g2g: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
