use std::time::Duration;
use tracing::{trace, debug};
use winit::{
    dpi::PhysicalSize,
    event::{Event, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder, Window},
    monitor::MonitorHandle,
};
use pixels::wgpu;
use ascetic_vis::{Scene, Theme};
use crate::{Action, Scheduler, Renderer, Raster, Frame, Pan, Zoom, Keyboard, Mouse};

#[derive(Debug)]
pub struct Gui {
    scheduler:          Scheduler,
    window:             Window,
    win_width:          u32,
    win_height:         u32,
    keyboard:           Keyboard,
    mouse:              Mouse,
    pan:                Pan,
    zoom:               Zoom,
    fullscreen_monitor: MonitorHandle,
    is_dark:            bool,
    renderer:           Renderer,
    raster:             Raster,
    frame:              Frame,
    need_resize:        bool,
    need_redraw:        bool,
}

impl Gui {
    const DEFAULT_RENDER_SIZE: (f64, f64) = (1000., 1000.);
    const DEFAULT_RENDER_MARGIN: (f64, f64) = (10., 10.);
    const DEFAULT_UPDATE_WINDOW_PERIOD: Duration = Duration::from_millis(20);

    pub fn new(
        event_loop: &EventLoop<()>,
        window_builder: &WindowBuilder,
    ) -> Result<Self, crate::Error> {
        let scheduler = [Action::UpdateWindow, Action::RenderScene]
            .iter()
            .copied()
            .collect::<Scheduler>()
            .with_debouncers(
                [(Action::UpdateWindow, Self::DEFAULT_UPDATE_WINDOW_PERIOD)].iter().copied(),
            );

        let fullscreen_monitor = event_loop.available_monitors().next().unwrap();
        let window = window_builder.clone().build(event_loop).unwrap();

        let mut keyboard = Keyboard::new();
        keyboard.set_repeat_delay(Duration::from_millis(500));
        keyboard.set_repeat_period(Duration::from_millis(100));

        let mouse = Mouse::new();

        let pan = Pan::new();
        let zoom = Zoom::new()
            .with_ins(vec![1.5, 2., 2.75, 3.75, 4.5, 6.])
            .with_outs(vec![0.75, 0.5, 0.3, 0.1]);

        let is_dark = false;

        let PhysicalSize { width: win_width, height: win_height } = window.inner_size();
        let renderer = Renderer::new(Self::DEFAULT_RENDER_SIZE, Self::DEFAULT_RENDER_MARGIN);
        let raster = Raster::new(win_width, win_height);
        let frame = Frame::new(&window)?;
        let need_resize = false;
        let need_redraw = false;

        Ok(Gui {
            window,
            win_width,
            win_height,
            scheduler,
            keyboard,
            mouse,
            pan,
            zoom,
            fullscreen_monitor,
            is_dark,
            renderer,
            raster,
            frame,
            need_resize,
            need_redraw,
        })
    }

    fn update_window(&mut self) -> Result<(), crate::Error> {
        let PhysicalSize { width, height } = self.window.inner_size();

        if width != self.win_width || height != self.win_height {
            self.win_width = width;
            self.win_height = height;
            self.need_resize = true;

            self.scheduler.enroll(Action::RedrawContents);
        }

        Ok(())
    }

    fn render_scene(&mut self, scene: &Scene, theme: &Theme) -> Result<(), crate::Error> {
        let transform = self.zoom.as_transform();

        self.renderer.render(scene, theme, transform)?;

        self.scheduler.enroll(Action::RedrawContents);

        Ok(())
    }

    fn redraw_contents(&mut self) -> Result<(), crate::Error> {
        if self.need_resize {
            self.raster.resize(self.win_width, self.win_height);
            self.frame.resize(self.win_width, self.win_height);
            self.need_resize = false;
        }

        if let Some(buffer) = self.renderer.get_buffer() {
            let transform = self.pan.as_transform() * self.zoom.as_transform();
            let (pix_width, pix_height) = self.renderer.get_pix_size();

            self.raster.redraw(buffer, pix_width, pix_height, transform)?;

            self.window.request_redraw();
        } else {
            // FIXME
        }

        Ok(())
    }

