#!/usr/bin/python3
"""Einstellungen 0.2.0: Auswertung von werkzeug/020-gui-lauf.sh (Seite „Treiber“).

  p    je Fall die Zeile „Treiber gelesen“ im Protokoll der App wie erwartet
       (Karten mit Kernelmodul, Treiber mit Zustand, Renderer, Software-OpenGL),
       --grafik-pruefen ohne die drei Software-Variablen aufgerufen, kein
       PackageKit-Schreibvorgang; Fotos vorhanden, Ausschnitt nach
       nachweis/020/bilder/
  w1   Installieren: InstallPackages mit Treiber und Kernelmodul-Paket, Hinweise
       gesetzt, Bestand danach installiert, „Neu starten …“ ruft
       querencia-sitzung --neustart (Stub, nichts ausgeführt)
  w2   Entfernen: RemovePackages mit dem ganzen 615-Zweig, allow_deps und
       autoremove false (sonst lehnt die Attrappe ab), allgemeine Pakete bleiben
  w3   polkit lehnt ab, w5 Paketsperre: Fehlermeldung, Bestand unverändert
  w4   ubuntu-drivers-common installiert, danach liest die Seite die Liste
  w6   Zurücksetzen: kein Schreibvorgang
  w7   wie w1 auf Englisch
  d1   alt (0.1.6-2) ↔ neu, Seiten paarweise im Fensterausschnitt: Inhalt
       pixelgleich bis auf die Programmversion auf „Informationen“ und das
       Uhrzeit-Band auf „Sprache“; Seitenleiste nur ab der Zeile „Treiber“
       abweichend (neuer Eintrag)
  alle kein „panicked“, Szenario vollständig

  . ~/projects/querencia-boot-splash/werkzeug/pil-umgebung.sh  # Pillow (PYTHONPATH)
  python3 werkzeug/auswerten-020.py <ziel.tsv>
"""
import glob
import os
import re
import sys

from PIL import Image, ImageChops

sys.dont_write_bytecode = True
P = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
N = os.path.join(P, "nachweis")
NW = os.path.join(P, os.environ.get("NW", "nachweis/020"))
BILDORDNER = os.path.join(NW, "bilder")
SEITEN = ["erscheinungsbild", "hintergrundbild", "symbole", "fensterdesign", "schriften", "panels",
          "tastatur", "maus", "bildschirm", "arbeitsflaechen", "standardanwendungen", "netzwerk",
          "sprache", "informationen"]
INHALT_X = 244
UHRZEIT = (470, 506)
VERSION = (180, 198)
SEITENLEISTE_AB = 462  # Zeile „Treiber“ im Fensterausschnitt (neuer Eintrag)
ZWEIG_615 = {"nvidia-driver-615-open", "linux-modules-nvidia-615-open-generic", "libnvidia-gl-615",
             "libnvidia-compute-615", "libnvidia-common-615", "nvidia-kernel-common-615", "nvidia-utils-615",
             "nvidia-firmware-615-615.48.03", "xserver-xorg-video-nvidia-615",
             "linux-modules-nvidia-615-open-6.20.0-11-generic"}
zeilen = []


def pruef(gruppe, fall, ok, detail=""):
    zeilen.append((gruppe, fall, "PASS" if ok else "FAIL", str(detail).replace("\t", " ")[:400]))


def info(gruppe, fall, detail):
    zeilen.append((gruppe, fall, "INFO", str(detail).replace("\t", " ")[:400]))


def text(lauf, datei):
    p = os.path.join(N, lauf, datei)
    return open(p, encoding="utf-8").read() if os.path.exists(p) else None


def applog(lauf):
    return text(lauf, os.path.join("logs", "app-direkt.log")) or ""


def befunde(lauf):
    return [z.split("Treiber gelesen: ", 1)[1] for z in applog(lauf).splitlines() if "Treiber gelesen: " in z]


def installiert(lauf, datei):
    return {z.split("\t")[0] for z in (text(lauf, f"treiber/{datei}") or "").splitlines() if z.strip()}


def pk(lauf):
    return text(lauf, "treiber/pk.log") or ""


