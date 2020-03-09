use std::{mem, fmt, error::Error};
use piet::{ErrorKind, ImageFormat};
use piet_cairo::CairoRenderContext;
use cairo::{Context, Format, ImageSurface};

#[derive(Debug)]
pub struct CairoError(cairo::Status);

impl fmt::Display for CairoError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for CairoError {}

pub struct BitmapDevice {
    surface: ImageSurface,
    ctx:     Context,
}

impl BitmapDevice {
    pub fn new(width: usize, height: usize, pix_scale: f64) -> Result<Self, piet::Error> {
        let surface = ImageSurface::create(Format::ARgb32, width as i32, height as i32)
            .map_err(CairoError)
            .map_err(Into::<Box<dyn Error>>::into)?;
        let ctx = Context::new(&surface);
        ctx.scale(pix_scale, pix_scale);

        Ok(BitmapDevice { surface, ctx })
    }

    pub fn render_context(&mut self) -> CairoRenderContext {
        CairoRenderContext::new(&mut self.ctx)
    }

    pub fn into_raw_pixels(mut self, fmt: ImageFormat) -> Result<Vec<u8>, piet::Error> {
        if fmt != ImageFormat::RgbaPremul {
            return Err(piet::new_error(ErrorKind::NotSupported))
        }

        mem::drop(self.ctx);
        self.surface.flush();

        let stride = self.surface.get_stride() as usize;
        let width = self.surface.get_width() as usize;
        let height = self.surface.get_height() as usize;
        let buf = self.surface.get_data().map_err(Into::<Box<dyn Error>>::into)?;
        let mut raw_data = vec![0; width * height * 4];

        for y in 0..height {
            let src_off = y * stride;
            let dst_off = y * width * 4;

            for x in 0..width {
                let src = &buf[src_off + x * 4..];
                let dst = &mut raw_data[dst_off + x * 4..];

                dst[0] = src[2];
                dst[1] = src[1];
                dst[2] = src[0];
                dst[3] = src[3];
            }
        }

        Ok(raw_data)
    }
}
