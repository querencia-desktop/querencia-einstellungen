//! Bereich „Standardanwendungen“ (QD-046 ①, G2-Z): die Kategorien wie in
//! den „Bevorzugten Anwendungen“ von GNOME 2 – Tabelle und reine Logik, ohne
//! GIO. Kategorien, MIME-Typen und Vorschläge stehen **nur hier**; die Seite
//! (`standardanwendungen.rs`) und `docs/standardanwendungen.md` folgen ihr.
//!
//! Regel beim Festlegen (wie das GNOME-Kontrollzentrum): der Haupttyp (erster
//! Eintrag) wird immer gesetzt, die übrigen Typen nur, wenn die gewählte
//! Anwendung sie in ihrem `MimeType=` nennt. So bekommt keine Anwendung
//! Dateitypen zugeordnet, die sie nicht öffnen kann.
use crate::i18n::gettext_noop;

/// Wie eine Kategorie bestimmt wird.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Art {
    /// Über MIME-Typen (bzw. `x-scheme-handler/…`), wählbar, schreibt über GIO.
    Mime,
    /// Nur Anzeige: es gibt keinen MIME-Typ und keinen GIO-Schreibweg.
    NurAnzeige,
}

#[derive(Debug)]
pub struct Kategorie {
    /// Fester Schlüssel (Diagnoseausgabe, Prüfwerkzeuge).
    pub schluessel: &'static str,
    /// Beschriftung auf der Seite.
    pub name: &'static str,
    pub art: Art,
    /// Erster Eintrag = Haupttyp: bestimmt die Anzeige und die Auswahlliste.
    pub typen: &'static [&'static str],
    /// Vorschlag (Desktop-ID), nur als Markierung in der Auswahl – wird nie
    /// von selbst gesetzt.
    pub vorschlag: Option<&'static str>,
}

/// Querencia Terminal: Starter seit 20.10.2026 (QD-066).
pub const TERMINAL_ID: &str = "org.querencia.Terminal.desktop";
/// Alte ID, auf älteren Installationen der einzige Starter; heute nur noch
/// eine Brücke (NoDisplay), die dasselbe Terminal startet.
pub const TERMINAL_BRUECKE: &str = "org.example.Querencia.Terminal.desktop";

/// Der Starter des Terminals: der neue, sonst die Brücke, falls nur sie
/// installiert ist; fehlen beide, der neue (wird als nicht installiert gezeigt).
pub fn terminal_id(installiert: impl Fn(&str) -> bool) -> &'static str {
    if !installiert(TERMINAL_ID) && installiert(TERMINAL_BRUECKE) {
        TERMINAL_BRUECKE
    } else {
        TERMINAL_ID
    }
}

