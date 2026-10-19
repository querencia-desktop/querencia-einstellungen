#!/bin/bash
# Querencia Einstellungen 0.0.2 (G2-G: Bereich „Bildschirm“ – Auflösung,
# Wiederholrate, Drehung, Skalierung, Anordnung, Hauptmonitor, Ein/Aus,
# bildschirm.ini, --bildschirm-anwenden): Versionswechsel der Live-Ablage.
# NUR NACH FREIGABE DES BESITZERS.
#
#   bash einstellungen-g2g-integrieren.sh pruefen      nur lesen
#   bash einstellungen-g2g-integrieren.sh aktivieren   0.0.1 (G2-C) → 0.0.2 (G2-G)
#   bash einstellungen-g2g-integrieren.sh zurueck      0.0.2 → 0.0.1
#
# Ausgang ist genau der Endstand von einstellungen-integrieren.sh (G2-C):
# Link ~/opt/querencia/bin/querencia-einstellungen → Ablage 2f44261d01fd2948,
# Menüeintrag b93db7b1…. Ein anderer Stand wird nicht angefasst. `zurueck`
# führt genau dorthin zurück; von dort gilt einstellungen-integrieren.sh zurueck
# (ganz entfernen).
#
# Komponenten (jede atomar per mv bzw. Link-Tausch):
#   A  ~/opt/querencia/einstellungen/03c6dec5e63966b9/querencia-einstellungen
#        neue schreibgeschützte Ablage (0555) + 03c6dec5e63966b9.herkunft; bleibt liegen
#   P  ~/opt/querencia/bin/querencia-einstellungen → Ablage A
#   D  ~/.local/share/applications/org.querencia.Einstellungen.desktop
#        Exec/TryExec → 03c6dec5e63966b9 (Name/Kategorie unverändert: Systemeinstellungen)
#   Dazu je eine Zeile in ~/opt/querencia/protokoll.txt und im Journal
#   ~/.local/state/querencia/panel-rueckfall.txt.
# NICHT Teil dieses Skripts: der Autostart-Eintrag (integration/g2g/
# autostart-bildschirm.patch, Runtime-Repo, eigene Freigabe) und eine
# ~/.config/querencia/bildschirm.ini (schreibt nur der Benutzer über „Behalten“).
# Ein offenes Einstellungsfenster läuft als 0.0.1 weiter, bis es geschlossen wird.
#
# Aufbau wie einstellungen-integrieren.sh / qs024-integrieren.sh: Sperre,
# Vorprüfung (jede Komponente „alt“ oder „neu“, sonst nichts geändert),
# Sicherung, Schritte mit Kontrolle, Rückrollen bei Fehler oder INT/TERM/HUP;
# Exit 0 erledigt, 1 zurückgerollt, 2 verweigert, 3 Rückrollen unvollständig.
# Nur für die Probe: QE_OPT, QE_APPS, QE_STATE; QE_PROBE=1 mit QE_FEHLER=<Stelle>.
set -uo pipefail
HIER=~/projects/querencia-werkstatt-w6-einstellungen/werkzeug
VORSCHLAG=~/projects/querencia-werkstatt-w6-einstellungen/integration/g2g/vorschlag
AUSGANG=~/projects/querencia-werkstatt-w6-einstellungen/integration/g2g/ausgang
OPT=${QE_OPT:-~/opt/querencia}
APPS=${QE_APPS:-~/.local/share/applications}
STATE=${QE_STATE:-~/.local/state/querencia}

PROG=querencia-einstellungen
DESKNAME=org.querencia.Einstellungen.desktop
LINK=$OPT/bin/$PROG
DESK=$APPS/$DESKNAME
PROTO=$OPT/protokoll.txt

