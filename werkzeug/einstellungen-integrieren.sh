#!/bin/bash
# Querencia Einstellungen 0.0.1 (Kontrollzentrum, erster Bereich Hintergrund,
# QE-01; Querencia-Werkstatt G2-C): Live-Integration von Ablage, Programm-Link und
# Menüeintrag „Systemeinstellungen“. NUR NACH FREIGABE DES BESITZERS.
#
#   bash einstellungen-integrieren.sh pruefen      nur lesen
#   bash einstellungen-integrieren.sh aktivieren   nicht installiert → installiert
#   bash einstellungen-integrieren.sh zurueck      installiert → nicht installiert
#
# Ausgang („alt“) ist der Stand ohne Querencia Einstellungen: kein Link
# ~/opt/querencia/bin/querencia-einstellungen, kein Menüeintrag. Der
# Desktop-Menüpunkt „Hintergrund der Arbeitsfläche ändern …“ (QE-01) öffnet
# dann den GTK-Dateidialog. Ein anderer Stand wird nicht angefasst.
#
# Komponenten (jede atomar per mv bzw. Link-Tausch):
#   A  ~/opt/querencia/einstellungen/2f44261d01fd2948/querencia-einstellungen
#        neue schreibgeschützte Ablage (0555) + 2f44261d01fd2948.herkunft;
#        bleibt bei `zurueck` liegen (unveränderlich, wie bei installieren.sh)
#   P  ~/opt/querencia/bin/querencia-einstellungen → Ablage A
#        (hier sucht der QE-01-Desktop für seinen Menüpunkt, Aufruf --hintergrund)
#   D  ~/.local/share/applications/org.querencia.Einstellungen.desktop
#        Name=Systemeinstellungen, Categories=Settings; → Panel: System →
#        Einstellungen (QREL-04d liest Anwendungen von selbst neu ein)
#   Dazu je eine Zeile in ~/opt/querencia/protokoll.txt (wie installieren.sh)
#   und im Journal ~/.local/state/querencia/panel-rueckfall.txt.
# Unangetastet und geprüft: Links querencia-desktop/-panel und ihre Ablagen,
# panel.ini, desktop.ini, desktop-symbole, installieren.sh. Kein
# labwc --reconfigure, kein Neustart nötig; Programm und Menüeintrag gelten
# für jeden neuen Start.
#
# Aufbau und Sicherheit wie qs024-integrieren.sh (W3, live bewährt):
# Sperre; Vorprüfung vor jeder Änderung (jede Komponente „alt“ oder „neu“,
# sonst nichts geändert); Sicherung unter ~/.local/state/querencia/
# sicherung-einstellungen-…; Schritte P, D mit Vermerk VOR dem Austausch und
# Kontrolle danach; bei Fehler oder INT/TERM/HUP Rückrollen jeder Komponente,
# die vom gesicherten Ausgangsstand abweicht; Exit 0 erledigt, 1 zurück-
# gerollt, 2 verweigert (nichts geändert), 3 Rückrollen unvollständig.
#
# Nur für die Probe: QE_OPT, QE_APPS, QE_STATE lenken die Pfade um; mit
# QE_PROBE=1 (nur zusammen mit QE_OPT) löst QE_FEHLER=<Stelle> einen
# künstlichen Fehler aus.
set -uo pipefail
HIER=~/projects/querencia-einstellungen/werkzeug
VORSCHLAG=~/projects/querencia-einstellungen/integration/vorschlag
OPT=${QE_OPT:-~/opt/querencia}
APPS=${QE_APPS:-~/.local/share/applications}
STATE=${QE_STATE:-~/.local/state/querencia}

PROG=querencia-einstellungen
DESKNAME=org.querencia.Einstellungen.desktop
LINK=$OPT/bin/$PROG
DESK=$APPS/$DESKNAME
PROTO=$OPT/protokoll.txt

