#!/bin/bash
# Einstellungen 0.1.6: isolierte GUI-Läufe über werkzeug/016-lauf.sh (eigenes
# HOME, XDG_RUNTIME_DIR, Bus, headless labwc mit virtuellen Ausgängen; ohne
# die QUERENCIA_*-Variablen der Live-Sitzung). Die Ausgänge der laufenden
# Sitzung werden nie berührt: alles Umschalten geschieht im headless labwc.
#   f1–f5  Bereich Bildschirm im eigenen Fenster (--bildschirm), zwei
#          Ausgänge (HEADLESS-1 1920×1080 rechts, HEADLESS-2 1280×720 bei 0,0):
#          f1 Auflösung, Anwenden, Countdown läuft ab → zurück, nichts
#             gespeichert (en, dunkel)
#          f2 Auflösung, Anwenden, „Beibehalten“ → bildschirm.toml
#          f3 Skalierung 1,25 · f4 Als Hauptmonitor · f5 Anordnung ziehen
#   k1     dasselbe im Kontrollzentrum (--seite bildschirm, gerollt), Skalierung 2
#   f6–f8  --bildschirm-anwenden: mit bildschirm.toml, nur mit der alten
#          bildschirm.ini, ohne Datei
#   f9     Hinweis „keine EDID“ (Prüfschalter QE_BILDSCHIRM_OHNE_EDID)
#   p      Fotos der Seite: de/en × 1024×768/1920×1080 × hell/dunkel; das
#          Fenster liegt auf HEADLESS-2, das --bildschirm-anwenden vorher auf
#          die Fotogröße stellt
#   d1     Regression mit den 0.1.5-Szenarien: alle Seiten alt (0.1.5, live
#          Ablage) ↔ neu, de 1024 und en 1920, hell und dunkel
#   r      (0.1.6-2) Fensterrahmen folgt „Dunkle Darstellung bevorzugen“:
#          Ausgangslage heller Querencia-Rahmen, dunkler, fremder Rahmen,
#          ohne installiertes dunkles Thema; je umschalten (Übernehmen,
#          labwc --reconfigure des isolierten labwc), Fensterdesign ansehen,
#          zurückstellen. Thema-Ablage 519997cf (Thema 0.2.2 mit dem
#          dunklen labwc-Rahmen). Eine fremde rc.xml.vorher-* liegt bereit.
#
#   bash werkzeug/016-gui-lauf.sh [nur-läufe-mit-diesem-muster]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
ALT=${ALT_BIN:-$HOME/opt/querencia/einstellungen/3069d052f00d30a2/querencia-einstellungen}
NEU=${NEU_BIN:-$P/nachweis/016-2/release/querencia-einstellungen}
MUSTER=${1:-}
Z=szenarien/016
export NW=${NW:-nachweis/016-2}
mkdir -p "$NW"
for f in "$ALT" "$NEU" "$Z/foto-1024.toml" szenarien/015/d1-alle-seiten-1024.txt; do [ -e "$f" ] || { echo "fehlt: $f" >&2; exit 2; }; done
AB=$(date +%s)
echo "alt $(sha256sum "$ALT" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$ALT")" | tee "$NW/gui-binaries.txt"
echo "neu $(sha256sum "$NEU" | cut -c1-16) $(bash werkzeug/version-isoliert.sh "$NEU")" | tee -a "$NW/gui-binaries.txt"
werkzeug/produktiv-stand.sh > "$NW/produktiv-vorher.txt"
ls -l --time-style=full-iso ~/.local/share/applications > "$NW/starter-vorher.txt"
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-vorher.txt"
ls -l --time-style=full-iso ~/.config/querencia/ > "$NW/konfig-live-vorher.txt" 2>&1

lauf() {  # lauf <name> <bin> <hell|dunkel> <sprache> <aufl> <szenario> [app-args]
    [ -z "$MUSTER" ] || [[ $1 == *$MUSTER* ]] || return 0
    bash werkzeug/016-lauf.sh "$@"
}
# Funktionsläufe (zwei Ausgänge)
export QE_OUTPUTS=2
lauf 016-f1-rueckfall-en-1920 "$NEU" dunkel en 1920x1080 $Z/f1-rueckfall.txt --bildschirm
for f in f2-beibehalten f3-skalierung f4-hauptmonitor f5-ziehen; do
    lauf "016-$f-de-1920" "$NEU" hell de 1920x1080 "$Z/$f.txt" --bildschirm
