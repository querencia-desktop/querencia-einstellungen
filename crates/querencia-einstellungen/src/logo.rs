//! Das offizielle Querencia-Logo C2, eingebettet wie im Panel (`data/logo/`,
//! byte-gleich mit `~/Downloads` bzw. W2 `hmp11a/logo-original`).
use gtk4::{gdk, glib};

const GROESSEN: &[(i32, &[u8])] = &[
    (16, include_bytes!("../data/logo/querencia-logo-16.png")),
    (22, include_bytes!("../data/logo/querencia-logo-22.png")),
    (24, include_bytes!("../data/logo/querencia-logo-24.png")),
    (32, include_bytes!("../data/logo/querencia-logo-32.png")),
    (48, include_bytes!("../data/logo/querencia-logo-48.png")),
    (64, include_bytes!("../data/logo/querencia-logo-64.png")),
    (128, include_bytes!("../data/logo/querencia-logo-128.png")),
    (256, include_bytes!("../data/logo/querencia-logo-256.png")),
];

/// Die kleinste eingebettete Größe, die mindestens `size` hat.
pub fn textur(size: i32) -> Option<gdk::Texture> {
    let (_, bytes) = GROESSEN
        .iter()
        .find(|(s, _)| *s >= size)
        .or_else(|| GROESSEN.last())?;
    gdk::Texture::from_bytes(&glib::Bytes::from_static(bytes)).ok()
}

pub fn bild(size: i32) -> gtk4::Image {
    let image = match textur(size) {
        Some(t) => gtk4::Image::from_paintable(Some(&t)),
        None => gtk4::Image::from_icon_name("image-missing"),
    };
    image.set_pixel_size(size);
    image
}
