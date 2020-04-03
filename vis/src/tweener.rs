use std::fmt;
use piet::Color;
use crate::{Stroke, Fill};

pub trait Steppable: fmt::Debug {
    /// Note: `amount` is interpreted in tween space; the caller is
    /// assumed to constrain `amount` to the open unit interval.
    fn step(&mut self, target: &Self, amount: f64);
}

pub trait Tweenable: Steppable + Sized {
    /// Compute a set of passing positions between `self` and `other`
    /// extremes.
    ///
    /// The value returned is expected to be a `Vec` of no less than 2
    /// and no more than `max_inner` + 2 tweens equally spaced in
    /// tween space, i.e. without the application of time-mapping
    /// (easing).  All distances between two consecutive elements are
    /// assumed to be the same.  The first element must be equal to
    /// `self`, the last element must be equal to `other`.
    ///
    /// Default implementation returns a two-element `Vec` containing
    /// just the two extremes.  Reimplement, if there is a need for
    /// expensive high quality interpolation (e.g. hsv-based color
    /// interpolation) that shouldn't be performed within each call to
    /// `Steppable::step()`.
    #[allow(unused_variables)]
    fn breakdown(self, other: Self, max_inner: usize) -> Vec<Self> {
        vec![self, other]
    }
}

#[derive(Clone, Debug)]
pub struct Tweener<T: Tweenable + Clone> {
    breakpoints:  Vec<T>,
    num_segments: f64,
    position:     f64,
    value:        T,
}

impl<T: Tweenable + Clone> Tweener<T> {
    pub fn new(start: T, stop: T, max_inner: usize) -> Self {
        // eprintln!("Tweener {:?} -> {:?}", start, stop);

        let value = start.clone();
        let breakpoints = start.breakdown(stop, max_inner);
        let mut tweener = Tweener { breakpoints, num_segments: 0.0, position: 0.0, value };

        tweener.initialize();

        tweener
    }

    fn initialize(&mut self) {
        if self.breakpoints.is_empty() {
            self.breakpoints = vec![self.value.clone()];
        }

        let mut num_segments = self.breakpoints.len();

        if num_segments == 1 {
            self.breakpoints.push(self.breakpoints[0].clone());
        } else {
            num_segments -= 1;
        }

        self.num_segments = num_segments as f64;
        self.position = 0.0;
    }

    pub fn reverse(&mut self) {
        self.position = 0.0;
        self.value.clone_from(self.breakpoints.last().unwrap());
        self.breakpoints.reverse();
    }

    pub fn restart(&mut self, new_stop: T, max_inner: usize) {
        if let Some(new_start) = self.breakpoints.pop() {
            self.value.clone_from(&new_start);
            self.breakpoints = new_start.breakdown(new_stop, max_inner);
        } else {
            self.value.clone_from(&new_stop);
            self.breakpoints = vec![new_stop; 2];
        }

        self.initialize();
    }

    /// The `amount` is expected to be a strictly positive number such
    /// that the total amount accumulated across any sequence of calls
    /// to `tween_on()` until a call to `restart()` is less than or
    /// equal to 1.
    pub fn tween_on(&mut self, amount: f64) -> Option<&T> {
        if amount > 0.0 {
            let position = self.position + amount;

            if position < 1.0 {
                let findex = (position * self.num_segments).trunc() + 1.0;
                let target = &self.breakpoints[findex as usize];
                let target_position = findex / self.num_segments;
                // FIXME if possible, set self.value to pre-target and
                // increase self.position accordingly.

                self.value.step(target, amount / (target_position - self.position));
                self.position = position;

                Some(&self.value)
            } else {
                self.position = 1.0;

                self.breakpoints.last()
            }
        } else {
            None
        }
    }
}

/// A mapping from time to tween space.
pub trait Easing {
    /// Takes a point on the time axis expressed as a fraction of the
    /// total animation time.  Returns amount of change in the tween
    /// space.
    ///
    /// Note: this is meant to be calculated globally for entire
    /// [`Theme`].
    fn ease(&mut self, time: f64) -> f64;

    fn restart(&mut self);
}

#[derive(Default)]
pub struct LinearEasing {
    position: f64,
}

impl LinearEasing {
    pub fn new() -> Self {
        LinearEasing::default()
    }
}