PROG_ALT=2f44261d01fd29482b3a31cbf900c021ceb5ae82ecd9b5f6ec2ee328eee24e41
DESK_ALT=b93db7b11b96d81a0a92e3a17b0164094a880e3dac7086729fdd8d99fc147a77
PROG_NEU=03c6dec5e63966b9088b97c02311d6e3cec0f662831a315e3a436669e0fd0a9b
DESK_NEU=4624a5df3069b13b23206a6e54371c3e40872a10eaf53d5dca5b0383ba806970
ALT16=${PROG_ALT:0:16}
NEU16=${PROG_NEU:0:16}
ABLAGE_ALT=$OPT/einstellungen/$ALT16
ABLAGE_NEU=$OPT/einstellungen/$NEU16

summe() { sha256sum "$1" 2>/dev/null | cut -d' ' -f1; }
melde() { echo "einstellungen-g2g-integrieren: $*"; }
die() { echo "einstellungen-g2g-integrieren: $*" >&2; exit 2; }

fehlerstelle() {
    [ "${QE_PROBE:-}" = 1 ] && [ -n "${QE_OPT:-}" ] || return 0
    case ${QE_FEHLER:-} in
    "$1") melde "Testfehler an Stelle $1"; return 1 ;;
    "signal-$1") melde "Testsignal TERM an Stelle $1"; kill -TERM $$; sleep 1; return 0 ;;
    esac
    return 0
}

zustand_p() {
    local ziel s
    [ -L "$LINK" ] || { [ -e "$LINK" ] && echo fremd || echo fehlt; return; }
    ziel=$(readlink "$LINK"); s=$(summe "$LINK")
    if [ "$ziel" = "$ABLAGE_ALT/$PROG" ] && [ "$s" = "$PROG_ALT" ]; then echo alt
    elif [ "$ziel" = "$ABLAGE_NEU/$PROG" ] && [ "$s" = "$PROG_NEU" ]; then echo neu
    else echo fremd; fi
}

zustand_d() {
    local s
    [ -f "$DESK" ] && [ ! -L "$DESK" ] || { [ -e "$DESK" ] && echo fremd || echo fehlt; return; }
    s=$(summe "$DESK")
    if [ "$s" = "$DESK_ALT" ]; then echo alt
    elif [ "$s" = "$DESK_NEU" ]; then echo neu
    else echo fremd; fi
}
alle() { echo "$(zustand_p) $(zustand_d)"; }

invarianten() {
    [ "$(summe "$VORSCHLAG/$PROG")" = $PROG_NEU ] || die "Vorlage Programm: falsche Prüfsumme ($VORSCHLAG/$PROG)"
    [ "$(summe "$VORSCHLAG/$DESKNAME")" = $DESK_NEU ] || die "Vorlage Menüeintrag: falsche Prüfsumme"
    grep -qx "Exec=~/opt/querencia/einstellungen/$NEU16/$PROG" "$VORSCHLAG/$DESKNAME" || die "Vorlage Menüeintrag: Exec passt nicht zur Ablage $NEU16"
    [ "$(summe "$AUSGANG/$DESKNAME")" = $DESK_ALT ] || die "Referenzkopie Menüeintrag 0.0.1: falsche Prüfsumme"
    [ -d "$OPT/bin" ] && [ ! -L "$OPT/bin" ] || die "kein Ablageordner: $OPT/bin"
    [ -f "$PROTO" ] || die "Protokoll fehlt: $PROTO"
    [ -L "$OPT/bin/querencia-desktop" ] || die "kein Desktop-Link in $OPT/bin – falscher Ablageordner?"
    [ -d "$APPS" ] || die "kein Anwendungsordner: $APPS"
    [ "$(summe "$ABLAGE_ALT/$PROG")" = $PROG_ALT ] || die "0.0.1-Ablage fehlt oder verändert (Rückfallziel): $ABLAGE_ALT"
    if [ -e "$ABLAGE_NEU" ] || [ -L "$ABLAGE_NEU" ]; then
        [ -d "$ABLAGE_NEU" ] && [ ! -L "$ABLAGE_NEU" ] || die "0.0.2-Ablage ist kein Ordner: $ABLAGE_NEU"
        [ "$(summe "$ABLAGE_NEU/$PROG")" = $PROG_NEU ] || die "0.0.2-Ablage verändert: $ABLAGE_NEU"
    fi
}