pub const KATEGORIEN: &[Kategorie] = &[
    Kategorie {
        schluessel: "webbrowser",
        name: gettext_noop("Web Browser"),
        art: Art::Mime,
        typen: &[
            "x-scheme-handler/http",
            "x-scheme-handler/https",
            "text/html",
            "application/xhtml+xml",
        ],
        vorschlag: None,
    },
    Kategorie {
        schluessel: "e-mail",
        name: gettext_noop("Email"),
        art: Art::Mime,
        typen: &["x-scheme-handler/mailto"],
        vorschlag: None,
    },
    Kategorie {
        schluessel: "texteditor",
        name: gettext_noop("Text Editor"),
        art: Art::Mime,
        typen: &["text/plain", "text/markdown", "text/x-markdown"],
        vorschlag: Some("org.querencia.Editor.desktop"),
    },
    Kategorie {
        schluessel: "terminal",
        name: gettext_noop("Terminal"),
        art: Art::NurAnzeige,
        typen: &[],
        vorschlag: Some(TERMINAL_ID),
    },
    Kategorie {
        schluessel: "bilder",
        name: gettext_noop("Images"),
        art: Art::Mime,
        typen: &[
            "image/png",
            "image/jpeg",
            "image/gif",
            "image/webp",
            "image/bmp",
            "image/svg+xml",
            "image/tiff",
        ],
        vorschlag: Some("org.querencia.Bilder.desktop"),
    },
    Kategorie {
        schluessel: "musik",
        name: gettext_noop("Music"),
        art: Art::Mime,
        typen: &[
            "audio/mpeg",
            "audio/ogg",
            "audio/x-vorbis+ogg",
            "audio/flac",
            "audio/x-wav",
            "audio/mp4",
        ],
        vorschlag: None,
    },
    Kategorie {
        schluessel: "video",
        name: gettext_noop("Video"),
        art: Art::Mime,
        typen: &[
            "video/mp4",
            "video/webm",
            "video/x-matroska",
            "video/ogg",
            "video/mpeg",
        ],
        vorschlag: None,
    },
    Kategorie {
        schluessel: "dateimanager",
        name: gettext_noop("File Manager"),
        art: Art::Mime,
        typen: &["inode/directory"],
        vorschlag: Some("io.github.lgse.Strata.desktop"),
    },
    Kategorie {
        schluessel: "pdf",
        name: gettext_noop("PDF Documents"),
        art: Art::Mime,
        typen: &["application/pdf"],
        vorschlag: None,
    },
    Kategorie {
        schluessel: "archive",
        name: gettext_noop("Archives"),
        art: Art::Mime,
        typen: &[
            "application/zip",
            "application/x-tar",
            "application/x-compressed-tar",
            "application/gzip",
            "application/x-xz-compressed-tar",
            "application/x-7z-compressed",
        ],
        vorschlag: None,
    },
];

impl Kategorie {
    pub fn haupttyp(&self) -> Option<&'static str> {
        self.typen.first().copied()
    }

    /// Die Typen, die für eine Anwendung mit diesen `MimeType=`-Einträgen
    /// gesetzt werden: Haupttyp immer, die anderen nur, wenn unterstützt.
    pub fn zu_setzende_typen<S: AsRef<str>>(&self, unterstuetzt: &[S]) -> Vec<&'static str> {
        self.typen
            .iter()
            .enumerate()
            .filter(|(i, t)| *i == 0 || unterstuetzt.iter().any(|u| u.as_ref() == **t))
            .map(|(_, t)| *t)
            .collect()
    }
}

#[cfg(test)]
pub fn kategorie(schluessel: &str) -> Option<&'static Kategorie> {
    KATEGORIEN.iter().find(|k| k.schluessel == schluessel)
}

