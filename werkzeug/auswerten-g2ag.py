#!/usr/bin/python3 -I
"""G2-AG: Auswertung der GUI-Läufe (nachweis/g2ag-*-1920, *-1024). Prüft die
geschriebenen Dateien der Benutzerkopie, das Protokoll der App und einzelne
Bildpunkte der Bildschirmfotos (Fenster maximiert? Ecke rund/eckig? Schatten?
Maximieren-Knopf? Panel-Umschalter). Schreibt eine TSV; Exit 0 nur ohne FAIL."""
import os, re, struct, sys, zlib

N = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "nachweis")
zeilen = []

# Bildpunkte je Auflösung (Kontrollzentrum 860×600 mittig, DejaVu Sans 10)
BP = {
    "1920": {"ecke": (531, 227), "links_daneben": (520, 500), "max_knopf": (1339, 240), "umschalter6": (1694, 1063)},
    "1024": {"ecke": (83, 71), "links_daneben": (72, 400), "max_knopf": (891, 84), "umschalter6": (850, 752)},
}

def pruef(lauf, fall, ok, detail=""):
    zeilen.append((lauf, fall, "PASS" if ok else "FAIL", detail))

def lesen(p):
    try:
        with open(p, encoding="utf-8") as f:
            return f.read()
    except OSError:
        return ""

def ls(p):
    try:
        return os.listdir(p)
    except OSError:
        return []

