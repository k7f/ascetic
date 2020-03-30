use ascetic_vis::{TranslateScale, Vec2};

#[derive(Default, Debug)]
struct Level {
    left:  Vec2,
    right: Vec2,
    up:    Vec2,
    down:  Vec2,
}

impl Level {
    fn new(amount: f64) -> Self {
        let left = (-amount, 0.).into();
        let right = (amount, 0.).into();
        let up = (0., -amount).into();
        let down = (0., amount).into();

        Level { left, right, up, down }
    }
}

#[derive(Default, Debug)]
pub struct Pan {
    levels:    Vec<Level>,
    transform: Option<TranslateScale>,
}

impl Pan {
    pub fn new() -> Self {
        let levels = vec![Level::new(8.), Level::new(16.), Level::new(32.)];

        Pan { levels, transform: None }
    }

    #[inline]
    pub fn as_transform(&self) -> TranslateScale {
        self.transform.unwrap_or_default()
    }

    #[inline]
    pub fn reset(&mut self) {
        self.transform = None;
    }

    pub fn step_left(&mut self, level: usize) {
        let amount = self.levels.get(level).or_else(|| self.levels.last()).unwrap().left;

        self.translate(amount);
    }

    pub fn step_right(&mut self, level: usize) {
        let amount = self.levels.get(level).or_else(|| self.levels.last()).unwrap().right;

        self.translate(amount);
    }

    pub fn step_up(&mut self, level: usize) {
        let amount = self.levels.get(level).or_else(|| self.levels.last()).unwrap().up;

        self.translate(amount);
    }

    pub fn step_down(&mut self, level: usize) {
        let amount = self.levels.get(level).or_else(|| self.levels.last()).unwrap().down;

        self.translate(amount);
    }

    fn translate(&mut self, amount: Vec2) {
        if let Some(ref mut t) = self.transform {
            *t += amount;
        } else {
            self.transform = Some(TranslateScale::translate(amount));
        }
    }

    /// The actual amount of translation applied after thresholding
    /// and quantizing may be less than requested (but never more).
    ///
    /// Returns the remaining amount of translation (the difference)
    /// or `None` if translation hasn't been applied.
    pub fn move_xy(&mut self, dx: f64, dy: f64, threshold_level: usize) -> Option<(f64, f64)> {
        let mut dx_remaining = dx;
        let mut dy_remaining = dy;
        let mut x_amount = None;
        let mut y_amount = None;
        let level = self.levels.get(threshold_level).or_else(|| self.levels.last()).unwrap();

        let mut count = 0;

        while dx_remaining >= level.right.x {
            dx_remaining -= level.right.x;
            count += 1;
        }

        if count > 0 {
            x_amount = Some(level.right * count as f64);
        } else {
            count = 0;

            while dx_remaining <= level.left.x {
                dx_remaining -= level.left.x;
                count += 1;
            }

            if count > 0 {
                x_amount = Some(level.left * count as f64);
            }
        }

        count = 0;

        while dy_remaining >= level.down.y {
            dy_remaining -= level.down.y;
            count += 1;
        }

        if count > 0 {
            y_amount = Some(level.down * count as f64);
        } else {
            count = 0;

            while dy_remaining <= level.up.y {
                dy_remaining -= level.up.y;
                count += 1;
            }

            if count > 0 {
                y_amount = Some(level.up * count as f64);
            }
        }

        if let Some(x_amount) = x_amount {
            if let Some(y_amount) = y_amount {
                self.translate(x_amount + y_amount);
            } else {
                self.translate(x_amount);
            }
        } else if let Some(y_amount) = y_amount {
            self.translate(y_amount);
        } else {
            return None
        }

        Some((dx - dx_remaining, dy - dy_remaining))
    }
}