ERWARTET = {
    "nvidia-nouveau": ("karten=[0000:01:00.0:nouveau]",
                       "0000:01:00.0[nvidia-driver-615-open=NichtInstalliert nvidia-driver-615=NichtInstalliert "
                       "nvidia-driver-580-server-open=NichtInstalliert xserver-xorg-video-nouveau=Aktiv]",
                       "renderer=„zink Vulkan 1.4(NVIDIA GeForce RTX 3060 (NVK GA106) (MESA_NVK))“", "software-gl=true"),
    "nvidia-original": ("karten=[0000:01:00.0:nvidia]",
                        "0000:01:00.0[nvidia-driver-615-open=Aktiv nvidia-driver-615=NichtInstalliert "
                        "nvidia-driver-580-server-open=NichtInstalliert xserver-xorg-video-nouveau=Verfuegbar]",
                        "renderer=„NVIDIA GeForce RTX 3060/PCIe/SSE2“", "software-gl=false"),
    "amd": ("karten=[0000:0a:00.0:amdgpu]", "0000:0a:00.0[xserver-xorg-video-amdgpu=Aktiv]",
            "renderer=„AMD Radeon RX 6600", "software-gl=false"),
    "intel": ("karten=[0000:00:02.0:i915]", "0000:00:02.0[xserver-xorg-video-intel=Aktiv]",
              "renderer=„Mesa Intel(R) Iris(R) Xe Graphics (ADL GT2)“", "software-gl=false"),
    "keine": ("karten=[]", "liste=[]", "renderer=nicht gemessen", "software-gl=false"),
    "werkzeug-fehlt": ("karten=[0000:01:00.0:nouveau]", "liste=[ubuntu-drivers fehlt]",
                       "renderer=„zink Vulkan", "software-gl=true"),
}


def fotos():
    g = "Fotos"
    os.makedirs(BILDORDNER, exist_ok=True)
    for lauf in sorted(os.path.basename(d) for d in glob.glob(os.path.join(N, "020-p-*"))):
        fall = (text(lauf, "treiber/fall.txt") or "").strip()
        b = befunde(lauf)
        pruef(g, f"{lauf}: genau einmal gelesen", len(b) == 1, len(b))
        if b:
            fehlt = [e for e in ERWARTET.get(fall, ("?",)) if e not in b[0]]
            pruef(g, f"{lauf}: Befund wie Fall {fall}", not fehlt, f"fehlt {fehlt}; ist {b[0]}")
        auf = text(lauf, "treiber/aufrufe.log") or ""
        gp = [z for z in auf.splitlines() if "--grafik-pruefen" in z]
        pruef(g, f"{lauf}: --grafik-pruefen ohne Software-Variablen",
              len(gp) == 1 and "LIBGL_ALWAYS_SOFTWARE= GALLIUM_DRIVER= MESA_LOADER_DRIVER_OVERRIDE=" in gp[0], gp)
        pruef(g, f"{lauf}: kein Schreibvorgang an PackageKit",
              not re.search(r"InstallPackages|RemovePackages", pk(lauf)), "")
        w = int(lauf.rsplit("-", 1)[1])
        for datei in ("01-oben.png", "02-unten.png"):
            p = os.path.join(N, lauf, datei)
            pruef(g, f"{lauf}/{datei}", os.path.exists(p), "")
            if os.path.exists(p):
                im = Image.open(p).convert("RGB")
                box = (520, 200, 1400, 860) if w == 1920 else (72, 44, 952, 704)
                im.crop(box).save(os.path.join(BILDORDNER, f"{lauf[6:]}-{datei[:2]}.png"))


def schreibvorgaenge(lauf, art):
    return [z for z in pk(lauf).splitlines() if f" {art} " in z]


