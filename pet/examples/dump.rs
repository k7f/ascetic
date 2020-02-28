#[macro_use]
extern crate log;

use std::{path::PathBuf, error::Error};
use ascetic_pet::PNML;

struct AppLogger(log::Level);

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.0
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static mut APP_LOGGER: AppLogger = AppLogger(log::Level::Warn);

struct App {
    path:      PathBuf,
    verbosity: u32,
}

impl App {
    fn new<P: Into<PathBuf>>(default_path: P) -> Self {
        let args = std::env::args();
        let mut path = None;
        let mut verbosity = 0;

        for arg in args.skip(1) {
            match arg.as_str() {
                "-v" => verbosity += 1,
                "-vv" => verbosity += 2,
                arg => {
                    if arg.starts_with('-') {
                        panic!("ERROR: Invalid CLI option \"{}\"", arg)
                    } else {
                        path = Some(PathBuf::from(arg));
                    }
                }
            }
        }

        let log_level = match verbosity {
            0 => log::Level::Warn,
            1 => log::Level::Info,
            _ => log::Level::Debug,
        };

        unsafe {
            APP_LOGGER.0 = log_level;

            if let Err(err) = log::set_logger(&APP_LOGGER) {
                panic!("ERROR: {}", err)
            }
        }

        log::set_max_level(log_level.to_level_filter());

        App { path: path.unwrap_or_else(|| default_path.into()), verbosity }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    const DEFAULT_PATH: &str = "data/pnml/test.pnml";

    let app = App::new(DEFAULT_PATH);
    let pnml = PNML::from_file(&app.path)?;

    for err in pnml.get_errors().iter() {
        warn!("[{}]: {}", err.text_start(), err);
    }

    if app.verbosity > 0 {
        info!("{:?}", pnml.get_nets());
    }

    Ok(())
}