zustaende_zulaessig() {
    local z
    for z in "Programm:$(zustand_p)" "Menüeintrag:$(zustand_d)"; do
        case ${z#*:} in alt | neu) ;; fehlt) die "${z%%:*} fehlt – zuerst einstellungen-integrieren.sh aktivieren (0.0.1); nichts geändert" ;; *) die "${z%%:*} ist ${z#*:} – nichts geändert" ;; esac
    done
}

sperren() {
    mkdir -p -- "$STATE" || die "kann $STATE nicht anlegen"
    exec 8>"$STATE/einstellungen-integrieren.lock"
    flock -n 8 || die "ein anderer Lauf eines einstellungen-*-integrieren.sh ist aktiv – nichts geändert"
}

journal() { echo "$(date '+%F %T') einstellungen-g2g: $*" >>"$STATE/panel-rueckfall.txt"; }
protokoll() { echo "$(date '+%F %T') $*" >>"$PROTO"; }

link_setzen() {
    local link=$1 ziel=$2 stelle=$3 tmp
    tmp=$(dirname "$link")/.$(basename "$link").g2g-tmp
    rm -f -- "$tmp"
    ln -s -- "$ziel" "$tmp" || return 1
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$link" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

datei_ersetzen() {
    local quelle=$1 ziel=$2 stelle=$3 modus tmp
    modus=$(stat -c %a "$ziel" 2>/dev/null || echo 644)
    tmp=$(dirname "$ziel")/.$(basename "$ziel").g2g-tmp
    rm -f -- "$tmp"
    cp -- "$quelle" "$tmp" && chmod "$modus" "$tmp" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$ziel" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

ablage_anlegen() {
    local tmp
    if [ ! -e "$ABLAGE_NEU" ]; then
        mkdir -p -- "$OPT/einstellungen" || return 1
        tmp=$OPT/einstellungen/.$NEU16.g2g-tmp
        if [ -e "$tmp" ]; then chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; fi
        mkdir -- "$tmp" && cp -- "$VORSCHLAG/$PROG" "$tmp/$PROG" &&
            [ "$(summe "$tmp/$PROG")" = $PROG_NEU ] &&
            chmod 0555 "$tmp/$PROG" "$tmp" && mv -T -- "$tmp" "$ABLAGE_NEU" || { [ -e "$tmp" ] && chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; return 1; }
        echo "Quelle: $VORSCHLAG/$PROG" >"$OPT/einstellungen/$NEU16.herkunft" || return 1
    fi
    [ "$(summe "$ABLAGE_NEU/$PROG")" = $PROG_NEU ] && [ -x "$ABLAGE_NEU/$PROG" ]
}

GEAENDERT=()
REIHENFOLGE=""
SICHERUNG=""
AKTION=""

zurueckrollen() {
    local grund=$1 k ok=1 soll
    trap '' INT TERM HUP
    melde "FEHLER: $grund – rolle zurück (Sicherung $SICHERUNG)"
    rm -f -- "$OPT/bin/.$PROG.g2g-tmp" "$APPS/.$DESKNAME.g2g-tmp"
    for k in $(echo "$REIHENFOLGE" | rev); do
        case $k in
        P)
            soll=$(cat "$SICHERUNG/programm-link.txt")
            [ "$(readlink "$LINK" 2>/dev/null)" = "$soll" ] || link_setzen "$LINK" "$soll" - || ok=0
            ;;
        D)
            [ "$(summe "$DESK")" = "$(summe "$SICHERUNG/$DESKNAME")" ] ||
                datei_ersetzen "$SICHERUNG/$DESKNAME" "$DESK" - || ok=0
            ;;
        esac
    done
    if [ "$ok" = 1 ] && [ "$(alle)" = "$(cat "$SICHERUNG/zustand-vorher.txt")" ] &&
        [ "$(readlink "$LINK")" = "$(cat "$SICHERUNG/programm-link.txt")" ] &&
        [ "$(summe "$DESK")" = "$(summe "$SICHERUNG/$DESKNAME")" ]; then
        journal "$AKTION abgebrochen ($grund), Ausgangsstand wiederhergestellt ($(alle)); Sicherung $SICHERUNG"
        melde "zurückgerollt: Zustand wie vor dem Aufruf ($(alle))"
        exit 1
    fi
    journal "$AKTION abgebrochen ($grund), RÜCKROLLEN UNVOLLSTÄNDIG; Sicherung $SICHERUNG"
    melde "RÜCKROLLEN UNVOLLSTÄNDIG – Ausgangsstand liegt in $SICHERUNG; Zustand jetzt: $(alle)" >&2
    exit 3
}

schritt() {
    local k=$1 soll=$2 ist
    shift 2
    GEAENDERT+=("$k")
    echo "$k" >>"$SICHERUNG/schritte.txt"
    "$@" || zurueckrollen "Schritt $k fehlgeschlagen"
    case $k in P) ist=$(zustand_p) ;; D) ist=$(zustand_d) ;; esac
    [ "$ist" = "$soll" ] || zurueckrollen "Kontrolle nach Schritt $k: $ist statt $soll"
}

