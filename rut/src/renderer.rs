use tracing::trace;
use ascetic_vis::{
    Scene, Theme, ImageFormat, TranslateScale,
    backend::usvg::{Tree, Pixmap, FitTo, AsUsvgTree, render_to_pixmap},
};

pub struct Renderer {
    pixmap:   Option<Pixmap>,
    size:     (f64, f64),
    margin:   (f64, f64),
    is_dirty: bool,
}

impl Renderer {
    pub fn new(size: (f64, f64), margin: (f64, f64)) -> Self {
        let pixmap = Pixmap::new(size.0.round() as u32, size.1.round() as u32);

        Renderer { pixmap, size, margin, is_dirty: true }
    }

    pub fn set_size(&mut self, size: (f64, f64)) {
        self.size = size;
        self.is_dirty = true;
    }

    #[inline]
    pub fn get_pix_size(&self) -> (u32, u32) {
        (self.size.0.round() as u32, self.size.1.round() as u32)
    }

    #[inline]
    pub fn get_buffer(&self) -> Option<&[u8]> {
        self.pixmap.as_ref().map(|p| p.data())
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn render(
        &mut self,
        scene: &Scene,
        theme: &Theme,
        transform: TranslateScale,
    ) -> Result<(), crate::Error> {
        trace!("renderer::render scene into pixmap");
        if let Some(ref mut pixmap) = self.pixmap.as_mut() {
            let rtree = scene.as_usvg_tree(theme, self.size, self.margin);
            let (_pix_translate, pix_scale) = transform.as_tuple();

            // FIXME
            let result = if let Some(zoom) = if pix_scale < 0.001 {
                Some(0.001)
            } else if pix_scale > 1000.0 {
                Some(1000.0)
            } else if (pix_scale - 1.0).abs() < 0.001 {
                None
            } else {
                Some(pix_scale as f32)
            } {
                render_to_pixmap(&rtree, FitTo::Zoom(zoom), pixmap.as_mut())
            } else {
                render_to_pixmap(&rtree, FitTo::Original, pixmap.as_mut())
            };

            if result.is_some() {
                self.is_dirty = false;

                Ok(())
            } else {
                self.is_dirty = true;

                Err(crate::Error::PixmapRenderingFailure)
            }
        } else {
            self.is_dirty = true;
            Err(crate::Error::MissingPixmap)
        }
    }
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("Renderer").field(&format_args!("..")).finish()
    }
}
