use winit::{
    event::{MouseButton, ElementState, MouseScrollDelta, TouchPhase},
    dpi::PhysicalPosition,
};

#[derive(Default, Debug)]
pub struct Mouse {
    position:     Option<PhysicalPosition<f64>>,
    inside:       bool,
    wheel_delta:  Option<MouseScrollDelta>,
    wheel_phase:  Option<TouchPhase>,
    last_left:    Option<(f32, f32)>,
    last_right:   Option<(f32, f32)>,
    last_scroll:  Option<(f32, f32)>,
    delta_left:   Option<(f32, f32)>,
    delta_right:  Option<(f32, f32)>,
    total_left:   (f32, f32),
    total_right:  (f32, f32),
    total_scroll: (f32, f32),
}

impl Mouse {
    pub fn new() -> Self {
        Mouse::default()
    }

    #[inline]
    pub fn get_left_state(&self) -> Option<(f32, f32)> {
        self.last_left
    }

    #[inline]
    pub fn get_left_drag(&self) -> (f32, f32) {
        self.total_left
    }

    #[inline]
    pub fn clear_left_drag(&mut self) {
        self.total_left = (0., 0.);
    }

    #[inline]
    pub fn set_left_drag(&mut self, dx: f32, dy: f32) {
        self.total_left = (dx, dy);
    }

    #[inline]
    pub fn get_right_state(&self) -> Option<(f32, f32)> {
        self.last_right
    }

    #[inline]
    pub fn get_right_drag(&self) -> (f32, f32) {
        self.total_right
    }

    #[inline]
    pub fn clear_right_drag(&mut self) {
        self.total_right = (0., 0.);
    }

    #[inline]
    pub fn set_right_drag(&mut self, dx: f32, dy: f32) {
        self.total_right = (dx, dy);
    }

    #[inline]
    pub fn get_scroll(&self) -> (f32, f32) {
        self.total_scroll
    }

    #[inline]
    pub fn clear_scroll(&mut self) {
        self.total_scroll = (0., 0.);
    }

    #[inline]
    pub fn set_scroll(&mut self, dx: f32, dy: f32) {
        self.total_scroll = (dx, dy);
    }

    pub fn submit_button(&mut self, button: &MouseButton, state: &ElementState) {
        match button {
            MouseButton::Left => {
                match state {
                    ElementState::Pressed => {
                        // FIXME
                    }
                    ElementState::Released => {
                        // FIXME
                    }
                }
            }
            MouseButton::Right => {
                match state {
                    ElementState::Pressed => {
                        // FIXME
                    }
                    ElementState::Released => {
                        // FIXME
                    }
                }
            }
            _ => {}
        }
    }

    pub fn submit_position(&mut self, position: &PhysicalPosition<f64>) {
        self.position = Some(*position);
    }

    pub fn submit_inside(&mut self, inside: bool) {
        self.inside = inside;
    }

    pub fn submit_wheel(&mut self, delta: &MouseScrollDelta, phase: &TouchPhase) {
        self.wheel_delta = Some(*delta);
        self.wheel_phase = Some(*phase);
    }
    // if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
    //     if window.get_mouse_down(MouseButton::Left) {
    //         state_change = true;

    //         if let Some((x0, y0)) = self.last_left {
    //             let dx = x - x0;
    //             let dy = y - y0;

    //             self.total_left.0 += dx;
    //             self.total_left.1 += dy;
    //             self.delta_left = Some((dx, dy));
    //         } else {
    //             self.delta_left = None;
    //         }

    //         self.last_left = Some((x, y));
    //     } else {
    //         self.last_left = None;
    //         self.delta_left = None;

    //         if window.get_mouse_down(MouseButton::Right) {
    //             state_change = true;

    //             if let Some((x0, y0)) = self.last_right {
    //                 let dx = x - x0;
    //                 let dy = y - y0;

    //                 self.total_right.0 += dx;
    //                 self.total_right.1 += dy;
    //                 self.delta_right = Some((dx, dy));
    //             } else {
    //                 self.delta_right = None;
    //             }

    //             self.last_right = Some((x, y));
    //         } else {
    //             self.last_right = None;
    //             self.delta_right = None;
    //         }
    //     }
    // } else {
    //     self.delta_left = None;
    //     self.delta_right = None;
    // }

    // if let Some((dx, dy)) = window.get_scroll_wheel() {
    //     state_change = true;
    //     self.total_scroll.0 += dx;
    //     self.total_scroll.1 += dy;
    //     self.last_scroll = Some((dx, dy));
    // } else {
    //     self.last_scroll = None;
    // }
}