def png_pixel(pfad, x, y):
    """RGB eines Bildpunkts (8 bit, Farbtyp 2/6, ohne Interlace); fehlt das Bild: (0, 0, 0)."""
    try:
        with open(pfad, "rb") as f:
            d = f.read()
    except OSError:
        return (0, 0, 0)
    pos, idat, w = 8, b"", 0
    while pos < len(d):
        n, typ = struct.unpack(">I4s", d[pos:pos + 8])
        teil = d[pos + 8:pos + 8 + n]
        if typ == b"IHDR":
            w, h, tiefe, farbe = struct.unpack(">IIBB", teil[:10])
            bpp = {2: 3, 6: 4}[farbe]
        elif typ == b"IDAT":
            idat += teil
        pos += 12 + n
    roh = zlib.decompress(idat)
    zeile = w * bpp
    vorige = bytearray(zeile)
    for r in range(y + 1):
        start = r * (zeile + 1)
        filt, akt = roh[start], bytearray(roh[start + 1:start + 1 + zeile])
        for i in range(zeile):
            a = akt[i - bpp] if i >= bpp else 0
            b = vorige[i]
            c = vorige[i - bpp] if i >= bpp else 0
            if filt == 1: akt[i] = (akt[i] + a) & 255
            elif filt == 2: akt[i] = (akt[i] + b) & 255
            elif filt == 3: akt[i] = (akt[i] + (a + b) // 2) & 255
            elif filt == 4:
                p = a + b - c; pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                akt[i] = (akt[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        vorige = akt
    return tuple(vorige[x * bpp:x * bpp + 3])

def blau(px):  # Hintergrund „Querencia Blau“
    return px[2] > px[0] + 40 and px[2] > 90

def orange(px):  # Akzent des Panel-Umschalters
    return px[0] > 200 and px[1] < 170 and px[2] < 120

def hell(px):
    return min(px) > 200

def wert(rc, muster):
    m = re.search(muster, rc)
    return m.group(1) if m else None

def app(lauf):
    return lesen(f"{N}/g2ag-{lauf}/logs/app-direkt.log")

def nachher(lauf, datei):
    return lesen(f"{N}/g2ag-{lauf}/labwc-nachher/{datei}")

def gemeinsam(lauf, sofort=True):
    a = app(lauf)
    pruef(lauf, "Szenario fertig", "szenario exit=0" in lesen(f"{N}/g2ag-{lauf}/logs/eingabe.log"))
    pruef(lauf, "kein Panic/Fehler/Konflikt", "panicked" not in a and "nicht gespeichert" not in a and "Konflikt" not in a)
    if sofort:
        pruef(lauf, "labwc neu geladen (Status)", "labwc hat die Konfiguration neu geladen" in a)
    else:
        pruef(lauf, "ab nächster Anmeldung (Status), kein Neuladen",
              "wirksam ab der nächsten Anmeldung" in a and "neu geladen" not in a)

for A in ("1920", "1024"):
    bp = BP[A]
    L = f"g1-tastatur-{A}"; gemeinsam(L); rc = nachher(L, "rc.xml"); env = nachher(L, "environment.d/zz-querencia-einstellungen.env")
    pruef(L, "repeatRate 30, repeatDelay 500, numlock on",
          "<repeatRate>30</repeatRate>" in rc and "<repeatDelay>500</repeatDelay>" in rc and "<numlock>on</numlock>" in rc)
    pruef(L, "Belegung ch (Deutsch (Schweiz)) in environment.d", "XKB_DEFAULT_LAYOUT=ch" in env and "gewählt Deutsch (Schweiz) (ch:)" in app(L))
    pruef(L, "environment unverändert (Vorgabe)", nachher(L, "environment") == lesen(f"{N}/g2ag-{L}/labwc-vorher/environment") != "")
    pruef(L, "Sicherung rc.xml.vorher-*", any(f.startswith("rc.xml.vorher-") for f in ls(f"{N}/g2ag-{L}/labwc-nachher")))

    L = f"g2-maus-{A}"; gemeinsam(L); rc = nachher(L, "rc.xml"); b = f"{N}/g2ag-{L}"
    pruef(L, "doubleClickTime 300", "<doubleClickTime>300</doubleClickTime>" in rc)
    pruef(L, "leftHanded/naturalScroll yes im Standardgerät",
          re.search(r'<device category="default">.*<leftHanded>yes</leftHanded>.*<naturalScroll>yes</naturalScroll>.*</device>', rc, re.S) is not None)
    sp = wert(rc, r"<pointerSpeed>([-0-9.]+)</pointerSpeed>")
    pruef(L, "pointerSpeed > 0", sp is not None and float(sp) > 0, f"pointerSpeed={sp}")
    p4, p5 = png_pixel(f"{b}/04-titel-zwei-klicks-gross-abstand.png", 300, 45), png_pixel(f"{b}/05-titel-zwei-klicks-klein-abstand.png", 300, 45)
    pruef(L, "300 ms: Klickabstand ~380 ms maximiert NICHT", blau(p4), str(p4))
    pruef(L, "300 ms: Klickabstand ~110 ms maximiert", not blau(p5), str(p5))
    L = f"g2k-doppelklick-kontrolle-{A}"; b = f"{N}/g2ag-{L}"
    pruef(L, "Szenario fertig", "szenario exit=0" in lesen(f"{b}/logs/eingabe.log"))
    p1, p2 = png_pixel(f"{b}/01-start.png", 300, 45), png_pixel(f"{b}/02-titel-zwei-klicks-gross-abstand.png", 300, 45)
    pruef(L, "Kontrolle 500 ms: Klickabstand ~380 ms maximiert", blau(p1) and not blau(p2), f"{p1} → {p2}")

    L = f"g3-fensterdesign-{A}"; gemeinsam(L); rc = nachher(L, "rc.xml"); b = f"{N}/g2ag-{L}"
    pruef(L, "cornerRadius 0, dropShadows yes, layout icon:close",
          "<cornerRadius>0</cornerRadius>" in rc and "<dropShadows>yes</dropShadows>" in rc and "<layout>icon:close</layout>" in rc)
    e1, e3 = png_pixel(f"{b}/01-fensterdesign.png", *bp["ecke"]), png_pixel(f"{b}/03-uebernommen.png", *bp["ecke"])
    pruef(L, "Ecke vorher rund (Hintergrund), nachher eckig (Rahmen)", blau(e1) and not blau(e3), f"{e1} → {e3}")
    s1, s3 = png_pixel(f"{b}/01-fensterdesign.png", *bp["links_daneben"]), png_pixel(f"{b}/03-uebernommen.png", *bp["links_daneben"])
    pruef(L, "Schatten: Hintergrund links neben dem Fenster dunkler", sum(s3) + 6 < sum(s1), f"{s1} → {s3}")
    m1, m3 = png_pixel(f"{b}/01-fensterdesign.png", *bp["max_knopf"]), png_pixel(f"{b}/03-uebernommen.png", *bp["max_knopf"])
    pruef(L, "Maximieren-Knopf weg", not hell(m1) and hell(m3), f"{m1} → {m3}")

    L = f"g4-arbeitsflaechen-{A}"; gemeinsam(L); rc = nachher(L, "rc.xml"); b = f"{N}/g2ag-{L}"
    pruef(L, "number 6, popupTime 1000", 'number="6" popupTime="1000"' in rc)
    pruef(L, "Namen Post, Arbeitsfläche 2, Web", "<name>Post</name>\n      <name>Arbeitsfläche 2</name>\n      <name>Web</name>" in rc)
    u1, u3 = png_pixel(f"{b}/01-arbeitsflaechen.png", *bp["umschalter6"]), png_pixel(f"{b}/03-uebernommen.png", *bp["umschalter6"])
    pruef(L, "Panel-Umschalter: vorher 4, nachher 6 Knöpfe", not orange(u1) and orange(u3), f"{u1} → {u3}")

    L = f"k1-standard-{A}"; gemeinsam(L); rc = nachher(L, "rc.xml"); env = nachher(L, "environment.d/zz-querencia-einstellungen.env"); b = f"{N}/g2ag-{L}"
    pruef(L, "vorher Rate 40 / us intl / 48",
          "<repeatRate>40</repeatRate>" in lesen(f"{b}/labwc-vorher/rc.xml") and "XKB_DEFAULT_VARIANT=intl" in lesen(f"{b}/labwc-vorher/environment.d/zz-querencia-einstellungen.env"))
    pruef(L, "Standard: repeatRate 25", "<repeatRate>25</repeatRate>" in rc)
    pruef(L, "Standard: eigene Belegung entfernt (Vorgabe de gilt)", "XKB_DEFAULT_LAYOUT" not in env and "XKB_DEFAULT_VARIANT" not in env)
    pruef(L, "Zeigergröße 32", "XCURSOR_SIZE=32" in env)
    pruef(L, "Sicherungen der .env enden nicht auf .env",
          [f for f in ls(f"{b}/labwc-nachher/environment.d") if f.endswith(".env")] == ["zz-querencia-einstellungen.env"])

    L = f"v1-vorgabe-{A}"; a = app(L); rc = nachher(L, "rc.xml"); b = f"{N}/g2ag-{L}"
    pruef(L, "Szenario fertig", "szenario exit=0" in lesen(f"{b}/logs/eingabe.log"))
    pruef(L, "neue Vorgabe übernommen (Protokoll, Neuladen)", "neue Vorgabe übernommen" in a and "panicked" not in a)
    pruef(L, "rc.xml = neue Vorgabe + eigener Wert repeatRate 40",
          'key="W-F12"' in rc and rc.replace("    <repeatRate>40</repeatRate>\n", "") == lesen(f"{b}/labwc-vorher/rc.xml.vorgabe-neu") != "")
    pruef(L, ".vorgabe/rc.xml = neue Vorgabe, vorgabe-neu weg",
          nachher(L, ".vorgabe/rc.xml") == lesen(f"{b}/labwc-vorher/rc.xml.vorgabe-neu") and not os.path.exists(f"{b}/labwc-nachher/rc.xml.vorgabe-neu"))

    L = f"o1-ohne-kopie-{A}"; gemeinsam(L, sofort=False); rc = nachher(L, "rc.xml"); b = f"{N}/g2ag-{L}"
    pruef(L, "vorher keine Benutzerkopie", not os.path.exists(f"{b}/labwc-vorher"))
    pruef(L, "rc.xml angelegt: number 5, popupTime 1000", 'number="5" popupTime="1000"' in rc)
    pruef(L, ".vorgabe/rc.xml = Vorgabe (für den Starter)", nachher(L, ".vorgabe/rc.xml") == lesen(f"{b}/profil/vorgabe/rc.xml") != "")
    pruef(L, "kein environment angelegt (macht der Starter)", not os.path.exists(f"{b}/labwc-nachher/environment"))
    u3 = png_pixel(f"{b}/03-uebernommen.png", *bp["umschalter6"])
    pruef(L, "Panel bleibt bei 4 (kein Neuladen)", not orange(u3), str(u3))

with open(sys.argv[1], "w", encoding="utf-8") as f:
    f.write("lauf\tfall\tergebnis\tdetail\n")
    for z in zeilen:
        f.write("\t".join(z) + "\n")
fails = [z for z in zeilen if z[2] != "PASS"]
for z in fails:
    print("FAIL", z)
print(f"{len(zeilen) - len(fails)}/{len(zeilen)} PASS")
sys.exit(1 if fails else 0)