PROG_NEU=2f44261d01fd29482b3a31cbf900c021ceb5ae82ecd9b5f6ec2ee328eee24e41
DESK_NEU=b93db7b11b96d81a0a92e3a17b0164094a880e3dac7086729fdd8d99fc147a77
SHA16=${PROG_NEU:0:16}
ABLAGE=$OPT/einstellungen/$SHA16
HERKUNFT=$OPT/einstellungen/$SHA16.herkunft

summe() { sha256sum "$1" 2>/dev/null | cut -d' ' -f1; }
melde() { echo "einstellungen-integrieren: $*"; }
die() { echo "einstellungen-integrieren: $*" >&2; exit 2; }

fehlerstelle() {
    [ "${QE_PROBE:-}" = 1 ] && [ -n "${QE_OPT:-}" ] || return 0
    case ${QE_FEHLER:-} in
    "$1") melde "Testfehler an Stelle $1"; return 1 ;;
    "signal-$1") melde "Testsignal TERM an Stelle $1"; kill -TERM $$; sleep 1; return 0 ;;
    esac
    return 0
}

# ---------------------------------------------------------- Zustände

# P: fehlt → alt (nicht installiert); Link auf die Ablage mit richtiger
# Prüfsumme → neu; alles andere (Datei, fremder oder hängender Link) → fremd.
zustand_p() {
    local ziel s
    if [ ! -L "$LINK" ]; then [ -e "$LINK" ] && echo fremd || echo alt; return; fi
    ziel=$(readlink "$LINK"); s=$(summe "$LINK")
    if [ "$ziel" = "$ABLAGE/$PROG" ] && [ "$s" = "$PROG_NEU" ]; then echo neu; else echo fremd; fi
}

# D: fehlt → alt; genau die Vorlage → neu; sonst fremd.
zustand_d() {
    local s
    if [ ! -e "$DESK" ] && [ ! -L "$DESK" ]; then echo alt; return; fi
    [ -f "$DESK" ] && [ ! -L "$DESK" ] || { echo fremd; return; }
    s=$(summe "$DESK")
    if [ "$s" = "$DESK_NEU" ]; then echo neu; else echo fremd; fi
}
alle() { echo "$(zustand_p) $(zustand_d)"; }

invarianten() {
    [ "$(summe "$VORSCHLAG/$PROG")" = $PROG_NEU ] || die "Vorlage Programm: falsche Prüfsumme ($VORSCHLAG/$PROG)"
    [ "$(summe "$VORSCHLAG/$DESKNAME")" = $DESK_NEU ] || die "Vorlage Menüeintrag: falsche Prüfsumme"
    grep -qx "Exec=~/opt/querencia/einstellungen/$SHA16/$PROG" "$VORSCHLAG/$DESKNAME" || die "Vorlage Menüeintrag: Exec passt nicht zur Ablage $SHA16"
    [ -d "$OPT/bin" ] && [ ! -L "$OPT/bin" ] || die "kein Ablageordner: $OPT/bin"
    [ -f "$PROTO" ] || die "Protokoll fehlt: $PROTO"
    [ -L "$OPT/bin/querencia-desktop" ] || die "kein Desktop-Link in $OPT/bin – falscher Ablageordner?"
    [ -L "$OPT/bin/querencia-panel" ] || die "kein Panel-Link in $OPT/bin – falscher Ablageordner?"
    [ -d "$APPS" ] || die "kein Anwendungsordner: $APPS"
    if [ -e "$OPT/einstellungen" ]; then
        [ -d "$OPT/einstellungen" ] && [ ! -L "$OPT/einstellungen" ] || die "$OPT/einstellungen ist kein Ordner"
    fi
    if [ -e "$ABLAGE" ] || [ -L "$ABLAGE" ]; then
        [ -d "$ABLAGE" ] && [ ! -L "$ABLAGE" ] || die "Ablage ist kein Ordner: $ABLAGE"
        [ "$(summe "$ABLAGE/$PROG")" = $PROG_NEU ] || die "Ablage verändert: $ABLAGE"
    fi
}

