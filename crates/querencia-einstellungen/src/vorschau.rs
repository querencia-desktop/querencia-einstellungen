//! Live-Vorschau: zeichnet einen Bildschirm der gewählten Auflösung verkleinert,
//! mit genau derselben Logik wie `querencia-desktop/src/wallpaper.rs`
//! (Farbe unten, Bild darüber per Snapshot) – plus Zentrieren und Kacheln.
use std::cell::{Cell, RefCell};

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gdk, glib, graphene, gsk};

use crate::modell::{self, Darstellung, Hintergrund, Typ};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Vorschau {
        pub einstellung: RefCell<Hintergrund>,
        pub textur: RefCell<Option<gdk::Texture>>,
        /// Nachgebildete Bildschirmgröße.
        pub flaeche: Cell<(f64, f64)>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Vorschau {
        const NAME: &'static str = "QuerenciaHintergrundVorschau";
        type Type = super::Vorschau;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("vorschau");
        }
    }

    impl ObjectImpl for Vorschau {
        fn constructed(&self) {
            self.parent_constructed();
            self.flaeche.set((1920.0, 1080.0));
        }
    }

    impl WidgetImpl for Vorschau {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let widget = self.obj();
            let (ww, wh) = (f64::from(widget.width()), f64::from(widget.height()));
            let flaeche = self.flaeche.get();
            let m = modell::vorschau_massstab(flaeche, (ww, wh));
            let (sw, sh) = (flaeche.0 * m, flaeche.1 * m);
            let (ox, oy) = ((ww - sw) / 2.0, (wh - sh) / 2.0);

            // Rahmen des nachgebildeten Bildschirms
            let rahmen = graphene::Rect::new(
                ox as f32 - 1.0,
                oy as f32 - 1.0,
                sw as f32 + 2.0,
                sh as f32 + 2.0,
            );
            // Dunkel (helle Schrift im Thema): Rand wie die Anordnungsfläche
            // auf der Seite Bildschirm, sonst verschwände er im Fenstergrund.
            let c = widget.color();
            let rand = if 0.2126 * c.red() + 0.7152 * c.green() + 0.0722 * c.blue() > 0.5 {
                gdk::RGBA::new(
                    0xbf as f32 / 255.0,
                    0xbb as f32 / 255.0,
                    0xb2 as f32 / 255.0,
                    1.0,
                )
            } else {
                gdk::RGBA::new(0.25, 0.25, 0.25, 1.0)
            };
            snapshot.append_color(&rand, &rahmen);

            snapshot.save();
            snapshot.translate(&graphene::Point::new(ox as f32, oy as f32));
            snapshot.scale(m as f32, m as f32);
            let bounds = graphene::Rect::new(0.0, 0.0, flaeche.0 as f32, flaeche.1 as f32);
            let e = self.einstellung.borrow();
            let (r, g, b) = e.farbe.anteile();
            snapshot.append_color(&gdk::RGBA::new(r, g, b, 1.0), &bounds);

            if e.typ == Typ::Bild
                && let Some(textur) = self.textur.borrow().as_ref()
            {
                let bild = (f64::from(textur.width()), f64::from(textur.height()));
                snapshot.push_clip(&bounds);
                if e.darstellung == Darstellung::Kacheln {
                    let kachel = graphene::Rect::new(0.0, 0.0, bild.0 as f32, bild.1 as f32);
                    snapshot.push_repeat(&bounds, Some(&kachel));
                    snapshot.append_scaled_texture(textur, gsk::ScalingFilter::Trilinear, &kachel);
                    snapshot.pop();
                } else {
                    let (x, y, w, h) = modell::platzierung(e.darstellung, bild, flaeche);
                    snapshot.append_scaled_texture(
                        textur,
                        gsk::ScalingFilter::Trilinear,
                        &graphene::Rect::new(x as f32, y as f32, w as f32, h as f32),
                    );
                }
                snapshot.pop();
            }
            snapshot.restore();
        }
    }
}

glib::wrapper! {
    pub struct Vorschau(ObjectSubclass<imp::Vorschau>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for Vorschau {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl Vorschau {
    pub fn zeigen(&self, einstellung: &Hintergrund, textur: Option<&gdk::Texture>) {
        self.imp().einstellung.replace(einstellung.clone());
        self.imp().textur.replace(textur.cloned());
        self.queue_draw();
    }

    pub fn set_flaeche(&self, breite: f64, hoehe: f64) {
        self.imp().flaeche.set((breite.max(1.0), hoehe.max(1.0)));
        self.queue_draw();
    }
}
