#!/bin/bash
# G2-C Schritt 6: Live-Kontrolle nach der Aktivierung, in der ECHTEN Sitzung
# (WAYLAND_DISPLAY=wayland-0, XDG_RUNTIME_DIR=/run/user/1001). Keine
# Eingaben in die Live-Sitzung (qe-eingabe bleibt gesperrt); stattdessen:
#  1. App über den Link mit --hintergrund starten (so ruft sie der Desktop-
#     Menüpunkt), Fenster per Bildschirmaufnahme erkennen, beenden;
#  2. Menüeintrag über GIO starten (gtk-launch, wie das Panel), erkennen, beenden;
#  3. desktop.ini byte-gleich mit dem, was die App bei Farbe / Zentrieren /
#     Kacheln / Standard schreibt (aus den isolierten Läufen r9–r11, r3),
#     atomar einspielen und das Desktop-Log beobachten;
#  4. Ausgangszustand: desktop.ini entfernen (vorher gab es keine).
# Von Bildschirmaufnahmen werden nur die Ausschnitte der App-Fenster
# gespeichert (der Rest des Bildschirms gehört dem Besitzer).
set -uo pipefail
P=~/projects/querencia-werkstatt-w6-einstellungen
N=$P/nachweis/live; mkdir -p "$N"; cd "$N"
S=/tmp/werkstatt
export XDG_RUNTIME_DIR=/run/user/1001 WAYLAND_DISPLAY=wayland-0 XDG_CURRENT_DESKTOP=X-Querencia LANG=en_US.UTF-8 LANGUAGE=de
. ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh
LOG=~/.local/state/querencia/desktop-live-kontrolle.txt
DLOG=~/.local/state/querencia/desktop-qe01.log
INI=~/.config/querencia/desktop.ini
ok=0; bad=0
pruef() { if eval "$2"; then ok=$((ok+1)); echo "PASS   $1"; else bad=$((bad+1)); echo "FAIL   $1"; fi; }
app_pids() { for p in /proc/[0-9]*; do e=$(readlink $p/exe 2>/dev/null) || continue; [[ "$e" == */querencia-einstellungen ]] && echo "${p#/proc/}"; done; }
aufnahme() { grim "$S/shot.png" && python3 - "$S/shot.png" "$1" "$2" <<'EOF'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("RGB")
art, ziel = sys.argv[2], sys.argv[3]
px = im.load()
if art == "dialog":
    zeilen = {}
    for y in range(140, 560):
        xs = [x for x in range(560, 1360) if px[x, y] == (64, 64, 64)]
        if len(xs) > 200: zeilen[y] = (min(xs), max(xs))
    r = None
    if len(zeilen) >= 2:
        t, b = min(zeilen), max(zeilen); l, rr = zeilen[t]; r = (l + 2, t + 2, rr - 2, b - 2)
    print("rahmen", r)
    im.crop((598, 145, 1322, 935)).save(ziel)
    sys.exit(0 if r and abs(r[0] - 748) < 40 and abs(r[1] - 217) < 40 else 1)
else:
    p1, p2 = im.getpixel((960, 540)), im.getpixel((700, 400))
    print("pixel", p1, p2)
    im.crop((522, 226, 1398, 858)).save(ziel)
    hell = lambda p: p != (28, 78, 150) and min(p) > 180
    sys.exit(0 if hell(p1) and hell(p2) else 1)
EOF
rc=$?; rm -f "$S/shot.png"; return $rc; }

echo "=== Ausgangsstand"
"$P/werkzeug/produktiv-stand.sh" | tee produktiv-vor-kontrolle.txt
pruef "keine desktop.ini, keine App läuft" "[ ! -e $INI ] && [ -z \"\$(app_pids)\" ]"
n0=$(wc -l <"$DLOG")

echo "=== 1 Desktop-Menüpunkt-Weg: Link --hintergrund"
pruef "QE-01-Desktop findet die App über ~/opt/querencia/bin (PATH der Sitzung enthält keine)" "[ -x ~/opt/querencia/bin/querencia-einstellungen ] && ! command -v querencia-einstellungen >/dev/null"
( cd ~ && ~/opt/querencia/bin/querencia-einstellungen --hintergrund >"$N/app-hintergrund.log" 2>&1 & )
sleep 4
aufnahme dialog "$N/01-hintergrund-dialog.png"; r=$?
pruef "Fenster „Querencia – Hintergrund“ offen (Vorschau-Rahmen wie isoliert)" "[ $r = 0 ]"
pruef "App erkennt den laufenden QE-01-Desktop: Farbe, Zentrieren, Kacheln true" "grep -q 'Farbe true, Zentrieren true, Kacheln true' app-hintergrund.log"
pruef "Programm der App ist die Ablage 2f44261d01fd2948" "for p in \$(app_pids); do readlink /proc/\$p/exe; done | grep -q '/opt/querencia/einstellungen/2f44261d01fd2948/querencia-einstellungen'"
for p in $(app_pids); do kill -TERM "$p"; done; sleep 1
pruef "App beendet, kein Absturz" "[ -z \"\$(app_pids)\" ] && ! grep -q 'panicked\|CRITICAL' app-hintergrund.log"