    fn change_theme_variation(&mut self, theme: &mut Theme) -> Result<(), crate::Error> {
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

    fn update_keys(&mut self) -> Result<(), crate::Error> {
        if let Some(mods) = self.keyboard.is_pressed(VirtualKeyCode::Q) {
            if mods == ModifiersState::CTRL {
                self.scheduler.enroll(Action::Exit);
            }
        } else {
            if let Some(mods) = self.keyboard.is_pressed(VirtualKeyCode::V) {
                if mods == ModifiersState::SHIFT | ModifiersState::CTRL {
                    self.scheduler.enroll(Action::ChangeThemeVariation);
                }
            }

            if self.keyboard.is_pressed(VirtualKeyCode::Escape).is_some() {
                self.scheduler.enroll(Action::FullscreenOff);
            }

            if self.keyboard.is_pressed(VirtualKeyCode::F11) == Some(ModifiersState::CTRL) {
                self.scheduler.enroll(Action::FullscreenToggle);
            }

            if self.keyboard.is_pressed(VirtualKeyCode::Key0) == Some(ModifiersState::CTRL) {
                self.zoom.reset();
                self.scheduler.enroll(Action::Zoom);
            }

            if self.keyboard.is_pressed(VirtualKeyCode::Minus) == Some(ModifiersState::CTRL) {
                self.zoom.step_out();
                self.scheduler.enroll(Action::Zoom);
            }

            if self.keyboard.is_pressed(VirtualKeyCode::Equals) == Some(ModifiersState::CTRL) {
                self.zoom.step_in();
                self.scheduler.enroll(Action::Zoom);
            }
        }

        //     if self.keyboard.is_pressed(Key::LeftCtrl)
        //         || self.keyboard.is_pressed(Key::RightCtrl)
        //     {
        //         if self.keyboard.is_pressed(Key::Left) {
        //             self.pan.step_left(2);
        //             self.scheduler.enroll(Action::Pan);
        //         }
        //         if self.keyboard.is_pressed(Key::Right) {
        //             self.pan.step_right(2);
        //             self.scheduler.enroll(Action::Pan);
        //         }
        //         if self.keyboard.is_pressed(Key::Up) {
        //             self.pan.step_up(2);
        //             self.scheduler.enroll(Action::Pan);
        //         }
        //         if self.keyboard.is_pressed(Key::Down) {
        //             self.pan.step_down(2);
        //             self.scheduler.enroll(Action::Pan);
        //         }
        //     } else {
        //         if self.keyboard.is_pressed(Key::Home) {
        //             self.pan.reset();
        //             self.scheduler.enroll(Action::Pan);
        //         }

        //         if self.keyboard.is_pressed(Key::Left) {
        //             self.pan.step_left(1);
        //             self.scheduler.enroll(Action::Pan);
        //         }
        //         if self.keyboard.is_pressed(Key::Right) {
        //             self.pan.step_right(1);
        //             self.scheduler.enroll(Action::Pan);
        //         }
        //         if self.keyboard.is_pressed(Key::Up) {
        //             self.pan.step_up(1);
        //             self.scheduler.enroll(Action::Pan);
        //         }
        //         if self.keyboard.is_pressed(Key::Down) {
        //             self.pan.step_down(1);
        //             self.scheduler.enroll(Action::Pan);
        //         }
        //     }

        Ok(())
    }

    fn update_mouse(&mut self) -> Result<(), crate::Error> {
        // if self.mouse.update(&self.window) {
        //     let (dx, dy) = self.mouse.get_left_drag();

        //     if let Some((dx, dy)) = self.pan.move_xy(dx as f64, dy as f64, 0) {
        //         self.mouse.set_left_drag(dx as f32, dy as f32);
        //         self.scheduler.enroll(Action::Pan);
        //     }

        //     let (dx, dy) = self.mouse.get_scroll();

        //     if let Some((dx, dy)) = self.pan.move_xy(dx as f64, dy as f64, 1) {
        //         self.mouse.set_scroll(dx as f32, dy as f32);
        //         self.scheduler.enroll(Action::Pan);
        //     }
        // }

        Ok(())
    }

    pub fn process_action(
        &mut self,
        action: Action,
        scene: &mut Scene,
        theme: &mut Theme,
    ) -> Result<(), crate::Error> {
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
            Action::ChangeThemeVariation => {
                self.change_theme_variation(theme)?;
            }
            Action::Pan => {
                self.redraw_contents()?;
            }
            Action::Zoom => {
                self.render_scene(scene, theme)?;
            }
            Action::FullscreenOff => {
                if self.window.fullscreen().is_some() {
                    self.window.set_fullscreen(None);
                }
            }
            Action::FullscreenToggle => {
                if self.window.fullscreen().is_some() {
                    self.window.set_fullscreen(None);
                } else {
                    self.window.set_fullscreen(Some(Fullscreen::Borderless(Some(
                        self.fullscreen_monitor.clone(),
                    ))));
                }
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

    fn exit_confirmed(&self) -> bool {
        // FIXME confirm
        println!("Bye.");

        true
    }

    pub fn update(
        &mut self,
        event: Event<()>,
        scene: &mut Scene,
        theme: &mut Theme,
    ) -> Result<bool, crate::Error> {
        if self.scheduler.is_pending(Action::Exit, true) && self.exit_confirmed() {
            return Ok(true)
        }

        while let Some(action) = self.scheduler.next_eager() {
            self.process_action(action, scene, theme)?;
        }

        match event {
            Event::LoopDestroyed => {
                debug!("Event {:?}", event);
            }
            Event::MainEventsCleared => {
                trace!("Event {:?}", event);
            }
            Event::RedrawRequested(_) => {
                trace!("Event {:?}", event);
                self.need_redraw = true;
            }
            Event::RedrawEventsCleared => {
                trace!("Event {:?} (need_redraw: {})", event, self.need_redraw);
                if self.need_redraw {
                    self.need_redraw = false;
                    self.raster.apply(self.frame.get());

                    if let Err(err) = self.frame.render() {
                        match err {
                            crate::Error::SwapChainFailure(wgpu::SwapChainError::Outdated)
                            | crate::Error::SwapChainFailure(wgpu::SwapChainError::Lost) => {
                                self.scheduler.enroll(Action::UpdateWindow);
                            }
                            _ => {}
                        }
                        return Err(err)
                    }
                }
            }
            Event::WindowEvent { ref event, window_id } if window_id == self.window.id() => {
                use WindowEvent::*;
                match event {
                    CloseRequested | Destroyed => return Ok(self.exit_confirmed()),
                    ModifiersChanged(modifiers) => {
                        self.keyboard.submit_modifiers(modifiers);
                    }
                    KeyboardInput { input, .. } => {
                        self.keyboard.submit_key(input);
                        self.scheduler.enroll(Action::UpdateKeys);
                    }
                    MouseInput { state, button, .. } => {
                        self.mouse.submit_button(button, state);
                        self.scheduler.enroll(Action::UpdateMouse);
                    }
                    CursorMoved { position, .. } => {
                        self.mouse.submit_position(position);
                    }
                    CursorEntered { .. } => {
                        self.mouse.submit_inside(true);
                    }
                    CursorLeft { .. } => {
                        self.mouse.submit_inside(false);
                    }
                    MouseWheel { delta, phase, .. } => {
                        self.mouse.submit_wheel(delta, phase);
                    }
                    Resized(_) => {
                        self.scheduler.enroll(Action::UpdateWindow);
                    }
                    ScaleFactorChanged { .. } => {
                        self.scheduler.enroll(Action::UpdateWindow);
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if let Some(action) = self.scheduler.next_lazy() {
            self.process_action(action, scene, theme)?;
        }

        Ok(false)
    }
}
