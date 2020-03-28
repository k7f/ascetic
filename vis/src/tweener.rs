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
    /// The value returned is assumed to be a `Vec` of no more than
    /// `max_subdivision` tweens equally spaced in tween space,
    /// i.e. without the application of time-mapping (easing).  All
    /// distances are required to be the same: from `self` to first
    /// element, from last element to `other`, and between any two
    /// consecutive elements.
    ///
    /// Default implementation returns an empty `Vec`.  Reimplement,
    /// if there is a need for expensive high quality interpolation
    /// (e.g. hsv-based color interpolation) that shouldn't be
    /// performed within each call to `Steppable::step()`.
    #[allow(unused_variables)]
    fn breakdown(&self, other: &Self, max_subdivision: usize) -> Vec<Self> {
        Vec::new()
    }
}

// FIXME clone only in the ctor
#[derive(Clone, Debug)]
pub struct Tweener<T: Tweenable + Clone> {
    start:    T,
    targets:  Vec<T>,
    total:    f64,
    position: f64,
    value:    T,
}

impl<T: Tweenable + Clone> Tweener<T> {
    pub fn new(start: T, stop: T, max_subdivision: usize) -> Self {
        // eprintln!("Tweener {:?} -> {:?}", start, stop);

        let mut targets = start.breakdown(&stop, max_subdivision);
        targets.push(stop);

        let total = targets.len() as f64;
        let position = 0.0;
        let value = start.clone();

        Tweener { start, targets, total, position, value }
    }

    pub fn restart(&mut self) {
        self.position = 0.0;
        self.value = self.start.clone();
    }

    /// The `amount` is expected to be a strictly positive number such
    /// that the total amount accumulated across any sequence of calls
    /// to `tween_on()` until a call to `restart()` is less than 1.
    pub fn tween_on(&mut self, amount: f64) -> &T {
        if amount > 0.0 {
            let position = self.position + amount;

            if position < 1.0 {
                let findex = (position * self.total).trunc();
                let target = &self.targets[findex as usize];
                let target_position = (findex + 1.0) / self.total;
                // FIXME if possible, set self.value to pre-target and
                // increase self.position accordingly.

                self.value.step(target, amount / (target_position - self.position));
                self.position = position;
            } else {
                let target = self.targets.last().unwrap();

                self.value = target.clone();
                self.position = 1.0;
            }
        }

        &self.value
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
}

impl AsRgba for Color {
    fn as_rgba(&self) -> (u32, u32, u32, u32) {
        let rgba = self.as_rgba_u32();

        ((rgba >> 24) & 255, (rgba >> 16) & 255, (rgba >> 8) & 255, rgba & 255)
    }
}

impl Steppable for Color {
    fn step(&mut self, target: &Self, amount: f64) {
        let (r0, g0, b0, a0) = self.as_rgba();
        let (r1, g1, b1, a1) = target.as_rgba();

        #[inline]
        fn lerp(v0: u32, v1: u32, amount: f64) -> u8 {
            let v = (v0 as f64 + (v1 as f64 - v0 as f64) * amount).round() as i32;
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
    fn breakdown(&self, other: &Self, max_subdivision: usize) -> Vec<Self> {
        if max_subdivision > 0 {
            let mut pp = self.clone();

            // FIXME use hsv
            pp.step(other, 0.5);

            vec![pp]
        } else {
            Vec::new()
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
    fn breakdown(&self, other: &Self, max_subdivision: usize) -> Vec<Self> {
        let brush = self.get_brush().breakdown(other.get_brush(), max_subdivision);
        let count = brush.len();

        if count > 0 {
            let w0 = self.get_width();
            let w1 = other.get_width();
            let n = (count + 1) as f64;

            brush
                .into_iter()
                .enumerate()
                .map(|(i, b)| {
                    Stroke::new().with_brush(b).with_width(w0 + (w1 - w0) * ((i + 1) as f64 / n))
                })
                .collect()
        } else {
            Vec::new()
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
    fn breakdown(&self, other: &Self, max_subdivision: usize) -> Vec<Self> {
        match self {
            Fill::Color(c0) => match other {
                Fill::Color(c1) => {
                    c0.breakdown(c1, max_subdivision).into_iter().map(Fill::Color).collect()
                }
                _ => Vec::new(), // FIXME
            },
            _ => Vec::new(), // FIXME
        }
    }
}