echo "=== 2 Menüeintrag-Weg: gtk-launch org.querencia.Einstellungen (GIO wie das Panel)"
( cd ~ && gtk-launch org.querencia.Einstellungen >"$N/app-menue.log" 2>&1 & )
sleep 4
aufnahme zentrum "$N/02-kontrollzentrum.png"; r=$?
pruef "Kontrollzentrum offen (Fenster in der Mitte, kein Hintergrund-Dialog)" "[ $r = 0 ]"
pruef "gestartet aus der Ablage" "for p in \$(app_pids); do readlink /proc/\$p/exe; done | grep -q '/opt/querencia/einstellungen/2f44261d01fd2948/querencia-einstellungen'"
for p in $(app_pids); do kill -TERM "$p"; done; sleep 1
pruef "App beendet, kein Absturz" "[ -z \"\$(app_pids)\" ] && ! grep -q 'panicked\|CRITICAL' app-menue.log"
pruef "Panel-Log: Systemmenü mit 1 Einstellung" "grep -q 'Systemmenü: Einstellungen 1' ~/.local/state/querencia/panel.log"

echo "=== 3 Hintergrund über desktop.ini (Inhalt byte-gleich mit der App-Ausgabe)"
schreibe() { printf '%s' "$2" >"$INI.neu" && mv -T "$INI.neu" "$INI"; sleep 2; echo "--- $1:"; cat "$INI"; tail -n +$((n0+1)) "$DLOG" | tail -3; }
schreibe farbe $'[Background]\nType=color\nFile=\nMode=zoom\nColor=#f07746\n'
pruef "Farbe: Desktop lädt die Datei neu (Log)" "tail -n +$((n0+1)) $DLOG | grep -q 'Hintergrund nur Farbe #f07746 (Type=color)'"
schreibe zentrieren "$(cat $P/nachweis/r10-zentrieren/desktop.ini.nachher)"$'\n'
pruef "Zentrieren: Desktop lädt die Datei neu (Log)" "tail -n +$((n0+1)) $DLOG | grep -q 'muster-4zu3.png, Modus centered'"
schreibe kacheln "$(cat $P/nachweis/r11-kacheln/desktop.ini.nachher)"$'\n'
pruef "Kacheln: Desktop lädt die Datei neu (Log)" "tail -n +$((n0+1)) $DLOG | grep -q 'kachel-160x120.png, Modus tiled'"
schreibe standard "$(cat $P/nachweis/r3-standard/desktop.ini.nachher)"$'\n'
pruef "Standard: Desktop zeigt wieder querencia-blau.png, zoom (Log)" "tail -n +$((n0+1)) $DLOG | tail -1 | grep -q 'querencia-blau.png, Modus zoom'"
cp "$INI" desktop.ini.standard-der-app

echo "=== 4 Ausgangszustand"
rm -f "$INI"; sleep 2
tail -n +$((n0+1)) "$DLOG" | tail -2
pruef "desktop.ini entfernt, Desktop wieder auf eingebautem Bild (Log)" "[ ! -e $INI ] && tail -n +$((n0+1)) $DLOG | tail -1 | grep -q 'querencia-blau.png, Modus zoom'"
"$P/werkzeug/produktiv-stand.sh" | tee produktiv-nach-kontrolle.txt
pruef "produktiver Stand (PIDs, Links, panel.ini, desktop-symbole) wie vor der Kontrolle" "cmp -s produktiv-vor-kontrolle.txt produktiv-nach-kontrolle.txt"
pruef "panel.ini 10cf47ae…, desktop-symbole ca932a93… unverändert" "sha256sum ~/.config/querencia/panel.ini | grep -q ^10cf47ae430f9832 && sha256sum ~/.local/state/querencia/desktop-symbole | grep -q ^ca932a9310d20cb8"
pruef "keine App läuft mehr, kein Fremdfenster hinterlassen" "[ -z \"\$(app_pids)\" ]"
tail -n +$((n0+1)) "$DLOG" > desktop-log-waehrend-kontrolle.txt
echo "ERGEBNIS live-kontrolle: $ok PASS, $bad FAIL"
[ "$bad" = 0 ]
