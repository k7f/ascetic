use std::path::PathBuf;
use tracing::{error, warn, trace};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::{ControlFlow, EventLoop},
    platform::unix::EventLoopExtUnix,
    window::WindowBuilder,
};
use ascetic_vis::{Scene, Theme};
use ascetic_rut::{Gui, Logger};

#[derive(Debug)]
struct App {
    script_path:    PathBuf,
    window_builder: WindowBuilder,
    #[allow(dead_code)]
    verbosity:      u32,
}

impl App {
    const DEFAULT_SCRIPT_PATH: &'static str = "test.ces";

    fn default_window_builder() -> WindowBuilder {
        WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(800, 640))
            .with_position(PhysicalPosition::new(100, 100))
            .with_title("ascetic playground")
            .with_resizable(true)
            .with_decorations(true)
            .with_transparent(true)
    }

    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut script_path = None;
        let mut win_width: Option<u32> = None;
        let mut win_height: Option<u32> = None;
        let mut verbosity = 0;

        for (prev_arg, next_arg) in std::env::args().zip(std::env::args().skip(1)) {
            match next_arg.as_str() {
                "-v" => verbosity += 1,
                "-vv" => verbosity += 2,
                "-vvv" => verbosity += 3,
                "-w" | "-h" => {}
                arg => {
                    if arg.starts_with('-') {
                        panic!("ERROR: Invalid CLI option \"{}\"", arg)
                    } else {
                        match prev_arg.as_str() {
                            "-w" => win_width = Some(next_arg.parse()?),
                            "-h" => win_height = Some(next_arg.parse()?),
                            _ => script_path = Some(PathBuf::from(arg)),
                        }
                    }
                }
            }
        }

        let _ = Logger::create(
            match verbosity {
                0 => "warn",
                1 => "info",
                2 => "debug",
                _ => "trace",
            },
            "error",
        )?;

        let tracing_span = tracing::info_span!("main");
        let _tracing_guard = tracing_span.enter();

        let script_path = script_path.unwrap_or_else(|| {
            let mut path = PathBuf::from(".");
            path.push(Self::DEFAULT_SCRIPT_PATH);
            if verbosity > 0 {
                warn!("Unspecified input script path; using \"{}\".", path.display());
            }
            path
        });

        let mut window_builder = Self::default_window_builder();

        if let Some(width) = win_width {
            if let Some(height) = win_height
                .or_else(|| window_builder.window.inner_size.map(|s| s.to_physical(1.0).height))
            {
                window_builder = window_builder.with_inner_size(PhysicalSize::new(width, height));
            } else {
                // FIXME error
            }
        } else if let Some(height) = win_height {
            if let Some(width) = window_builder.window.inner_size.map(|s| s.to_physical(1.0).width)
            {
                window_builder = window_builder.with_inner_size(PhysicalSize::new(width, height));
            } else {
                // FIXME error
            }
        }

        Ok(App { script_path, window_builder, verbosity })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()?;

    let mut theme = Theme::simple_demo();
    let mut scene = Scene::simple_demo(&theme);

    let event_loop: EventLoop<()> = EventLoop::new_x11().unwrap();
    let mut gui = Gui::new(&event_loop, &app.window_builder)?;

    event_loop.run(move |event, _, control_flow| {
        trace!("EVENT {:?}", event);
        match gui.update(event, &mut scene, &mut theme) {
            Ok(done) => *control_flow = if done { ControlFlow::Exit } else { ControlFlow::Wait },
            Err(err) => {
                use ascetic_rut::Error::*;
                match err {
                    Fatal(err) => {
                        error!("{}", err);
                        *control_flow = ControlFlow::Exit;
                        return
                    }
                    PietFailure(err) => error!("{}", err),
                    WinitFailure(err) => error!("{}", err),
                    SwapChainFailure(err) => error!("{}", err),
                    PixelsFailure(err) => error!("{}", err),
                    err => error!("{}", err),
                }
                *control_flow = ControlFlow::Wait;
            }
        }
    });
}
