#!/usr/bin/python3
"""Einstellungen 0.1.6: Auswertung von werkzeug/016-gui-lauf.sh.

  f1–f5, k1  Bereich Bildschirm: Ausgänge am Ende (--bildschirm-zeigen,
             logs/ausgaben-ende.txt) und gespeicherte bildschirm.toml
             (bildschirm.toml.nachher) wie erwartet; f1 ohne Datei und
             wieder im Ausgangszustand
  f6–f8      --bildschirm-anwenden mit TOML, nur mit alter INI, ohne Datei
  f9         Hinweis „keine EDID“: Seite weicht vom Fotolauf ohne Schalter ab
  p          Fotos vorhanden, Fotogröße gesetzt; Ausschnitt der Ausgabe 0,0
             nach nachweis/016/bilder/
  d1         alt (0.1.5) ↔ neu (0.1.6), Seiten paarweise im Fensterausschnitt
             wie auswerten-015.py: gleich bis auf die Seite „Bildschirm“
             (Markierung, INFO); Ausnahmen wie 0.1.5: Uhrzeit im
             Formatbeispiel, Spalte des Overlay-Rollbalkens, „Erscheinungsbild
             unten“ (höchstens 0,1 % der Fläche, 16 Stufen); dazu auf
             „Informationen“ die Programmversion (eine Ziffer)
  r          (0.1.6-2) Fensterrahmen folgt der dunklen Darstellung: Name in
             rc.xml nach dem Umschalten (hell → Dunkel, dunkel → Klassisch,
             fremder Rahmen und ohne dunkles Thema unverändert), nach dem
             Zurückstellen rc.xml byte-gleich (SHA256 vorher/nachher), keine
             eigene Sicherung, die fremde rc.xml.vorher-* bleibt; labwc
             --reconfigure traf den isolierten labwc (LABWC_PID, eigenes
             XDG_RUNTIME_DIR, nicht der Sitzungs-labwc)
  alle       kein „panicked“ in den Protokollen der App

  . ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh  # Pillow (PYTHONPATH)
  python3 werkzeug/auswerten-016.py <ziel.tsv>
"""
import glob
import os
import sys
import tomllib

from PIL import Image, ImageChops

sys.dont_write_bytecode = True
P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
NW = os.path.join(P, os.environ.get("NW", "nachweis/016-2"))
BILDORDNER = os.path.join(NW, "bilder")
SEITEN = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels",
          "tastatur", "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "netzwerk",
          "sprache", "informationen"]
INHALT_X = 244
UHRZEIT = (470, 506)
VERSION = (180, 198)  # Zeile „Querencia Einstellungen <version>“ auf „Informationen“
ROLLBALKEN = (850, 875)
zeilen = []


