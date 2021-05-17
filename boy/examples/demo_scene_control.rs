#[macro_use]
extern crate log;

use std::error::Error;
use ascetic_vis::{Scene, Theme};
use ascetic_boy::{Gui, BoyError, BoyLogger};

#[derive(Debug)]
struct App {
    gui:       Gui,
    #[allow(dead_code)]
    verbosity: u32,
}

impl App {
    const DEFAULT_WIN_WIDTH: usize = 800;
    const DEFAULT_WIN_HEIGHT: usize = 450;

    fn new() -> Result<Self, Box<dyn Error>> {
        let mut win_width = Self::DEFAULT_WIN_WIDTH;
        let mut win_height = Self::DEFAULT_WIN_HEIGHT;
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

        BoyLogger::init(match verbosity {
            0 => log::Level::Warn,
            1 => log::Level::Info,
            _ => log::Level::Debug,
        });

        let gui = Gui::new(win_width, win_height)?;

        Ok(App { gui, verbosity })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new()?;
    let mut theme = Theme::simple_demo();
    let mut scene = Scene::simple_demo(&theme);

    loop {
        if let Err(err) = app.gui.update(&mut scene, &mut theme) {
            match err {
                BoyError::Fatal(err) => {
                    error!("{}", err);
                    return Err(err)
                }
                BoyError::PietFailure(err) => error!("{}", err),
                BoyError::MinifbFailure(err) => error!("{}", err),
            }
        } else if app.gui.exit_confirmed() {
            return Ok(())
        }
    }
}
