use kurbo::Rect;
use crate::Font;

#[derive(Clone, Debug)]
pub struct TextLabel {
    pub(crate) x:    f64,
    pub(crate) y:    f64,
    pub(crate) body: String,
}

impl TextLabel {
    pub fn new(body: String) -> Self {
        TextLabel { x: 0.0, y: 0.0, body }
    }

    pub fn with_xy(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }
}

impl TextLabel {
    pub fn bounding_box(&self, _font: Font) -> Rect {
        // FIXME
        Rect::new(self.x, self.y, self.x + 100.0, self.y + 30.0)
    }
}
