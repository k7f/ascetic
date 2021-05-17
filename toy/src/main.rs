#![allow(clippy::toplevel_ref_arg)]

use ascesis::Logger;
use ascetic_toy::{App, Solve, Go, Sample, Validate, AppError};

fn main() {
    let ref cli_spec_str = include_str!("ascetic.cli");

    let cli_spec = match clap::YamlLoader::load_from_str(cli_spec_str) {
        Ok(spec) => spec,
        Err(err) => {
            let mut logger = Logger::new("ascetic_toy").with_console(log::LevelFilter::Debug);
            logger.apply();

            AppError::report("Internal error in CLI specification..".into());
            AppError::report(err.into());

            std::process::exit(-1)
        }
    };
    let cli_matches = clap::App::from_yaml(&cli_spec[0]);
    let mut app = App::from_clap(cli_matches);

    let mut command = match app.subcommand_name().unwrap_or("_") {
        "_" => {
            if app.is_present("START") {
                if app.is_present("NUM_PASSES") {
                    Sample::new_command(&mut app)
                } else {
                    Go::new_command(&mut app)
                }
            } else {
                Solve::new_command(&mut app)
            }
        }
        "validate" => Validate::new_command(&app),
        unreachable => unreachable!("command \"{}\"", unreachable),
    };

    let console_level = command.console_level().unwrap_or(match app.occurrences_of("verbose") {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    });

    let mut logger = Logger::new(app.get_name()).with_console(console_level);

    if let Some(dirname) = app.value_of("LOG_DIR") {
        logger = logger.with_explicit_directory(dirname);
    }

    if app.is_present("log") || logger.get_directory().is_some() {
        let mut file_level = match app.occurrences_of("log") {
            0 | 1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        };
        if file_level < console_level {
            file_level = console_level;
        }

        logger = logger.with_file(command.name_of_log_file(), file_level);
    }

    logger.apply();

    app.post_warnings();
    app.check_selectors(&["SAT_ENCODING", "SAT_SEARCH", "SEMANTICS", "MAX_STEPS", "NUM_PASSES"]);

    if let Err(err) = command.run() {
        AppError::report(err);

        std::process::exit(-1)
    } else {
        std::process::exit(0)
    }
}
