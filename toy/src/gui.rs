use std::{iter::FromIterator, time::Duration, error::Error};
use minifb::{Key, Scale, Window, WindowOptions, ScaleMode};
use ascetic_vis::{Scene, Theme};
use crate::{Action, Scheduler, Renderer, Pixels, Pan, Zoom, Keyboard, Mouse, ToyError};

#[derive(Debug)]
pub struct Gui {
    window:     Window,
    win_width:  usize,
    win_height: usize,
    scheduler:  Scheduler,
    keyboard:   Keyboard,
    mouse:      Mouse,
    pan:        Pan,
    zoom:       Zoom,
    is_dark:    bool,
    renderer:   Renderer,
    pixels:     Pixels,
}

impl Gui {
    const DEFAULT_RENDER_SIZE: (f64, f64) = (1000., 1000.);
    const DEFAULT_RENDER_MARGIN: (f64, f64) = (10., 10.);
    const DEFAULT_UPDATE_WINDOW_PERIOD: Duration = Duration::from_millis(20);

    pub fn new(win_width: usize, win_height: usize) -> Result<Self, Box<dyn Error>> {
        let window_options = WindowOptions {
            borderless: false,
            resize: true,
            scale: Scale::X1,
            scale_mode: ScaleMode::UpperLeft,
            ..WindowOptions::default()
        };
        let mut window = Window::new("ascetic toy", win_width, win_height, window_options)?;

        window.set_background_color(0x80, 0x80, 0x80);
        window.limit_update_rate(None);

        let scheduler =
            Scheduler::from_iter([Action::UpdateWindow, Action::RenderScene].iter().copied())
                .with_debouncers(
                    [(Action::UpdateWindow, Self::DEFAULT_UPDATE_WINDOW_PERIOD)].iter().copied(),
                );

        let keyboard = Keyboard::new();
        window.set_input_callback(Box::new(keyboard.clone()));
        window.set_key_repeat_delay(0.5);
        window.set_key_repeat_rate(0.1);

        let mouse = Mouse::new();

        let pan = Pan::new();
        let zoom = Zoom::new()
            .with_ins(vec![1.5, 2., 2.75, 3.75, 4.5, 6.])
            .with_outs(vec![0.75, 0.5, 0.3, 0.1]);

        let is_dark = false;

        let renderer = Renderer::new(Self::DEFAULT_RENDER_SIZE, Self::DEFAULT_RENDER_MARGIN);

        let (pix_width, pix_height) = renderer.get_pix_size();
        let pixels = Pixels::new(pix_width, pix_height);

        Ok(Gui {
            window,
            win_width,
            win_height,
            scheduler,
            keyboard,
            mouse,
            pan,
            zoom,
            is_dark,
            renderer,
            pixels,
        })
    }

    #[inline]
    pub fn exit_confirmed(&self) -> bool {
        if self.scheduler.is_pending(Action::Exit) {
            // FIXME confirm
            println!("Bye.");

            true
        } else {
            false
        }
    }

    fn update_window(&mut self) -> Result<(), ToyError> {
        if self.window.is_open() {
            let (new_width, new_height) = self.window.get_size();

            if new_width != self.win_width || new_height != self.win_height {
                self.win_width = new_width;
                self.win_height = new_height;

                self.scheduler.enroll(Action::RedrawContents);
            }
        } else {
            self.scheduler.enroll(Action::Exit);
        }

        Ok(())
    }

    fn render_scene(&mut self, scene: &Scene, theme: &Theme) -> Result<(), ToyError> {
        let transform = self.zoom.as_transform();

        self.renderer.render(scene, theme, transform)?;

        self.scheduler.enroll(Action::RedrawContents);

        Ok(())
    }

    fn redraw_contents(&mut self) -> Result<(), ToyError> {
        if let Some(buffer) = self.renderer.get_buffer() {
            let transform = self.pan.as_transform() * self.zoom.as_transform();
            let (pix_width, pix_height) = self.renderer.get_pix_size();

            self.pixels.set_size(pix_width, pix_height);
            self.pixels.redraw(buffer, transform)
        } else {
            // FIXME
            Ok(())
        }
    }

    fn modify_theme(&mut self, theme: &mut Theme) -> Result<(), ToyError> {
        if self.is_dark {
            theme.use_original_variation();
            self.is_dark = false;
        } else {
            theme.use_variation(Some("dark"));
            self.is_dark = true;
        }

        self.scheduler.enroll(Action::RenderScene);

        Ok(())
    }