impl Easing for LinearEasing {
    fn ease(&mut self, mut time: f64) -> f64 {
        if time >= 1.0 {
            time = 1.0;
        }

        if time > self.position {
            let result = time - self.position;

            self.position = time;

            result
        } else {
            0.0
        }
    }

    fn restart(&mut self) {
        self.position = 0.0;
    }
}

trait AsRgba {
    fn as_rgba(&self) -> (u32, u32, u32, u32);
    fn as_rgba_f64(&self) -> (f64, f64, f64, f64);
}

impl AsRgba for Color {
    fn as_rgba(&self) -> (u32, u32, u32, u32) {
        let rgba = self.as_rgba_u32();

        ((rgba >> 24) & 255, (rgba >> 16) & 255, (rgba >> 8) & 255, rgba & 255)
    }

    fn as_rgba_f64(&self) -> (f64, f64, f64, f64) {
        let rgba = self.as_rgba_u32();

        (
            ((rgba >> 24) & 255) as f64,
            ((rgba >> 16) & 255) as f64,
            ((rgba >> 8) & 255) as f64,
            (rgba & 255) as f64,
        )
    }
}

impl Steppable for Color {
    fn step(&mut self, target: &Self, amount: f64) {
        let (r0, g0, b0, a0) = self.as_rgba_f64();
        let (r1, g1, b1, a1) = target.as_rgba_f64();

        #[inline]
        fn lerp(v0: f64, v1: f64, amount: f64) -> u8 {
            let v = (v0 + (v1 - v0) * amount).round() as i32;
            let v = if v < 0 {
                0
            } else if v > 255 {
                255
            } else {
                v
            };

            v as u8
        }

        *self = Color::rgba8(
            lerp(r0, r1, amount),
            lerp(g0, g1, amount),
            lerp(b0, b1, amount),
            lerp(a0, a1, amount),
        );
    }
}

impl Tweenable for Color {
    fn breakdown(self, other: Self, max_inner: usize) -> Vec<Self> {
        if max_inner > 0 {
            let mut breakpoints = vec![self.clone()];
            let num_segments = (max_inner + 1) as f64;

            for i in 0..max_inner {
                let mut inner = self.clone();

                // FIXME use hsv
                inner.step(&other, (i + 1) as f64 / num_segments);
                breakpoints.push(inner);
            }

            breakpoints.push(other);

            breakpoints
        } else {
            vec![self, other]
        }
    }
}

impl Steppable for Stroke {
    fn step(&mut self, target: &Self, amount: f64) {
        let brush = self.get_mut_brush();
        brush.step(target.get_brush(), amount);

        let width = self.get_width();
        self.set_width(width + (target.get_width() - width) * amount);
    }
}

impl Tweenable for Stroke {
    fn breakdown(self, other: Self, max_inner: usize) -> Vec<Self> {
        let brush = self.get_brush().clone().breakdown(other.get_brush().clone(), max_inner);
        let num_breakpoints = brush.len();

        if num_breakpoints > 2 {
            let w0 = self.get_width();
            let w1 = other.get_width();
            let num_segments = (num_breakpoints - 1) as f64;

            brush
                .into_iter()
                .enumerate()
                .map(|(i, b)| {
                    Stroke::new().with_brush(b).with_width(w0 + (w1 - w0) * (i as f64 / num_segments))
                })
                .collect()
        } else {
            vec![self, other]
        }
    }
}

impl Steppable for Fill {
    fn step(&mut self, target: &Self, amount: f64) {
        match self {
            Fill::Color(c0) => match target {
                Fill::Color(c1) => c0.step(c1, amount),
                Fill::Linear(name) => {} // FIXME
                Fill::Radial(name) => {} // FIXME
            },
            Fill::Linear(name) => {} // FIXME
            Fill::Radial(name) => {} // FIXME
        }
    }
}

impl Tweenable for Fill {
    fn breakdown(self, other: Self, max_inner: usize) -> Vec<Self> {
        match self {
            Fill::Color(ref c0) => match other {
                Fill::Color(c1) => {
                    c0.clone().breakdown(c1, max_inner).into_iter().map(Fill::Color).collect()
                }
                _ => vec![self, other], // FIXME
            },
            _ => vec![self, other], // FIXME
        }
    }
}
