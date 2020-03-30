use std::{fmt, cmp::Ordering};
use ascetic_vis::TranslateScale;
use crate::ToyError;

pub struct Pixels {
    pixels:      Vec<u32>,
    width:       usize,
    height:      usize,
    is_dirty:    bool,
    was_applied: bool,
}

impl Pixels {
    pub fn new(width: usize, height: usize) -> Self {
        let pixels = vec![0; width * height];
        let is_dirty = true;
        let was_applied = false;

        Pixels { pixels, width, height, is_dirty, was_applied }
    }

    #[inline]
    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.is_dirty = true;
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn apply(&mut self) -> Option<(&[u32], usize, usize)> {
        if self.was_applied {
            None
        } else {
            self.was_applied = true;

            Some((self.pixels.as_slice(), self.width, self.height))
        }
    }

    pub fn redraw(&mut self, source: &[u8], transform: TranslateScale) -> Result<(), ToyError> {
        // FIXME validate source size
        self.pixels.resize(self.width * self.height, 0);

        let x_offset = transform.as_tuple().0.x.round() as i64;
        let y_offset = transform.as_tuple().0.y.round() as i64;

        match x_offset.cmp(&0) {
            Ordering::Greater => {
                let pix_offset = x_offset as usize;

                if pix_offset < self.width {
                    let mut iter = self.pixels.iter_mut();
                    let pix_limit = self.width - pix_offset;

                    for _ in 0..pix_offset {
                        if let Some(pixel) = iter.next() {
                            *pixel = 0x00d0_d0d0;
                        } else {
                            unreachable!()
                        }
                    }

                    for (pos, pixel) in iter.enumerate() {
                        if pos % self.width < pix_limit {
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
                    for pixel in self.pixels.iter_mut() {
                        *pixel = 0x00d0_d0d0;
                    }
                }
            }

            Ordering::Less => {
                let pix_offset = -x_offset as usize;

                if pix_offset < self.width {
                    for (pos, pixel) in self.pixels.iter_mut().enumerate() {
                        if (pos + pix_offset) % self.width >= pix_offset {
                            let offset = (pos + pix_offset) * 4;
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
                    for pixel in self.pixels.iter_mut() {
                        *pixel = 0x00d0_d0d0;
                    }
                }
            }

            Ordering::Equal => {
                for (pixel, chunk) in self.pixels.iter_mut().zip(source.chunks(4)) {
                    let (r, g, b) = (chunk[0] as u32, chunk[1] as u32, chunk[2] as u32);

                    *pixel = (r << 16) | (g << 8) | b;
                }
            }
        }

        self.is_dirty = false;
        self.was_applied = false;

        Ok(())
    }
}

impl fmt::Debug for Pixels {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Pixels").field(&format_args!("..")).finish()
    }
}