zustaende_zulaessig() {
    local z
    for z in "Programm:$(zustand_p)" "Menüeintrag:$(zustand_d)"; do
        case ${z#*:} in alt | neu) ;; *) die "${z%%:*} ist ${z#*:} – nichts geändert" ;; esac
    done
}

sperren() {
    mkdir -p -- "$STATE" || die "kann $STATE nicht anlegen"
    exec 8>"$STATE/einstellungen-integrieren.lock"
    flock -n 8 || die "ein anderer Lauf von einstellungen-integrieren.sh ist aktiv – nichts geändert"
}

journal() { echo "$(date '+%F %T') einstellungen: $*" >>"$STATE/panel-rueckfall.txt"; }
protokoll() { echo "$(date '+%F %T') $*" >>"$PROTO"; }

# -------------------------------------------------- atomare Einzelschritte

link_setzen() {
    local link=$1 ziel=$2 stelle=$3 tmp
    tmp=$(dirname "$link")/.$(basename "$link").g2c-tmp
    rm -f -- "$tmp"
    ln -s -- "$ziel" "$tmp" || return 1
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$link" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

link_entfernen() {
    local link=$1 stelle=$2
    fehlerstelle "$stelle" || return 1
    [ -L "$link" ] || return 1
    rm -f -- "$link" || return 1
    fehlerstelle "$stelle-nach-rm" || return 1
}

datei_anlegen() {
    local quelle=$1 ziel=$2 stelle=$3 tmp
    tmp=$(dirname "$ziel")/.$(basename "$ziel").g2c-tmp
    rm -f -- "$tmp"
    cp -- "$quelle" "$tmp" && chmod 0644 "$tmp" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle" || { rm -f -- "$tmp"; return 1; }
    mv -f -T -- "$tmp" "$ziel" || { rm -f -- "$tmp"; return 1; }
    fehlerstelle "$stelle-nach-mv" || return 1
}

datei_entfernen() {
    local ziel=$1 stelle=$2
    fehlerstelle "$stelle" || return 1
    rm -f -- "$ziel" || return 1
    fehlerstelle "$stelle-nach-rm" || return 1
}

# A: Ablage anlegen, falls sie fehlt (unveränderlich; wird nie entfernt).
ablage_anlegen() {
    local tmp
    if [ ! -e "$ABLAGE" ]; then
        mkdir -p -- "$OPT/einstellungen" || return 1
        tmp=$OPT/einstellungen/.$SHA16.g2c-tmp
        if [ -e "$tmp" ]; then chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; fi
        mkdir -- "$tmp" && cp -- "$VORSCHLAG/$PROG" "$tmp/$PROG" &&
            [ "$(summe "$tmp/$PROG")" = $PROG_NEU ] &&
            chmod 0555 "$tmp/$PROG" "$tmp" && mv -T -- "$tmp" "$ABLAGE" || { [ -e "$tmp" ] && chmod -R u+w -- "$tmp" && rm -rf -- "$tmp"; return 1; }
        echo "Quelle: $VORSCHLAG/$PROG" >"$HERKUNFT" || return 1
    fi
    [ "$(summe "$ABLAGE/$PROG")" = $PROG_NEU ] && [ -x "$ABLAGE/$PROG" ]
}

# ------------------------------------------------------- Transaktion

GEAENDERT=()
REIHENFOLGE=""
SICHERUNG=""
AKTION=""

zurueckrollen() {
    local grund=$1 k ok=1 soll
    trap '' INT TERM HUP
    melde "FEHLER: $grund – rolle zurück (Sicherung $SICHERUNG)"
    rm -f -- "$OPT/bin/.$PROG.g2c-tmp" "$APPS/.$DESKNAME.g2c-tmp"
    for k in $(echo "$REIHENFOLGE" | rev); do
        case $k in
        P)
            soll=$(cat "$SICHERUNG/programm-link.txt")
            if [ -z "$soll" ]; then
                if [ -L "$LINK" ]; then rm -f -- "$LINK" || ok=0; fi
            else
                [ "$(readlink "$LINK" 2>/dev/null)" = "$soll" ] || link_setzen "$LINK" "$soll" - || ok=0
            fi
            ;;
        D)
            if [ -f "$SICHERUNG/$DESKNAME" ]; then
                [ "$(summe "$DESK")" = "$(summe "$SICHERUNG/$DESKNAME")" ] ||
                    datei_anlegen "$SICHERUNG/$DESKNAME" "$DESK" - || ok=0
            else
                if [ -e "$DESK" ] || [ -L "$DESK" ]; then rm -f -- "$DESK" || ok=0; fi
            fi
            ;;
        esac
    done
    if [ "$ok" = 1 ] && [ "$(alle)" = "$(cat "$SICHERUNG/zustand-vorher.txt")" ] &&
        [ "$(readlink "$LINK" 2>/dev/null || true)" = "$(cat "$SICHERUNG/programm-link.txt")" ] &&
        { [ ! -f "$SICHERUNG/$DESKNAME" ] && [ ! -e "$DESK" ] || [ "$(summe "$DESK")" = "$(summe "$SICHERUNG/$DESKNAME")" ]; }; then
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
    SICHERUNG=$STATE/sicherung-einstellungen-$1-$(date +%Y%m%d-%H%M%S)-$$
    mkdir -p -- "$SICHERUNG" || die "Sicherung $SICHERUNG nicht anlegbar – nichts geändert"
    { readlink "$LINK" 2>/dev/null || true; } >"$SICHERUNG/programm-link.txt" &&
        { [ ! -e "$DESK" ] || cp -p -- "$DESK" "$SICHERUNG/$DESKNAME"; } &&
        alle >"$SICHERUNG/zustand-vorher.txt" &&
        cp -p -- "$PROTO" "$SICHERUNG/protokoll.txt.vorher" &&
        ls -la -- "$APPS" >"$SICHERUNG/applications-liste.txt" &&
        ls -la -- "$OPT/bin" >"$SICHERUNG/opt-bin-liste.txt" &&
        (cd "$SICHERUNG" && sha256sum -- * >sha256-vorher.txt) ||
        die "Sicherung unvollständig – nichts geändert"
    : >"$SICHERUNG/schritte.txt"
}

