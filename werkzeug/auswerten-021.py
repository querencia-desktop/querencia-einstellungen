#!/usr/bin/python3
"""Einstellungen 0.2.1: Auswertung von werkzeug/021-gui-lauf.sh (Zeile „Diese Sitzung“).

  p    je Fall die Zeile „Treiber gelesen“ im Protokoll der App wie erwartet
       (Karten mit Kernelmodul, Renderer ohne und mit den Software-Variablen,
       darstellung=…); --grafik-pruefen zuerst ohne die drei Variablen, ein
       zweites Mal mit ihnen nur im Fall mit Software-Variablen; kein
       PackageKit-Schreibvorgang; Fotos vorhanden, Ausschnitt nach
       nachweis/021/bilder/
  h    0.2.1-2 (P-2): querencia-sitzung nur unter ~/opt/querencia/bin des
       Profil-HOME: programm= ist dieser Pfad, Befund wie im Fall; in den
       p-Läufen programm=querencia-sitzung (Stub über PATH)
  d1   alt (0.2.0) ↔ neu, Seiten paarweise im Fensterausschnitt: Seitenleiste
       und Inhalt pixelgleich bis auf die Programmversion auf „Informationen“
       und das Uhrzeit-Band auf „Sprache“
  alle kein „panicked“, Szenario vollständig

  . ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh  # Pillow (PYTHONPATH)
  python3 werkzeug/auswerten-021.py <ziel.tsv>
"""
import glob
import os
import re
import sys

from PIL import Image, ImageChops

sys.dont_write_bytecode = True
P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
NW = os.path.join(P, os.environ.get("NW", "nachweis/021"))
BILDORDNER = os.path.join(NW, "bilder")
SEITEN = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels",
          "tastatur", "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "netzwerk",
          "sprache", "informationen"]
INHALT_X = 244
UHRZEIT = (470, 506)
VERSION = (180, 198)
OHNE = "LIBGL_ALWAYS_SOFTWARE= GALLIUM_DRIVER= MESA_LOADER_DRIVER_OVERRIDE="
MIT = "LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe MESA_LOADER_DRIVER_OVERRIDE=llvmpipe"
ZINK = "renderer=„zink Vulkan 1.4(NVIDIA GeForce RTX 3060 (NVK GA106) (MESA_NVK))“"
LLVM = "„llvmpipe (LLVM 21.1.8, 256 bits)“"
# (erwartete Teile der Befundzeile, zweite Messung mit den Variablen?)
ERWARTET = {
    "vm-llvmpipe": (("karten=[0000:00:01.0:virtio_gpu]", f"renderer={LLVM}", "sitzung=gleich",
                     "software-gl=false", "darstellung=SoftwareOhne3d"), False),
    "nvidia-nouveau": (("karten=[0000:01:00.0:nouveau]", ZINK, f"sitzung={LLVM}", "software-gl=true",
                        "darstellung=SoftwareUmgeschaltet"), True),
    "nvidia-nouveau-ohne-sw": (("karten=[0000:01:00.0:nouveau]", ZINK, "sitzung=gleich", "software-gl=false",
                                "darstellung=Frei"), False),
    "amd": (("karten=[0000:0a:00.0:amdgpu]", "renderer=„AMD Radeon RX 6600", "sitzung=gleich",
             "darstellung=Frei"), False),
    "intel": (("karten=[0000:00:02.0:i915]", "renderer=„Mesa Intel(R) Iris(R) Xe Graphics (ADL GT2)“",
               "sitzung=gleich", "darstellung=Frei"), False),
    "nvidia-original": (("karten=[0000:01:00.0:nvidia]", "renderer=„NVIDIA GeForce RTX 3060/PCIe/SSE2“",
                         "sitzung=gleich", "darstellung=Proprietaer"), False),
    "keine": (("karten=[]", "renderer=nicht gemessen", "sitzung=gleich", "darstellung=Unbekannt"), False),
}
zeilen = []