def funktion():
    g = "Wechsel"
    for lauf, sprache in (("020-w1-installieren-de-1920", "de"), ("020-w7-installieren-en-1920", "en")):
        inst = schreibvorgaenge(lauf, "InstallPackages")
        pruef(g, f"{lauf}: eine Installation mit Treiber und Kernelmodul-Paket",
              len(inst) == 1 and "nvidia-driver-615-open;" in inst[0]
              and "linux-modules-nvidia-615-open-generic;" in inst[0], inst)
        pruef(g, f"{lauf}: Hinweise locale/interactive/background gesetzt",
              "'interactive=true'" in pk(lauf) and "'background=false'" in pk(lauf), "")
        pruef(g, f"{lauf}: nichts entfernt", not schreibvorgaenge(lauf, "RemovePackages"), "")
        vor, nach = installiert(lauf, "dpkg-vorher.txt"), installiert(lauf, "dpkg.txt")
        pruef(g, f"{lauf}: Bestand danach = vorher + 2 Pakete",
              nach - vor == {"nvidia-driver-615-open", "linux-modules-nvidia-615-open-generic"} and vor <= nach,
              sorted(nach - vor))
        b = befunde(lauf)
        pruef(g, f"{lauf}: danach neu gelesen, 615-open installiert (nach Neustart)",
              len(b) == 2 and "nvidia-driver-615-open=NachNeustart" in b[1], b[-1:] if b else b)
        auf = text(lauf, "treiber/aufrufe.log") or ""
        pruef(g, f"{lauf}: „Neu starten …“ ruft querencia-sitzung --neustart (Stub)",
              "querencia-sitzung --neustart" in auf and "NEUSTART-RÜCKFRAGE" in auf, "")
        for d in ("01-dialog.png", "02-fortschritt.png", "03-fertig.png"):
            p = os.path.join(N, lauf, d)
            pruef(g, f"{lauf}/{d}", os.path.exists(p), "")
            if os.path.exists(p):
                Image.open(p).convert("RGB").crop((520, 200, 1400, 860)).save(
                    os.path.join(BILDORDNER, f"wechsel-installieren-{sprache}-{d[:2]}.png"))
    lauf = "020-w2-entfernen-de-1920"
    ent = schreibvorgaenge(lauf, "RemovePackages")
    namen = set(re.findall(r"'([a-z0-9.+-]+);", ent[0])) if ent else set()
    pruef(g, f"{lauf}: eine Entfernung mit dem ganzen 615-Zweig", len(ent) == 1 and namen == ZWEIG_615,
          f"zu viel {sorted(namen - ZWEIG_615)}, fehlt {sorted(ZWEIG_615 - namen)}")
    pruef(g, f"{lauf}: allow_deps und autoremove false", bool(ent) and ent[0].rstrip().endswith("False, False)"),
          ent[0][-40:] if ent else "")
    nach = installiert(lauf, "dpkg.txt")
    pruef(g, f"{lauf}: danach kein 615-Paket, allgemeine bleiben",
          not (nach & ZWEIG_615) and {"libnvidia-egl-wayland1", "nvidia-settings", "xserver-xorg-video-nouveau"} <= nach,
          sorted(nach))
    b = befunde(lauf)
    pruef(g, f"{lauf}: danach neu gelesen, nvidia noch geladen, nouveau verfügbar",
          len(b) == 2 and "nvidia-driver-615-open=NichtInstalliert" in b[1]
          and "xserver-xorg-video-nouveau=Verfuegbar" in b[1], b[-1:])
    for lauf, grund in (("020-w3-polkit-de-1920", "not-authorized"), ("020-w5-sperre-de-1920", "Paketsperre")):
        pruef(g, f"{lauf}: Fehler ({grund}) gemeldet, Bestand unverändert",
              installiert(lauf, "dpkg.txt") == installiert(lauf, "dpkg-vorher.txt")
              and "querencia-einstellungen: Treiber: " in applog(lauf), applog(lauf).count("Treiber: "))
        p = os.path.join(N, lauf, "02-fehler.png")
        pruef(g, f"{lauf}/02-fehler.png", os.path.exists(p), "")
        if os.path.exists(p):
            Image.open(p).convert("RGB").crop((520, 200, 1400, 860)).save(
                os.path.join(BILDORDNER, f"fehler-{grund}.png"))
    pruef(g, "020-w3-polkit-de-1920: NichtBerechtigt oder KeinAgent",
          re.search(r"Treiber: (NichtBerechtigt|KeinAgent)", applog("020-w3-polkit-de-1920")) is not None, "")
    pruef(g, "020-w5-sperre-de-1920: SperreBelegt", "Treiber: SperreBelegt" in applog("020-w5-sperre-de-1920"), "")
    lauf = "020-w4-werkzeug-de-1920"
    inst = schreibvorgaenge(lauf, "InstallPackages")
    pruef(g, f"{lauf}: ubuntu-drivers-common installiert",
          len(inst) == 1 and "ubuntu-drivers-common;" in inst[0] and "ubuntu-drivers-common" in installiert(lauf, "dpkg.txt"),
          inst)
    b = befunde(lauf)
    pruef(g, f"{lauf}: erst „fehlt“, danach die Liste",
          len(b) == 2 and "ubuntu-drivers fehlt" in b[0] and "xserver-xorg-video-nouveau=Aktiv" in b[1], b)
    for d in ("01-fehlt.png", "02-liste.png"):
        p = os.path.join(N, lauf, d)
        pruef(g, f"{lauf}/{d}", os.path.exists(p), "")
        if os.path.exists(p):
            Image.open(p).convert("RGB").crop((520, 200, 1400, 860)).save(os.path.join(BILDORDNER, f"werkzeug-{d[:2]}.png"))
    lauf = "020-w6-zuruecksetzen-de-1920"
    pruef(g, f"{lauf}: kein Schreibvorgang", not re.search(r"InstallPackages|RemovePackages", pk(lauf)), "")
    for d in ("01-gewaehlt.png", "02-zurueck.png"):
        pruef(g, f"{lauf}/{d}", os.path.exists(os.path.join(N, lauf, d)), "")
    a, z = (os.path.join(N, lauf, d) for d in ("00-start.png", "02-zurueck.png"))
    if os.path.exists(a) and os.path.exists(z):
        # Nur der Seiteninhalt rechts der Seitenleiste bis vor den
        # Overlay-Rollbalken: Fokus (Seitenleiste, Fenstertitel) und der nach
        # einem Klick eingeblendete Rollbalken gehören nicht zur Auswahl.
        box = (764, 298, 1374, 853)
        d = ImageChops.difference(Image.open(a).convert("RGB").crop(box), Image.open(z).convert("RGB").crop(box))
        bb = d.getbbox()
        pruef(g, f"{lauf}: nach „Zurücksetzen“ Seiteninhalt wie am Anfang (pixelgleich)", bb is None, bb)


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
        alt, neu = f"020-d1-hell-alt-{k}", f"020-d1-hell-neu-{k}"
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
            sb = d.crop((0, 0, INHALT_X, h)).getbbox()
            pruef(g, f"{fall}: Seitenleiste nur ab der Zeile „Treiber“ anders", sb is None or sb[1] >= SEITENLEISTE_AB, sb)
            d.paste((0, 0, 0), (0, 0, INHALT_X, h))
            if s == "sprache":
                d.paste((0, 0, 0), (INHALT_X, UHRZEIT[0], w, UHRZEIT[1]))
            if s == "informationen":
                vb = d.crop((INHALT_X, VERSION[0], w, VERSION[1])).getbbox()
                pruef(g, f"{fall}: Versionszeile weicht höchstens in der Nummer ab",
                      vb is None or (vb[2] - vb[0] <= 40), vb)
                d.paste((0, 0, 0), (INHALT_X, VERSION[0], w, VERSION[1]))
            bb = d.getbbox()
            if bb is not None:
                markierung(a, b, f"abweichung-{k}-{s}.png")
            pruef(g, f"{fall}: Inhalt pixelgleich", bb is None, bb)


def abstuerze():
    laeufe = sorted(os.path.basename(d) for d in glob.glob(os.path.join(N, "020-[dpw][0-9-]*")))
    schlecht = [l for l in laeufe if "panicked" in (applog(l) + (text(l, "logs/eingabe.log") or ""))]
    pruef("alle", f"{len(laeufe)} Läufe ohne „panicked“", not schlecht and laeufe, schlecht)
    fehlt = [l for l in laeufe if "szenario exit=0" not in (text(l, "logs/eingabe.log") or "")]
    pruef("alle", "Szenario jedes Laufs vollständig (exit 0)", not fehlt, fehlt)


def main():
    fotos()
    funktion()
    d1()
    abstuerze()
    ziel = sys.argv[1] if len(sys.argv) > 1 else os.path.join(NW, "gui-020.tsv")
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
