#!/bin/bash
# Einstellungen 0.2.0: ein isolierter Lauf der Seite „Treiber“ mit Attrappen.
# Bedingungen wie werkzeug/016-lauf.sh (Thema, Portal-Ersatz hell/dunkel,
# englische Benutzerkopie für „en“, ohne QUERENCIA_*-Variablen der
# Live-Sitzung); dazu je Fall aus szenarien/020/faelle/<fall>/:
#   sysfs.txt     PCI-Geräte (Adresse Klasse Hersteller Gerät Modul boot_vga)
#                 → sysfs-Attrappe, QE_TREIBER_SYSFS
#   devices.txt, list.txt, renderer.txt → Stubs ubuntu-drivers und
#                 querencia-sitzung (szenarien/020/bin, vorn im PATH)
#   bestand.tsv   Pakete der PackageKit-Attrappe (privater Bus, QS_PK_BUS);
#                 dpkg-query gibt deren Bestand aus
#   software-gl   vorhanden: die Sitzung hat Software-OpenGL eingeschaltet
#                 (die drei Variablen des Sitzungsstarters)
#   ohne-werkzeug vorhanden: kein ubuntu-drivers im PATH, bis die Attrappe
#                 ubuntu-drivers-common „installiert“
# QS_TROCKENLAUF=1 wie bei der Sitzung. Belege: nachweis/<lauf>/treiber/
# (pk.log, aufrufe.log, dpkg vorher/nachher).
#
#   bash werkzeug/020-lauf.sh <lauf> <binary> <hell|dunkel> <de|en> <BREITExHÖHE> <fall> <szenario> [app-args]
set -uo pipefail
P=$(dirname "$(dirname "$(readlink -f "$0")")")
cd "$P"
NAME=${1:?Lauf}; BIN=${2:?Binary}; ART=${3:?}; SP=${4:?}; AUFL=${5:?}; FALL=${6:?Fall}; SZ=${7:?Szenario}; ARGS=${8:---seite treiber}
F=$P/szenarien/020/faelle/$FALL
[ -d "$F" ] || { echo "Fall fehlt: $F" >&2; exit 2; }
export NW=${NW:-nachweis/020}
Z=$P/$NW/zustand/$NAME
rm -rf "$Z"; mkdir -p "$Z/sysfs/devices" "$Z/sysfs/module" "$Z/stubs" "$Z/bin"
while read -r slot klasse hersteller geraet modul boot; do
    [ -n "$slot" ] || continue
    d=$Z/sysfs/devices/$slot; mkdir -p "$d"
    echo "$klasse" > "$d/class"; echo "$hersteller" > "$d/vendor"; echo "$geraet" > "$d/device"
    [ "$boot" = 1 ] && echo 1 > "$d/boot_vga"
    if [ "$modul" != - ]; then mkdir -p "$Z/sysfs/module/$modul"; ln -s "$Z/sysfs/module/$modul" "$d/driver"; fi
done < "$F/sysfs.txt"
cp "$P"/szenarien/020/bin/* "$Z/stubs/"
if [ -e "$F/ohne-werkzeug" ]; then
    rm "$Z/stubs/ubuntu-drivers"
    mkdir -p "$Z/nach-installation/ubuntu-drivers-common"
    cp "$P/szenarien/020/bin/ubuntu-drivers" "$Z/nach-installation/ubuntu-drivers-common/"
fi
eval "$(bash werkzeug/020-pk.sh start "$Z" "$F/bestand.tsv")" || exit 2
cp "$Z/dpkg.txt" "$Z/dpkg-vorher.txt"
SW=()
[ -e "$F/software-gl" ] && SW=(LIBGL_ALWAYS_SOFTWARE=1 MESA_LOADER_DRIVER_OVERRIDE=llvmpipe GALLIUM_DRIVER=llvmpipe)
# Die Shells der Sessions erben die Software-Variablen der Live-Sitzung: erst
# entfernen, dann nur im Fall software-gl setzen.
env -u LIBGL_ALWAYS_SOFTWARE -u MESA_LOADER_DRIVER_OVERRIDE -u GALLIUM_DRIVER "${SW[@]}" QS_PK_BUS="$QS_PK_BUS" QE_TREIBER_SYSFS="$Z/sysfs/devices" QE_TREIBER_DATEN="$F" QE_TREIBER_ZUSTAND="$Z" \
    QE_PFAD_VORNE="$Z/bin:$Z/stubs" QS_TROCKENLAUF=1 \
    bash werkzeug/016-lauf.sh "$NAME" "$BIN" "$ART" "$SP" "$AUFL" "$SZ" "$ARGS"
bash werkzeug/020-pk.sh stop "$Z"
mkdir -p "nachweis/$NAME/treiber"
for f in pk.log aufrufe.log dpkg-vorher.txt dpkg.txt bestand.tsv attrappe.log; do
    [ -e "$Z/$f" ] && cp "$Z/$f" "nachweis/$NAME/treiber/"
done
echo "$FALL" > "nachweis/$NAME/treiber/fall.txt"