transaktion_beginnen() {
    REIHENFOLGE=$1
    trap 'zurueckrollen "Signal empfangen"' INT TERM HUP
}

# ------------------------------------------------------------ Befehle

pruefen() {
    local d
    invarianten
    case $(alle) in
    "alt alt") melde "Stand: alt – Querencia Einstellungen nicht installiert (Desktop-Menüpunkt öffnet den Dateidialog)" ;;
    "neu neu") melde "Stand: neu – Querencia Einstellungen 0.0.1 ($SHA16) installiert" ;;
    *) melde "Stand: gemischt ($(alle))" ;;
    esac
    melde "Programm-Link $LINK: $(zustand_p) → $(readlink "$LINK" 2>/dev/null || echo '-')"
    melde "Menüeintrag   $DESK: $(zustand_d) ($(summe "$DESK" 2>/dev/null | cut -c1-16 || echo -))"
    if [ -e "$ABLAGE" ]; then melde "Ablage        $ABLAGE: vorhanden ($(summe "$ABLAGE/$PROG" | cut -c1-16), $(stat -c %a "$ABLAGE/$PROG"))"
    else melde "Ablage        $ABLAGE: fehlt (wird bei aktivieren angelegt)"; fi
    d=$(readlink -f "$OPT/bin/querencia-desktop" 2>/dev/null || true)
    if [ -n "$d" ] && grep -qa -- '--faehigkeiten' "$d" 2>/dev/null; then
        melde "Desktop-Link  → $d ($(summe "$d" | cut -c1-16)) kennt --faehigkeiten (QE-01): Farbe/Zentrieren/Kacheln möglich"
    else
        melde "Desktop-Link  → ${d:--} kennt --faehigkeiten NICHT: die App läuft, Farbe/Zentrieren/Kacheln bleiben gesperrt"
    fi
    melde "unverändert bleiben: Links querencia-desktop/-panel, panel.ini, desktop.ini, desktop-symbole"
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
    [ "$(zustand_p)" = alt ] && schritt P neu link_setzen "$LINK" "$ABLAGE/$PROG" waehrend-programm
    fehlerstelle nach-programm || zurueckrollen "Testfehler nach dem Programm"
    [ "$(zustand_d)" = alt ] && schritt D neu datei_anlegen "$VORSCHLAG/$DESKNAME" "$DESK" waehrend-menue
    fehlerstelle nach-menue || zurueckrollen "Testfehler nach dem Menüeintrag"
    [ "$(alle)" = "neu neu" ] || zurueckrollen "Schlusskontrolle: $(alle)"
    (invarianten) || zurueckrollen "Invarianten nach der Aktivierung verletzt"
    desktop-file-validate "$DESK" >/dev/null 2>&1 || zurueckrollen "Menüeintrag ungültig"
    trap - INT TERM HUP
    protokoll "$PROG -> $ABLAGE/$PROG (aus $VORSCHLAG/$PROG; Menüeintrag $DESK; G2-C)"
    journal "aktiviert (Querencia Einstellungen 0.0.1 $SHA16, Link, Menüeintrag Systemeinstellungen; geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Rückfall: bash $HIER/einstellungen-integrieren.sh zurueck"
    melde "aktiviert (geändert: ${GEAENDERT[*]:-nichts}). Gilt für jeden neuen Start; das Panel liest den Menüeintrag von selbst neu ein."
}

