use winit::{window::Window, dpi::PhysicalSize};
use pixels::{Pixels, SurfaceTexture};

#[derive(Debug)]
pub struct Frame {
    pixels:     Pixels,
}

impl Frame {
    pub fn new(window: &Window) -> Result<Self, crate::Error> {
        let PhysicalSize { width, height } = window.inner_size();
        let pixels = {
            let surface_texture = SurfaceTexture::new(width, height, window);

            Pixels::new(width, height, surface_texture)?
        };

        Ok(Frame { pixels })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.pixels.resize_buffer(width, height);
        self.pixels.resize_surface(width, height);
    }

    pub fn get(&mut self) -> &mut [u8] {
        self.pixels.get_frame()
    }

    pub fn render(&mut self) -> Result<(), crate::Error> {
        self.pixels.render()?;

        Ok(())
    }
}
