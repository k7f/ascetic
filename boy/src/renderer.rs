use std::fmt;
use ascetic_vis::{Scene, Theme, CairoBitmapDevice, ImageFormat, TranslateScale};
use crate::BoyError;

pub struct Renderer {
    buffer:   Option<Vec<u8>>,
    size:     (f64, f64),
    margin:   (f64, f64),
    is_dirty: bool,
}

impl Renderer {
    pub fn new(size: (f64, f64), margin: (f64, f64)) -> Self {
        Renderer { buffer: None, size, margin, is_dirty: true }
    }

    pub fn set_size(&mut self, size: (f64, f64)) {
        self.size = size;
        self.is_dirty = true;
    }

    #[inline]
    pub fn get_pix_size(&self) -> (usize, usize) {
        (self.size.0.round() as usize, self.size.1.round() as usize)
    }

    #[inline]
    pub fn get_buffer(&self) -> Option<&[u8]> {
        self.buffer.as_deref()
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
    ) -> Result<(), BoyError> {
        let (pix_width, pix_height) = self.get_pix_size();
        let pix_scale = transform.as_tuple().1;
        let mut device = CairoBitmapDevice::new(pix_width, pix_height, pix_scale)?;
        let mut rc = device.render_context();

        scene.render(theme, self.size, self.margin, &mut rc)?;

        match device.into_raw_pixels(ImageFormat::RgbaPremul) {
            Ok(buffer) => {
                self.buffer = Some(buffer);
                self.is_dirty = false;

                Ok(())
            }
            Err(err) => {
                self.buffer = None;
                self.is_dirty = true;

                Err(err.into())
            }
        }
    }
}

impl fmt::Debug for Renderer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Renderer").field(&format_args!("..")).finish()
    }
}
