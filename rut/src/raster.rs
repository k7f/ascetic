use std::cmp::Ordering;
use tracing::trace;
use ascetic_vis::TranslateScale;

pub struct Raster {
    buffer:      Vec<u32>,
    width:       u32,
    height:      u32,
    is_dirty:    bool,
    was_applied: bool,
}

impl Raster {
    pub fn new(width: u32, height: u32) -> Self {
        let buffer = vec![0; (width * height) as usize];
        let is_dirty = true;
        let was_applied = false;

        Raster { buffer, width, height, is_dirty, was_applied }
    }

    #[inline]
    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.is_dirty = true;
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn apply(&mut self, frame: &mut [u8]) {
        if self.was_applied {
            trace!("raster::apply not needed");
        } else {
            trace!(
                "raster::apply buffer of {} to pixel frame of {}",
                self.buffer.len(),
                frame.len() / 4
            );

            for (dst, &src) in frame.chunks_exact_mut(4).zip(self.buffer.iter()) {
                dst[0] = (src >> 16) as u8;
                dst[1] = (src >> 8) as u8;
                dst[2] = src as u8;
                dst[3] = (src >> 24) as u8;
            }

            self.was_applied = true;
        }
    }

    pub fn redraw(
        &mut self,
        source: &[u8],
        source_width: u32,
        source_height: u32,
        transform: TranslateScale,
    ) -> Result<(), crate::Error> {
        trace!("raster::redraw rendered pixmap into buffer");
        // FIXME validate source size
        self.buffer.resize((self.width * self.height) as usize, 0);

        let x_offset = transform.as_tuple().0.x.round() as i64;
        let y_offset = transform.as_tuple().0.y.round() as i64;

        match x_offset.cmp(&0) {
            Ordering::Greater => {
                let pix_offset = x_offset as u32;

                if pix_offset < self.width {
                    let mut iter = self.buffer.iter_mut();
                    let pix_limit = self.width - pix_offset;

                    for _ in 0..pix_offset {
                        if let Some(pixel) = iter.next() {
                            *pixel = 0x00d0_d0d0;
                        } else {
                            unreachable!()
                        }
                    }

                    for (pos, pixel) in iter.enumerate() {
                        if pos as u32 % self.width < pix_limit {
                            let offset = pos * 4;
                            let (r, g, b) = (
                                source[offset] as u32,
                                source[offset + 1] as u32,
                                source[offset + 2] as u32,
                            );

                            *pixel = (r << 16) | (g << 8) | b;
                        } else {
                            *pixel = 0x00d0_d0d0;
                        }
                    }
                } else {
                    for pixel in self.buffer.iter_mut() {
                        *pixel = 0x00d0_d0d0;
                    }
                }
            }

            Ordering::Less => {
                let pix_offset = -x_offset as u32;

                if pix_offset < self.width {
                    for (pos, pixel) in self.buffer.iter_mut().enumerate() {
                        if (pos as u32 + pix_offset) % self.width >= pix_offset {
                            let offset = (pos + pix_offset as usize) * 4;
                            let (r, g, b) = (
                                source[offset] as u32,
                                source[offset + 1] as u32,
                                source[offset + 2] as u32,
                            );

                            *pixel = (r << 16) | (g << 8) | b;
                        } else {
                            *pixel = 0x00d0_d0d0;
                        }
                    }
                } else {
                    for pixel in self.buffer.iter_mut() {
                        *pixel = 0x00d0_d0d0;
                    }
                }
            }

            Ordering::Equal => {
                if source_width > self.width {
                    let mut source_iter = source.chunks(4);
                    let mut dest_iter = self.buffer.iter_mut();
                    let mut x_pos = 0;
                    let mut y_pos = 0;
                    for pixel in &mut dest_iter {
                        if x_pos < self.width {
                            x_pos += 1;
                        } else if y_pos < self.height {
                            x_pos = 1;
                            y_pos += 1;
                            if let Err(offset) =
                                source_iter.advance_by((source_width - self.width) as usize)
                            {
                                *pixel = 0;
                                break
                            }
                        } else {
                            *pixel = 0;
                            break
                        }

                        if let Some(chunk) = source_iter.next() {
                            let (r, g, b) = (chunk[0] as u32, chunk[1] as u32, chunk[2] as u32);

                            *pixel = (r << 16) | (g << 8) | b;
                        } else {
                            *pixel = 0;
                            break
                        }
                    }
                    for pixel in dest_iter {
                        *pixel = 0;
                    }
                } else if source_width < self.width {
                    let mut source_iter = source.chunks(4);
                    let mut dest_iter = self.buffer.iter_mut();
                    let mut x_pos = 0;
                    let mut y_pos = 0;
                    for pixel in self.buffer.iter_mut() {
                        if x_pos < source_width {
                            x_pos += 1;
                        } else if x_pos < self.width {
                            x_pos += 1;
                            *pixel = 0;
                            continue
                        } else if y_pos < self.height {
                            x_pos = 1;
                            y_pos += 1;
                        } else {
                            *pixel = 0;
                            break
                        }

                        if let Some(chunk) = source_iter.next() {
                            let (r, g, b) = (chunk[0] as u32, chunk[1] as u32, chunk[2] as u32);

                            *pixel = (r << 16) | (g << 8) | b;
                        } else if y_pos < self.height {
                            *pixel = 0;
                        } else {
                            // FIXME error
                            *pixel = 0;
                            break
                        }
                    }
                } else {
                    let mut dest_iter = self.buffer.iter_mut();
                    for (pixel, chunk) in (&mut dest_iter).zip(source.chunks(4)) {
                        let (r, g, b) = (chunk[0] as u32, chunk[1] as u32, chunk[2] as u32);

                        *pixel = (r << 16) | (g << 8) | b;
                    }
                    for pixel in dest_iter {
                        *pixel = 0;
                    }
                }
            }
        }

        self.is_dirty = false;
        self.was_applied = false;

        Ok(())
    }
}

impl std::fmt::Debug for Raster {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("Raster").field(&format_args!("..")).finish()
    }
}