/// Die Einträge einer Gruppe einer `mimeapps.list` (Typ → Desktop-IDs),
/// nur lesend, für Anzeige und Prüfung. Kommentare und leere Werte entfallen.
pub fn gruppe_lesen(text: &str, gruppe: &str) -> Vec<(String, Vec<String>)> {
    let mut drin = false;
    let mut aus = Vec::new();
    for zeile in text.lines() {
        let z = zeile.trim();
        if z.is_empty() || z.starts_with('#') {
            continue;
        }
        if z.starts_with('[') && z.ends_with(']') {
            drin = &z[1..z.len() - 1] == gruppe;
            continue;
        }
        if !drin {
            continue;
        }
        if let Some((k, v)) = z.split_once('=') {
            let ids: Vec<String> = v
                .split(';')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            aus.push((k.trim().to_string(), ids));
        }
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabelle_stimmig() {
        let mut schluessel: Vec<_> = KATEGORIEN.iter().map(|k| k.schluessel).collect();
        schluessel.sort();
        schluessel.dedup();
        assert_eq!(schluessel.len(), KATEGORIEN.len(), "Schlüssel doppelt");
        for k in KATEGORIEN {
            match k.art {
                Art::Mime => assert!(!k.typen.is_empty(), "{}", k.name),
                Art::NurAnzeige => assert!(k.typen.is_empty(), "{}", k.name),
            }
            if let Some(v) = k.vorschlag {
                assert!(v.ends_with(".desktop"), "{v}");
            }
            for t in k.typen {
                assert!(t.contains('/') && !t.contains(';'), "{t}");
            }
        }
        // Auftrag: Texteditor umfasst text/plain und text/markdown.
        let te = kategorie("texteditor").unwrap();
        assert_eq!(te.haupttyp(), Some("text/plain"));
        assert!(te.typen.contains(&"text/markdown"));
        assert_eq!(te.vorschlag, Some("org.querencia.Editor.desktop"));
        assert_eq!(
            kategorie("bilder").unwrap().vorschlag,
            Some("org.querencia.Bilder.desktop")
        );
        assert_eq!(
            kategorie("dateimanager").unwrap().vorschlag,
            Some("io.github.lgse.Strata.desktop")
        );
        assert_eq!(kategorie("terminal").unwrap().vorschlag, Some(TERMINAL_ID));
        assert_eq!(TERMINAL_ID, "org.querencia.Terminal.desktop");
        assert!(kategorie("gibtsnicht").is_none());
    }

    #[test]
    fn haupttyp_immer_rest_nur_unterstuetzt() {
        let te = kategorie("texteditor").unwrap();
        // Vim nennt text/markdown nicht: nur text/plain.
        assert_eq!(
            te.zu_setzende_typen(&["text/plain", "text/x-c"]),
            vec!["text/plain"]
        );
        // Querencia Editor nennt alle drei.
        assert_eq!(
            te.zu_setzende_typen(&["text/x-markdown", "text/plain", "text/markdown"]),
            vec!["text/plain", "text/markdown", "text/x-markdown"]
        );
        // Haupttyp auch dann, wenn die Anwendung ihn nicht nennt (Auswahl
        // stammt aus all_for_type des Haupttyps, also kommt das nur über
        // Unterklassen vor).
        assert_eq!(te.zu_setzende_typen::<&str>(&[]), vec!["text/plain"]);
        let web = kategorie("webbrowser").unwrap();
        assert_eq!(
            web.zu_setzende_typen(&["x-scheme-handler/https", "text/html"]),
            vec![
                "x-scheme-handler/http",
                "x-scheme-handler/https",
                "text/html"
            ]
        );
        assert!(
            kategorie("terminal")
                .unwrap()
                .zu_setzende_typen(&["x"])
                .is_empty()
        );
    }

    #[test]
    fn gruppe_lesen_wie_mimeapps_list() {
        let text = "# Kommentar\n[Added Associations]\ntext/plain=vim.desktop;\n\n\
                    [Default Applications]\ntext/plain=org.querencia.Editor.desktop\n\
                    image/png = a.desktop;b.desktop;\nleer=\n[Removed Associations]\nimage/png=c.desktop\n";
        let d = gruppe_lesen(text, "Default Applications");
        assert_eq!(
            d,
            vec![
                (
                    "text/plain".into(),
                    vec!["org.querencia.Editor.desktop".to_string()]
                ),
                (
                    "image/png".into(),
                    vec!["a.desktop".to_string(), "b.desktop".to_string()]
                ),
                ("leer".into(), vec![]),
            ]
        );
        assert_eq!(gruppe_lesen(text, "Added Associations").len(), 1);
        assert!(gruppe_lesen(text, "Fehlt").is_empty());
        assert!(gruppe_lesen("", "Default Applications").is_empty());
    }

    #[test]
    fn terminal_neu_vor_bruecke() {
        let beide = |i: &str| i == TERMINAL_ID || i == TERMINAL_BRUECKE;
        assert_eq!(terminal_id(beide), TERMINAL_ID);
        assert_eq!(terminal_id(|i| i == TERMINAL_BRUECKE), TERMINAL_BRUECKE);
        assert_eq!(terminal_id(|i| i == TERMINAL_ID), TERMINAL_ID);
        assert_eq!(terminal_id(|_| false), TERMINAL_ID);
    }
}