def pruef(gruppe, fall, ok, detail=""):
    zeilen.append((gruppe, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ")[:400]))


def text(lauf, datei):
    p = os.path.join(N, lauf, datei)
    return open(p, encoding="utf-8").read() if os.path.exists(p) else None


def applog(lauf):
    return text(lauf, os.path.join("logs", "app-direkt.log")) or ""


def befunde(lauf):
    return [z.split("Treiber gelesen: ", 1)[1] for z in applog(lauf).splitlines() if "Treiber gelesen: " in z]


def fotos():
    g = "Fotos"
    os.makedirs(BILDORDNER, exist_ok=True)
    laeufe = sorted(os.path.basename(d) for d in glob.glob(os.path.join(N, "021-p-*")))
    faelle = {(text(l, "treiber/fall.txt") or "").strip() for l in laeufe}
    pruef(g, "alle Fälle fotografiert", faelle == set(ERWARTET), sorted(faelle))
    heim = sorted(os.path.basename(d) for d in glob.glob(os.path.join(N, "021-h-*")))
    pruef("P-2", "zwei Läufe mit querencia-sitzung nur in ~/opt/querencia/bin", len(heim) == 2, heim)
    for lauf in laeufe + heim:
        b = befunde(lauf)
        if lauf.startswith("021-h-"):
            # HOME der App ist das Profil-HOME (isoliert.sh, QE_PROFIL_STABIL)
            ok = bool(b) and re.search(r" programm=/\S+/heim/opt/querencia/bin/querencia-sitzung$", b[0])
            pruef("P-2", f"{lauf}: programm=<Profil-HOME>/opt/querencia/bin/querencia-sitzung, Stub aufgerufen",
                  ok and "--grafik-pruefen" in (text(lauf, "treiber/aufrufe.log") or ""),
                  b[0][-160:] if b else "kein Befund")
        else:
            pruef("P-2", f"{lauf}: programm=querencia-sitzung (Stub über PATH)",
                  bool(b) and b[0].endswith(" programm=querencia-sitzung"), b[0][-120:] if b else "kein Befund")
    for lauf in laeufe + heim:
        fall = (text(lauf, "treiber/fall.txt") or "").strip()
        teile, zweite = ERWARTET.get(fall, (("?",), False))
        b = befunde(lauf)
        pruef(g, f"{lauf}: genau einmal gelesen", len(b) == 1, len(b))
        if b:
            fehlt = [e for e in teile if e not in b[0]]
            pruef(g, f"{lauf}: Befund wie Fall {fall}", not fehlt, f"fehlt {fehlt}; ist {b[0]}")
        auf = text(lauf, "treiber/aufrufe.log") or ""
        gp = [z for z in auf.splitlines() if "--grafik-pruefen" in z]
        soll = [OHNE, MIT] if zweite else [OHNE]
        pruef(g, f"{lauf}: --grafik-pruefen {'ohne, dann mit' if zweite else 'nur ohne'} Software-Variablen",
              len(gp) == len(soll) and all(s in z for s, z in zip(soll, gp)), gp)
        pruef(g, f"{lauf}: kein Schreibvorgang an PackageKit",
              not re.search(r"InstallPackages|RemovePackages", text(lauf, "treiber/pk.log") or ""), "")
        w = int(lauf.rsplit("-", 1)[1])
        for datei in ("01-oben.png", "02-unten.png"):
            p = os.path.join(N, lauf, datei)
            pruef(g, f"{lauf}/{datei}", os.path.exists(p), "")
            if os.path.exists(p):
                im = Image.open(p).convert("RGB")
                box = (520, 200, 1400, 860) if w == 1920 else (72, 44, 952, 704)
                im.crop(box).save(os.path.join(BILDORDNER, f"{lauf[6:]}-{datei[:2]}.png"))


def fenster(lauf, datei):
    p = os.path.join(N, lauf, datei)
    if not os.path.exists(p):
        return None
    im = Image.open(p).convert("RGB")
    b = int(lauf.rsplit("-", 1)[1])
    dx, dy = (0, 0) if b == 1920 else (-448, -156)
    return im.crop((520 + dx, 200 + dy, 1400 + dx, 850 + dy))


def d1():
    for k in ("de-1024", "en-1920"):
        alt, neu = f"021-d1-hell-alt-{k}", f"021-d1-hell-neu-{k}"
        g = f"d1 {k}"
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
            if s == "sprache":
                d.paste((0, 0, 0), (INHALT_X, UHRZEIT[0], w, UHRZEIT[1]))
            if s == "informationen":
                vb = d.crop((INHALT_X, VERSION[0], w, VERSION[1])).getbbox()
                pruef(g, f"{fall}: Versionszeile weicht höchstens in der Nummer ab",
                      vb is None or (vb[2] - vb[0] <= 40), vb)
                d.paste((0, 0, 0), (INHALT_X, VERSION[0], w, VERSION[1]))
            bb = d.getbbox()
            pruef(g, f"{fall}: Seitenleiste und Inhalt pixelgleich", bb is None, bb)


def abstuerze():
    laeufe = sorted(os.path.basename(d) for d in glob.glob(os.path.join(N, "021-[dp][0-9-]*")))
    schlecht = [l for l in laeufe if "panicked" in (applog(l) + (text(l, "logs/eingabe.log") or ""))]
    pruef("alle", f"{len(laeufe)} Läufe ohne „panicked“", not schlecht and laeufe, schlecht)
    fehlt = [l for l in laeufe if "szenario exit=0" not in (text(l, "logs/eingabe.log") or "")]
    pruef("alle", "Szenario jedes Laufs vollständig (exit 0)", not fehlt, fehlt)


def main():
    fotos()
    d1()
    abstuerze()
    ziel = sys.argv[1] if len(sys.argv) > 1 else os.path.join(NW, "gui-021.tsv")
    with open(ziel, "w", encoding="utf-8") as f:
        f.write("gruppe\tfall\tergebnis\tdetail\n")
        for z in zeilen:
            f.write("\t".join(z) + "\n")
    n = {k: sum(1 for z in zeilen if z[2] == k) for k in ("PASS", "FAIL")}
    for z in zeilen:
        if z[2] != "PASS":
            print("\t".join(z))
    print(f"{n['PASS']} PASS / {n['FAIL']} FAIL → {ziel}")
    return 1 if n["FAIL"] else 0


if __name__ == "__main__":
    sys.exit(main())