sichern() {
    SICHERUNG=$STATE/sicherung-einstellungen-g2g-$1-$(date +%Y%m%d-%H%M%S)-$$
    mkdir -p -- "$SICHERUNG" || die "Sicherung $SICHERUNG nicht anlegbar – nichts geändert"
    readlink "$LINK" >"$SICHERUNG/programm-link.txt" &&
        cp -p -- "$DESK" "$SICHERUNG/$DESKNAME" &&
        alle >"$SICHERUNG/zustand-vorher.txt" &&
        cp -p -- "$PROTO" "$SICHERUNG/protokoll.txt.vorher" &&
        (cd "$SICHERUNG" && sha256sum -- * >sha256-vorher.txt) ||
        die "Sicherung unvollständig – nichts geändert"
    : >"$SICHERUNG/schritte.txt"
}

transaktion_beginnen() {
    REIHENFOLGE=$1
    trap 'zurueckrollen "Signal empfangen"' INT TERM HUP
}

pruefen() {
    invarianten
    case $(alle) in
    "alt alt") melde "Stand: alt – Querencia Einstellungen 0.0.1 ($ALT16, nur Hintergrund)" ;;
    "neu neu") melde "Stand: neu – Querencia Einstellungen 0.0.2 ($NEU16, Hintergrund + Bildschirm)" ;;
    *) melde "Stand: gemischt ($(alle))" ;;
    esac
    melde "Programm-Link $LINK: $(zustand_p) → $(readlink "$LINK" 2>/dev/null || echo '-')"
    melde "Menüeintrag   $DESK: $(zustand_d) ($(summe "$DESK" 2>/dev/null | cut -c1-16 || echo -))"
    if [ -e "$ABLAGE_NEU" ]; then melde "Ablage 0.0.2  $ABLAGE_NEU: vorhanden"; else melde "Ablage 0.0.2  $ABLAGE_NEU: fehlt (wird bei aktivieren angelegt)"; fi
    melde "Ablage 0.0.1  $ABLAGE_ALT: vorhanden (Rückfallziel)"
    if [ -f "${XDG_CONFIG_HOME:-$HOME/.config}/querencia/bildschirm.ini" ]; then
        melde "bildschirm.ini vorhanden (wird vom Skript nie angefasst)"
    else
        melde "keine bildschirm.ini (entsteht erst durch „Einstellungen behalten“ in der App)"
    fi
    melde "unverändert bleiben: Links querencia-desktop/-panel, panel.ini, desktop.ini, desktop-symbole, Autostart"
}