done
lauf 016-k1-kontrollzentrum-de-1920 "$NEU" hell de 1920x1080 $Z/k1-kontrollzentrum.txt "--seite bildschirm"
QE_BILDSCHIRM_TOML=$Z/f6-anwenden.toml lauf 016-f6-anwenden-toml-de-1920 "$NEU" hell de 1920x1080 $Z/f6-anwenden.txt --bildschirm-anwenden
QE_BILDSCHIRM_INI=$Z/f7-anwenden.ini lauf 016-f7-anwenden-ini-de-1920 "$NEU" hell de 1920x1080 $Z/f6-anwenden.txt --bildschirm-anwenden
lauf 016-f8-ohne-datei-de-1920 "$NEU" hell de 1920x1080 $Z/f8-ohne-datei.txt --bildschirm-anwenden
QE_BILDSCHIRM_OHNE_EDID=HEADLESS-1,HEADLESS-2 QE_BILDSCHIRM_TOML=$Z/foto-1024.toml \
    lauf 016-f9-edid-hell-de-1024 "$NEU" hell de 1920x1080 $Z/foto-1024.txt --bildschirm-anwenden
QE_BILDSCHIRM_OHNE_EDID=HEADLESS-1,HEADLESS-2 QE_BILDSCHIRM_TOML=$Z/foto-1920.toml \
    lauf 016-f9-edid-dunkel-en-1920 "$NEU" dunkel en 1920x1080 $Z/foto-1920.txt --bildschirm-anwenden
# Fotos
for sp in de en; do for b in 1024 1920; do for art in hell dunkel; do
    QE_BILDSCHIRM_TOML=$Z/foto-$b.toml lauf "016-p-$art-$sp-$b" "$NEU" $art $sp 1920x1080 "$Z/foto-$b.txt" --bildschirm-anwenden
done; done; done
# Fensterrahmen (ein Ausgang, 1920, de)
export QE_OUTPUTS=1
RAHMEN_THEMA=${RAHMEN_THEMA:-$HOME/opt/querencia-theme/519997cf33de0cb1/Querencia-Klassisch}
THEMA=$RAHMEN_THEMA QE_LABWC_DATEIEN=$P/$Z/rahmen-hell lauf 016-r-hell-de-1920 "$NEU" hell de 1920x1080 $Z/r-hell.txt "--seite erscheinungsbild"
THEMA=$RAHMEN_THEMA QE_LABWC_DATEIEN=$P/$Z/rahmen-dunkel lauf 016-r-dunkel-de-1920 "$NEU" dunkel de 1920x1080 $Z/r-dunkel.txt "--seite erscheinungsbild"
THEMA=$RAHMEN_THEMA QE_LABWC_DATEIEN=$P/$Z/rahmen-fremd QE_HEIM_DATEIEN=$P/$Z/heim-fremd lauf 016-r-fremd-de-1920 "$NEU" hell de 1920x1080 $Z/r-fremd.txt "--seite erscheinungsbild"
THEMA=$RAHMEN_THEMA QE_LABWC_DATEIEN=$P/$Z/rahmen-hell QE_THEMEN_WEG=Querencia-Klassisch-Dunkel lauf 016-r-ohne-de-1920 "$NEU" hell de 1920x1080 $Z/r-ohne.txt "--seite erscheinungsbild"
# Regression wie 0.1.5 (ein Ausgang)
export QE_OUTPUTS=1
for k in de-1024 en-1920; do
    sp=${k%-*} b=${k#*-}
    s=$([ "$b" = 1920 ] && echo 1920x1080 || echo 1024x768)
    for art in hell dunkel; do
        lauf "016-d1-$art-alt-$k" "$ALT" $art $sp $s szenarien/015/d1-alle-seiten-$b.txt
        lauf "016-d1-$art-neu-$k" "$NEU" $art $sp $s szenarien/015/d1-alle-seiten-$b.txt
    done
done
werkzeug/produktiv-stand.sh > "$NW/produktiv-nachher.txt"
ls -l --time-style=full-iso ~/.local/share/applications > "$NW/starter-nachher.txt"
diff "$NW/starter-vorher.txt" "$NW/starter-nachher.txt" > "$NW/starter-diff.txt" && echo "Starter unverändert" || { echo "Starter geändert (andere Sessions?):"; cat "$NW/starter-diff.txt"; }
gsettings list-recursively org.gnome.desktop.interface > "$NW/gsettings-live-nachher.txt"
ls -l --time-style=full-iso ~/.config/querencia/ > "$NW/konfig-live-nachher.txt" 2>&1
if diff "$NW/produktiv-vorher.txt" "$NW/produktiv-nachher.txt" && diff "$NW/gsettings-live-vorher.txt" "$NW/gsettings-live-nachher.txt" &&
    diff "$NW/konfig-live-vorher.txt" "$NW/konfig-live-nachher.txt" && [ ! -e ~/.config/querencia/bildschirm.toml ] &&
    cmp -s <(grep org.querencia.Einstellungen "$NW/starter-vorher.txt") <(grep org.querencia.Einstellungen "$NW/starter-nachher.txt"); then
    echo "PRODUKTIV UNVERÄNDERT"
else
    echo "PRODUKTIV VERÄNDERT"
fi
bash werkzeug/wsdd-reste.sh "$AB" | tee "$NW/wsdd-nach-gui.txt"
. "$HOME/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh"
python3 werkzeug/auswerten-016.py "$NW/gui-016.tsv"; echo "auswertung exit=$?"