def pruef(gruppe, fall, ok, detail=""):
    zeilen.append((gruppe, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ")[:400]))


def info(gruppe, fall, detail):
    zeilen.append((gruppe, fall, "INFO", str(detail).replace("\t", " ")[:400]))


def text(lauf, datei):
    p = os.path.join(N, lauf, datei)
    return open(p, encoding="utf-8").read() if os.path.exists(p) else None


def ausgaben(lauf, datei):
    """{name: {modus, position, skalierung, logisch}} aus --bildschirm-zeigen."""
    t = text(lauf, os.path.join("logs", datei)) or ""
    out = {}
    for z in t.splitlines():
        teile = z.split()
        if len(teile) < 2:
            continue
        w = dict(x.split("=", 1) for x in teile[2:] if "=" in x)
        out[teile[0]] = {"an": teile[1], **{k: w.get(k) for k in ("modus", "position", "skalierung", "logisch")}}
    return out


def toml_nachher(lauf):
    t = text(lauf, "bildschirm.toml.nachher")
    if t is None:
        return None
    return {a["name"]: a for a in tomllib.loads(t).get("ausgabe", [])}


def applog(lauf):
    return text(lauf, os.path.join("logs", "app-direkt.log")) or ""


def erwartet(gruppe, lauf, ende, datei):
    a = ausgaben(lauf, "ausgaben-ende.txt")
    for name, werte in ende.items():
        ist = {k: a.get(name, {}).get(k) for k in werte}
        pruef(gruppe, f"{lauf}: {name} am Ende {werte}", ist == werte, ist)
    t = toml_nachher(lauf)
    if datei is None:
        pruef(gruppe, f"{lauf}: keine bildschirm.toml", t is None, t)
        return
    if t is None:
        pruef(gruppe, f"{lauf}: bildschirm.toml geschrieben", False, "fehlt")
        return
    for name, werte in datei.items():
        ist = {k: t.get(name, {}).get(k) for k in werte}
        pruef(gruppe, f"{lauf}: bildschirm.toml {name} {werte}", ist == werte, ist)
    pruef(gruppe, f"{lauf}: bildschirm.toml nennt je Ausgabe Name, Hersteller, Modell",
          all({"name", "hersteller", "modell", "seriennummer"} <= set(a) for a in t.values()), list(t))


def funktion():
    g = "Bildschirm"
    l1 = "016-f1-rueckfall-en-1920"
    s, e = ausgaben(l1, "ausgaben-start.txt"), ausgaben(l1, "ausgaben-ende.txt")
    pruef(g, f"{l1}: nach Ablauf des Countdowns wie am Anfang", s and s == e, (s, e))
    log = applog(l1)
    pruef(g, f"{l1}: angewendet und zurückgesetzt", "Bildschirm angewendet" in log and "zurückgesetzt" in log, "")
    pruef(g, f"{l1}: nichts gespeichert", text(l1, "bildschirm.toml.nachher") is None, "")
    erwartet(g, "016-f2-beibehalten-de-1920",
             {"HEADLESS-1": {"modus": "1600x900@0", "position": "1280,0"}, "HEADLESS-2": {"position": "0,0"}},
             {"HEADLESS-1": {"breite": 1600, "hoehe": 900, "x": 1280, "primaer": False},
              "HEADLESS-2": {"breite": 1280, "hoehe": 720, "x": 0, "primaer": True}})
    erwartet(g, "016-f3-skalierung-de-1920",
             {"HEADLESS-1": {"skalierung": "1.25", "logisch": "1536x864"}},
             {"HEADLESS-1": {"skalierung": 1.25, "breite": 1920}})
    erwartet(g, "016-f4-hauptmonitor-de-1920",
             {"HEADLESS-1": {"position": "0,0"}, "HEADLESS-2": {"position": "1920,0"}},
             {"HEADLESS-1": {"primaer": True, "x": 0}, "HEADLESS-2": {"primaer": False, "x": 1920}})
    erwartet(g, "016-f5-ziehen-de-1920",
             {"HEADLESS-1": {"position": "0,720"}, "HEADLESS-2": {"position": "0,0"}},
             {"HEADLESS-1": {"x": 0, "y": 720}, "HEADLESS-2": {"x": 0, "y": 0}})
    erwartet(g, "016-k1-kontrollzentrum-de-1920",
             {"HEADLESS-1": {"skalierung": "2", "logisch": "960x540"}},
             {"HEADLESS-1": {"skalierung": 2.0}})
    ziel = {"HEADLESS-2": {"modus": "1024x768@0", "position": "0,0"},
            "HEADLESS-1": {"modus": "1600x900@0", "position": "1024,0"}}
    l6, l7, l8 = "016-f6-anwenden-toml-de-1920", "016-f7-anwenden-ini-de-1920", "016-f8-ohne-datei-de-1920"
    for lauf in (l6, l7):
        a = ausgaben(lauf, "ausgaben-ende.txt")
        for name, werte in ziel.items():
            ist = {k: a.get(name, {}).get(k) for k in werte}
            pruef(g, f"{lauf}: {name} gesetzt {werte}", ist == werte, ist)
    pruef(g, f"{l6}: aus bildschirm.toml, 2 von 2", "bildschirm.toml (2 von 2" in applog(l6), applog(l6)[-300:])
    pruef(g, f"{l7}: alte bildschirm.ini gelesen, 2 von 2",
          "lese die alte" in applog(l7) and "bildschirm.ini (2 von 2" in applog(l7), applog(l7)[-300:])
    pruef(g, f"{l7}: alte Datei bleibt unverändert, keine neue angelegt",
          text(l7, "bildschirm.ini.nachher") == open(os.path.join(P, "szenarien/016/f7-anwenden.ini"), encoding="utf-8").read()
          and text(l7, "bildschirm.toml.nachher") is None, "")
    s, e = ausgaben(l8, "ausgaben-start.txt"), ausgaben(l8, "ausgaben-ende.txt")
    pruef(g, f"{l8}: ohne Datei nichts geändert", s and s == e and "keine Datei" in applog(l8), applog(l8)[-200:])


def ausschnitt(lauf, datei, b):
    p = os.path.join(N, lauf, datei)
    if not os.path.exists(p):
        return None
    w, h = (1024, 768) if b == 1024 else (1920, 1080)
    return Image.open(p).convert("RGB").crop((0, 0, w, h))


def fotos():
    g = "Fotos"
    os.makedirs(BILDORDNER, exist_ok=True)
    for sp in ("de", "en"):
        for b in (1024, 1920):
            for art in ("hell", "dunkel"):
                lauf = f"016-p-{art}-{sp}-{b}"
                a = ausgaben(lauf, "ausgaben-ende.txt")
                pruef(g, f"{lauf}: Ausgabe 0,0 hat die Fotogröße", a.get("HEADLESS-2", {}).get("logisch") ==
                      ("1024x768" if b == 1024 else "1920x1080") and a.get("HEADLESS-2", {}).get("position") == "0,0", a)
                for datei in ("01-bildschirm.png", "02-bildschirm-unten.png", "03-eigenes-fenster.png"):
                    im = ausschnitt(lauf, datei, b)
                    pruef(g, f"{lauf}/{datei}", im is not None, "")
                    if im is not None:
                        im.save(os.path.join(BILDORDNER, f"bildschirm-{sp}-{b}-{art}-{datei[:2]}.png"))
    for lauf, vergleich, b in (("016-f9-edid-hell-de-1024", "016-p-hell-de-1024", 1024),
                               ("016-f9-edid-dunkel-en-1920", "016-p-dunkel-en-1920", 1920)):
        a, v = ausschnitt(lauf, "01-bildschirm.png", b), ausschnitt(vergleich, "01-bildschirm.png", b)
        if a is None or v is None:
            pruef(g, f"{lauf}: Hinweis keine EDID", False, "Bild fehlt")
            continue
        a.save(os.path.join(BILDORDNER, f"edid-{lauf.split('-', 3)[3]}.png"))
        d = ImageChops.difference(a, v)
        d.paste((0, 0, 0), (0, 0, b, 30))  # Panel mit Uhr
        pruef(g, f"{lauf}: Seite mit Hinweis weicht von {vergleich} ab", d.getbbox() is not None, d.getbbox())
    for lauf, datei, name in (("016-f1-rueckfall-en-1920", "03-dialog.png", "dialog-en-dunkel"),
                              ("016-f2-beibehalten-de-1920", "01-dialog.png", "dialog-de-hell"),
                              ("016-k1-kontrollzentrum-de-1920", "02-dialog.png", "dialog-kontrollzentrum-de-hell")):
        p = os.path.join(N, lauf, datei)
        if os.path.exists(p):
            Image.open(p).convert("RGB").crop((0, 0, 1280, 720)).save(os.path.join(BILDORDNER, f"{name}.png"))
        pruef(g, f"{lauf}/{datei}", os.path.exists(p), "")


def fenster(lauf, datei):
    p = os.path.join(N, lauf, datei)
    if not os.path.exists(p):
        return None
    im = Image.open(p).convert("RGB")
    b = int(lauf.rsplit("-", 1)[1])
    dx, dy = (0, 0) if b == 1920 else (-448, -156)
    return im.crop((520 + dx, 200 + dy, 1400 + dx, 850 + dy))


def markierung(a, b, name):
    m = b.point(lambda v: v // 3 + 170)
    d = ImageChops.difference(a, b).convert("L").point(lambda v: 255 if v else 0)
    m.paste((255, 140, 0), mask=d)
    w, h = a.size
    ges = Image.new("RGB", (3 * w + 20, h), (255, 255, 255))
    ges.paste(a, (0, 0))
    ges.paste(b, (w + 10, 0))
    ges.paste(m, (2 * w + 20, 0))
    ges.save(os.path.join(BILDORDNER, name))


def d1():
    for k in ("de-1024", "en-1920"):
        for art in ("hell", "dunkel"):
            alt, neu = f"016-d1-{art}-alt-{k}", f"016-d1-{art}-neu-{k}"
            g = f"d1 {art} {k}"
            paare = [(f"{i:02d}-{s}.png", s) for i, s in enumerate(SEITEN, 1)]
            paare.append(("15-erscheinungsbild-unten.png", "erscheinungsbild-unten"))
            for datei, s in paare:
                a, b = fenster(alt, datei), fenster(neu, datei)
                fall = f"{s}: {alt} ↔ {neu}/{datei}"
                if a is None or b is None:
                    pruef(g, fall, False, "Bild fehlt")
                    continue
                d = ImageChops.difference(a, b)
                w, h = a.size
                if s == "bildschirm":
                    markierung(a, b, f"vergleich-015-016-{art}-{k}-bildschirm.png")
                    sb = d.crop((0, 0, INHALT_X, h)).getbbox()
                    pruef(g, f"{fall}: Seitenleiste gleich", sb is None, sb)
                    info(g, f"{fall}: Seite geändert (Markierung)", d.getbbox())
                    continue
                if s == "sprache":
                    d.paste((0, 0, 0), (INHALT_X, UHRZEIT[0], w, UHRZEIT[1]))
                if s == "informationen":
                    # benannte Ausnahme: die Programmversion „0.1.5“ → „0.1.6“
                    # in der Zeile „Querencia Einstellungen …“ (höchstens eine
                    # Ziffer breit, im Zeilenband VERSION)
                    vb = d.crop((INHALT_X, VERSION[0], w, VERSION[1])).getbbox()
                    pruef(g, f"{fall}: Versionszeile weicht höchstens um eine Ziffer ab",
                          vb is None or (vb[2] - vb[0] <= 12), vb)
                    d.paste((0, 0, 0), (INHALT_X, VERSION[0], w, VERSION[1]))
                d.paste((0, 0, 0), (ROLLBALKEN[0], 0, ROLLBALKEN[1], h))
                bb = d.getbbox()
                if s == "erscheinungsbild-unten" and bb is not None:
                    px = sum(1 for v in getattr(d.convert("L"), "get_flattened_data", d.convert("L").getdata)() if v)
                    hoch = d.convert("L").getextrema()[1]
                    pruef(g, f"{fall}: gleich bis auf Zeiger-Hover (≤ 0,1 %, ≤ 16 Stufen)",
                          px <= w * h // 1000 and hoch <= 16, f"{px} Pixel, höchstens {hoch} Stufen, {bb}")
                    continue
                if bb is not None:
                    markierung(a, b, f"abweichung-{art}-{k}-{s}.png")
                pruef(g, f"{fall}: pixelgleich", bb is None, bb)


def rahmen():
    import hashlib
    import re
    g = "Fensterrahmen"
    pv = os.path.join(NW, "produktiv-vorher.txt")
    live = re.search(r"labwc_sitzung=([0-9,]*)", open(pv, encoding="utf-8").read() if os.path.exists(pv) else "")
    live_pids = {p for p in (live.group(1).split(",") if live else []) if p}
    fremd = "rc.xml.vorher-20261020-112812"
    erwartet = {"hell": "Querencia-Klassisch-Dunkel", "dunkel": "Querencia-Klassisch",
                "fremd": "Probe-Rahmen", "ohne": "Querencia-Klassisch"}
    for fall, mitte in erwartet.items():
        lauf = f"016-r-{fall}-de-1920"
        d = os.path.join(N, lauf)
        def lies(*teile):
            p = os.path.join(d, *teile)
            return open(p, "rb").read() if os.path.exists(p) else None
        vorher, zwischen, nachher = lies("labwc-vorher", "rc.xml"), lies("rc-zwischen.xml"), lies("labwc-nachher", "rc.xml")
        if None in (vorher, zwischen, nachher):
            pruef(g, f"{lauf}: rc.xml vorher/zwischen/nachher", False, "fehlt")
            continue
        name = re.search(rb"<theme>\s*<name>([^<]*)</name>", zwischen)
        pruef(g, f"{lauf}: Rahmen nach dem Umschalten {mitte}", name and name.group(1).decode() == mitte,
              name.group(1).decode() if name else None)
        if fall in ("fremd", "ohne"):
            pruef(g, f"{lauf}: rc.xml beim Umschalten nicht verändert", zwischen == vorher,
                  hashlib.sha256(zwischen).hexdigest()[:16])
        sv, sn = hashlib.sha256(vorher).hexdigest(), hashlib.sha256(nachher).hexdigest()
        pruef(g, f"{lauf}: rc.xml nach dem Zurückstellen byte-gleich (SHA256 vorher = nachher)", sv == sn,
              f"vorher {sv} nachher {sn}")
        dateien = sorted(os.listdir(os.path.join(d, "labwc-nachher")))
        pruef(g, f"{lauf}: keine eigene Sicherung, fremde {fremd} unverändert",
              not [f for f in dateien if ".einstellungen-vorher-" in f]
              and lies("labwc-nachher", fremd) == lies("labwc-vorher", fremd) is not None, dateien)
        u = (text(lauf, "umgebung.txt") or "")
        pid = re.search(r"LABWC_PID=(\d+)", u)
        pruef(g, f"{lauf}: labwc --reconfigure nur an den isolierten labwc",
              pid and live_pids and pid.group(1) not in live_pids
              and "XDG_RUNTIME_DIR=" + os.path.expanduser("~/.cache/qe01/") in u
              and " -C " + os.path.expanduser("~/.cache/qe01/") in u.split("cmdline=", 1)[-1],
              f"Sitzungs-labwc {sorted(live_pids)}; " + u.strip().replace("\n", " | ")[:300])
        log = applog(lauf)
        neu_geladen = log.count("labwc hat die Konfiguration neu geladen")
        pruef(g, f"{lauf}: zweimal übernommen; labwc neu geladen, wo rc.xml geschrieben wurde",
              log.count("Erscheinungsbild übernommen") == 2
              and neu_geladen == (2 if fall in ("hell", "dunkel") else 0), f"neu geladen {neu_geladen}×")
        # Seite Fensterdesign nach dem Umschalten: neu gelesen, zeigt den Rahmen aus rc.xml
        fd = [z for z in log.splitlines() if "Fensterdesign geladen" in z]
        pruef(g, f"{lauf}: Fensterdesign zeigt danach {mitte}", bool(fd) and f'thema: "{mitte}",' in fd[-1],
              fd[-1][-160:] if fd else "keine Zeile")
        for datei in ("01-start.png", "02-umgeschaltet.png", "03-fensterdesign.png", "04-zurueck.png"):
            p = os.path.join(d, datei)
            if os.path.exists(p):
                Image.open(p).convert("RGB").crop((300, 120, 1620, 960)).save(
                    os.path.join(BILDORDNER, f"rahmen-{fall}-{datei}"))
            pruef(g, f"{lauf}/{datei}", os.path.exists(p), "")


def abstuerze():
    laeufe = sorted(os.path.basename(d) for d in glob.glob(os.path.join(N, "016-[dfkpr][0-9-]*")))
    schlecht = [l for l in laeufe if "panicked" in (applog(l) + (text(l, "logs/eingabe.log") or ""))]
    pruef("alle", f"{len(laeufe)} Läufe ohne „panicked“", not schlecht and laeufe, schlecht)
    fehlt = [l for l in laeufe if "szenario exit=0" not in (text(l, "logs/eingabe.log") or "")]
    pruef("alle", "Szenario jedes Laufs vollständig (exit 0)", not fehlt, fehlt)


def main():
    funktion()
    fotos()
    d1()
    rahmen()
    abstuerze()
    ziel = sys.argv[1] if len(sys.argv) > 1 else os.path.join(NW, "gui-016.tsv")
    with open(ziel, "w", encoding="utf-8") as f:
        f.write("gruppe\tfall\tergebnis\tdetail\n")
        for z in zeilen:
            f.write("\t".join(z) + "\n")
    n = {k: sum(1 for z in zeilen if z[2] == k) for k in ("PASS", "FAIL", "INFO")}
    for z in zeilen:
        if z[2] != "PASS":
            print("\t".join(z))
    print(f"{n['PASS']} PASS / {n['FAIL']} FAIL / {n['INFO']} INFO → {ziel}")
    return 1 if n["FAIL"] else 0


if __name__ == "__main__":
    sys.exit(main())