aktivieren() {
    AKTION=aktivieren
    sperren
    invarianten
    zustaende_zulaessig
    if [ "$(alle)" = "neu neu" ]; then melde "bereits aktiv, nichts geändert"; return 0; fi
    desktop-file-validate "$VORSCHLAG/$DESKNAME" >/dev/null 2>&1 || die "Vorlage Menüeintrag ungültig"
    sichern aktivieren
    melde "Sicherung: $SICHERUNG (vorher: $(alle))"
    transaktion_beginnen "P D"
    fehlerstelle vor-erstem || zurueckrollen "Testfehler vor dem ersten Schritt"
    ablage_anlegen || zurueckrollen "Ablage nicht anlegbar"
    [ "$(zustand_p)" = alt ] && schritt P neu link_setzen "$LINK" "$ABLAGE_NEU/$PROG" waehrend-programm
    fehlerstelle nach-programm || zurueckrollen "Testfehler nach dem Programm"
    [ "$(zustand_d)" = alt ] && schritt D neu datei_ersetzen "$VORSCHLAG/$DESKNAME" "$DESK" waehrend-menue
    fehlerstelle nach-menue || zurueckrollen "Testfehler nach dem Menüeintrag"
    [ "$(alle)" = "neu neu" ] || zurueckrollen "Schlusskontrolle: $(alle)"
    (invarianten) || zurueckrollen "Invarianten nach der Aktivierung verletzt"
    desktop-file-validate "$DESK" >/dev/null 2>&1 || zurueckrollen "Menüeintrag ungültig"
    trap - INT TERM HUP
    protokoll "$PROG -> $ABLAGE_NEU/$PROG (aus $VORSCHLAG/$PROG; 0.0.2 Bildschirm; Menüeintrag $DESK; G2-G)"
    journal "aktiviert (Querencia Einstellungen 0.0.2 $NEU16, Link, Menüeintrag; geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Rückfall: bash $HIER/einstellungen-g2g-integrieren.sh zurueck"
    melde "aktiviert (geändert: ${GEAENDERT[*]:-nichts}). Gilt für jeden neuen Start; ein offenes Fenster läuft als 0.0.1 weiter."
}

zurueck() {
    AKTION=zurueck
    sperren
    invarianten
    zustaende_zulaessig
    if [ "$(alle)" = "alt alt" ]; then melde "nichts zu tun (Stand ist bereits 0.0.1)"; return 0; fi
    sichern zurueck
    melde "Sicherung: $SICHERUNG (vorher: $(alle))"
    transaktion_beginnen "D P"
    fehlerstelle zurueck-vor-erstem || zurueckrollen "Testfehler vor dem ersten Rückfall-Schritt"
    [ "$(zustand_d)" = neu ] && schritt D alt datei_ersetzen "$AUSGANG/$DESKNAME" "$DESK" zurueck-waehrend-menue
    fehlerstelle zurueck-nach-menue || zurueckrollen "Testfehler nach dem Menüeintrag"
    [ "$(zustand_p)" = neu ] && schritt P alt link_setzen "$LINK" "$ABLAGE_ALT/$PROG" zurueck-waehrend-programm
    [ "$(alle)" = "alt alt" ] || zurueckrollen "Schlusskontrolle: $(alle)"
    (invarianten) || zurueckrollen "Invarianten nach dem Rückfall verletzt"
    trap - INT TERM HUP
    protokoll "zurück: $PROG -> $ABLAGE_ALT/$PROG (0.0.1; Ablage $NEU16 bleibt; G2-G)"
    journal "zurück auf 0.0.1 (geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Ablage $NEU16 bleibt liegen"
    melde "zurück auf 0.0.1 (geändert: ${GEAENDERT[*]:-nichts}). Eine bildschirm.ini bleibt liegen; 0.0.1 kennt --bildschirm-anwenden nicht (Autostart-Zeile bleibt wirkungslos)."
}

case ${1:-} in
pruefen) pruefen ;;
aktivieren) aktivieren ;;
zurueck) zurueck ;;
*) echo "Aufruf: bash $HIER/einstellungen-g2g-integrieren.sh pruefen|aktivieren|zurueck" >&2; exit 2 ;;
esac