    fn update_keys(&mut self) -> Result<(), ToyError> {
        if self.keyboard.update(&self.window) {
            if self.keyboard.is_pressed(Key::Escape) {
                self.scheduler.enroll(Action::Exit);
            } else if self.keyboard.is_pressed(Key::LeftCtrl)
                || self.keyboard.is_pressed(Key::RightCtrl)
            {
                if self.keyboard.is_pressed(Key::Key0) {
                    self.zoom.reset();
                    self.scheduler.enroll(Action::Zoom);
                } else {
                    if self.keyboard.is_pressed(Key::Minus) {
                        self.zoom.step_out();
                        self.scheduler.enroll(Action::Zoom);
                    }
                    if self.keyboard.is_pressed(Key::Equal) {
                        self.zoom.step_in();
                        self.scheduler.enroll(Action::Zoom);
                    }
                }

                if self.keyboard.is_pressed(Key::Left) {
                    self.pan.step_left(2);
                    self.scheduler.enroll(Action::Pan);
                }
                if self.keyboard.is_pressed(Key::Right) {
                    self.pan.step_right(2);
                    self.scheduler.enroll(Action::Pan);
                }
                if self.keyboard.is_pressed(Key::Up) {
                    self.pan.step_up(2);
                    self.scheduler.enroll(Action::Pan);
                }
                if self.keyboard.is_pressed(Key::Down) {
                    self.pan.step_down(2);
                    self.scheduler.enroll(Action::Pan);
                }
            } else {
                if self.keyboard.is_pressed(Key::Home) {
                    self.pan.reset();
                    self.scheduler.enroll(Action::Pan);
                }

                if self.keyboard.is_pressed(Key::Left) {
                    self.pan.step_left(1);
                    self.scheduler.enroll(Action::Pan);
                }
                if self.keyboard.is_pressed(Key::Right) {
                    self.pan.step_right(1);
                    self.scheduler.enroll(Action::Pan);
                }
                if self.keyboard.is_pressed(Key::Up) {
                    self.pan.step_up(1);
                    self.scheduler.enroll(Action::Pan);
                }
                if self.keyboard.is_pressed(Key::Down) {
                    self.pan.step_down(1);
                    self.scheduler.enroll(Action::Pan);
                }

                if self.keyboard.is_pressed(Key::Space) {
                    self.scheduler.enroll(Action::ModifyTheme);
                }
            }
        }

        Ok(())
    }

    fn update_mouse(&mut self) -> Result<(), ToyError> {
        if self.mouse.update(&self.window) {
            let (dx, dy) = self.mouse.get_left_drag();

            if let Some((dx, dy)) = self.pan.move_xy(dx as f64, dy as f64, 0) {
                self.mouse.set_left_drag(dx as f32, dy as f32);
                self.scheduler.enroll(Action::Pan);
            }

            let (dx, dy) = self.mouse.get_scroll();

            if let Some((dx, dy)) = self.pan.move_xy(dx as f64, dy as f64, 1) {
                self.mouse.set_scroll(dx as f32, dy as f32);
                self.scheduler.enroll(Action::Pan);
            }
        }

        Ok(())
    }

    pub fn process_action(
        &mut self,
        action: Action,
        scene: &mut Scene,
        theme: &mut Theme,
    ) -> Result<(), ToyError> {
        match action {
            Action::UpdateWindow => {
                self.update_window()?;
            }
            Action::RenderScene => {
                self.render_scene(scene, theme)?;
            }
            Action::RedrawContents => {
                self.redraw_contents()?;
            }
            Action::ModifyTheme => {
                self.modify_theme(theme)?;
            }
            Action::Pan => {
                self.redraw_contents()?;
            }
            Action::Zoom => {
                self.render_scene(scene, theme)?;
            }
            Action::UpdateKeys => {
                self.update_keys()?;
            }
            Action::UpdateMouse => {
                self.update_mouse()?;
            }
            Action::Exit => {}
            Action::DoNothing => {
                self.scheduler.enroll(Action::UpdateWindow);
            }
        }

        Ok(())
    }

    pub fn update(&mut self, scene: &mut Scene, theme: &mut Theme) -> Result<(), ToyError> {
        while let Some(action) = self.scheduler.next_eager() {
            self.process_action(action, scene, theme)?;
        }

        if let Some(action) = self.scheduler.next_lazy() {
            self.process_action(action, scene, theme)?;
        }

        if let Some((pixels, width, height)) = self.pixels.apply() {
            self.window.update_with_buffer(pixels, width, height)?;
        } else {
            self.window.update();
        }

        self.scheduler.enroll(Action::UpdateWindow);
        self.scheduler.enroll(Action::UpdateKeys);
        self.scheduler.enroll(Action::UpdateMouse);

        Ok(())
    }
}
