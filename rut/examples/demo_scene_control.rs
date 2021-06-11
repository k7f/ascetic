use tracing::error;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::{ControlFlow, EventLoop},
    platform::unix::EventLoopExtUnix,
    window::WindowBuilder,
};
use ascetic_vis::{Scene, Theme};
use ascetic_rut::{Gui, Logger};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win_width: u32 = 800;
    let mut win_height: u32 = 450;
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
                        "-w" => win_width = arg.parse()?,
                        "-h" => win_height = arg.parse()?,
                        _ => panic!("ERROR: Invalid CLI argument \"{}\"", arg),
                    }
                }
            }
        }
    }

    let mut logger = Logger::new("error")?;
    logger.set_crate_filter(
        match verbosity {
            0 => "warn",
            1 => "info",
            _ => "debug",
        },
    )?;
    logger.install()?;

    let event_loop: EventLoop<()> = EventLoop::new_x11().unwrap();
    let window_builder = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(win_width, win_height))
        .with_position(PhysicalPosition::new(100, 100))
        .with_title("demo scene")
        .with_resizable(true)
        .with_decorations(true)
        .with_transparent(true);
    let mut gui = Gui::new(&event_loop, &window_builder)?;
    let mut theme = Theme::simple_demo();
    let mut scene = Scene::simple_demo(&theme);

    event_loop.run(move |event, _, control_flow| match gui.update(event, &mut scene, &mut theme) {
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
    });
}