zurueck() {
    AKTION=zurueck
    sperren
    invarianten
    zustaende_zulaessig
    if [ "$(alle)" = "alt alt" ]; then melde "nichts zu tun (Querencia Einstellungen ist nicht installiert)"; return 0; fi
    sichern zurueck
    melde "Sicherung: $SICHERUNG (vorher: $(alle))"
    transaktion_beginnen "D P"
    fehlerstelle zurueck-vor-erstem || zurueckrollen "Testfehler vor dem ersten Rückfall-Schritt"
    [ "$(zustand_d)" = neu ] && schritt D alt datei_entfernen "$DESK" zurueck-waehrend-menue
    fehlerstelle zurueck-nach-menue || zurueckrollen "Testfehler nach dem Menüeintrag"
    [ "$(zustand_p)" = neu ] && schritt P alt link_entfernen "$LINK" zurueck-waehrend-programm
    [ "$(alle)" = "alt alt" ] || zurueckrollen "Schlusskontrolle: $(alle)"
    (invarianten) || zurueckrollen "Invarianten nach dem Rückfall verletzt"
    trap - INT TERM HUP
    protokoll "entfernt: $PROG (vorher $ABLAGE/$PROG; Ablage bleibt; Menüeintrag in $SICHERUNG; G2-C)"
    journal "zurück (Link und Menüeintrag entfernt; geändert: ${GEAENDERT[*]:-nichts}); Sicherung $SICHERUNG; Ablage $SHA16 bleibt liegen"
    melde "zurück (geändert: ${GEAENDERT[*]:-nichts}). Der Desktop-Menüpunkt öffnet ab jetzt wieder den Dateidialog; ein offenes Einstellungsfenster läuft weiter."
}

case ${1:-} in
pruefen) pruefen ;;
aktivieren) aktivieren ;;
zurueck) zurueck ;;
*) echo "Aufruf: bash $HIER/einstellungen-integrieren.sh pruefen|aktivieren|zurueck" >&2; exit 2 ;;
esac
