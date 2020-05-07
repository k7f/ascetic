use std::{str::FromStr, rc::Rc, error::Error};
use ascesis::{Context, ContextHandle, Contextual, CEStructure, AscesisFormat, YamlFormat, sat};
use super::{App, Command, Styled};

pub struct Solve {
    verbosity:          u64,
    plain_printout:     bool,
    main_path:          String,
    more_paths:         Vec<String>,
    requested_encoding: Option<sat::Encoding>,
    requested_search:   Option<sat::Search>,
    ces:                CEStructure,
}

impl Solve {
    pub(crate) fn new(app: &mut App) -> Self {
        let verbosity = app.occurrences_of("verbose").max(app.occurrences_of("log"));
        let plain_printout = app.is_present("plain");

        let mut path_values = app.values_of("MAIN_PATH").unwrap_or_else(|| unreachable!());
        let main_path = path_values.next().unwrap_or_else(|| unreachable!()).to_owned();
        let more_paths: Vec<_> = path_values.map(|p| p.to_owned()).collect();

        let requested_encoding = app.value_of("SAT_ENCODING").map(|v| match v {
            "PL" | "port-link" => sat::Encoding::PortLink,
            "FJ" | "fork-join" => sat::Encoding::ForkJoin,
            _ => unreachable!(),
        });

        let requested_search = app.value_of("SAT_SEARCH").map(|v| match v {
            "min" => sat::Search::MinSolutions,
            "all" => sat::Search::AllSolutions,
            _ => unreachable!(),
        });

        let context_name = format!(
            "ascetic-{}",
            app.get_mode().expect("unexpected anonymous mode").to_lowercase()
        );
        let context = Context::new_toplevel(context_name);
        let ces = CEStructure::new_interactive(&context);

        app.accept_selectors(&["SAT_ENCODING", "SAT_SEARCH"]);

        Self {
            verbosity,
            plain_printout,
            main_path,
            more_paths,
            requested_encoding,
            requested_search,
            ces,
        }
    }

    /// Creates a [`Solve`] instance and returns it as a [`Command`]
    /// trait object.
    ///
    /// The [`App`] argument is modified, because [`Solve`] is a
    /// top-level [`Command`] which accepts a set of CLI selectors
    /// (see [`App::accept_selectors()`]) and specifies an application
    /// mode.
    pub fn new_command(app: &mut App) -> Box<dyn Command> {
        app.set_mode("Solve");

        Box::new(Self::new(app))
    }

    #[inline]
    pub fn get_context(&self) -> &ContextHandle {
        self.ces.get_context()
    }

    #[inline]
    pub fn get_ces(&self) -> &CEStructure {
        &self.ces
    }

    #[inline]
    pub fn plain_printout(&self) -> bool {
        self.plain_printout
    }
}

impl Command for Solve {
    fn name_of_log_file(&self) -> String {
        if let Ok(mut path) = std::path::PathBuf::from_str(&self.main_path) {
            if path.set_extension("log") {
                if let Some(file_name) = path.file_name() {
                    return file_name.to_str().unwrap().to_owned()
                } else {
                }
            } else {
            }
        } else {
        }

        "ascetic.log".to_owned()
    }

    fn console_level(&self) -> Option<log::LevelFilter> {
        Some(match self.verbosity {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let pp = self.plain_printout;

        if let Some(encoding) = self.requested_encoding {
            self.ces.get_context().lock().unwrap().set_encoding(encoding);
        }

        if let Some(search) = self.requested_search {
            self.ces.get_context().lock().unwrap().set_search(search);
        }

        let path = &self.main_path;

        self.ces.add_from_file_as_origin(
            path,
            &[Rc::new(YamlFormat::from_path(path)), Rc::new(AscesisFormat::from_path(path))],
        )?;

        for path in self.more_paths.iter() {
            self.ces.add_from_file(
                path,
                &[Rc::new(YamlFormat::from_path(path)), Rc::new(AscesisFormat::from_path(path))],
            )?;
        }

        trace!("{:?}", self.ces);
        // FIXME impl Display
        // info!("{}", self.ces);

        self.ces.solve()?;

        if let Some(fset) = self.ces.get_firing_set() {
            match fset.as_slice().len() {
                0 => {
                    warn!("Unsat resulted in empty FiringSet, instead of explicit deadlock");
                    println!("{}.", "Structural deadlock".bright_red().bold().plain(pp));
                }
                1 => println!("{} one firing component:", "Found".bright_green().bold().plain(pp)),
                n => {
                    println!("{} {} firing components:", "Found".bright_green().bold().plain(pp), n)
                }
            }

            let ctx = self.ces.get_context();

            for (i, fc) in fset.as_slice().iter().enumerate() {
                println!(
                    "{}. {}",
                    format!("{:4}", (i + 1)).bright_yellow().bold().plain(pp),
                    fc.with(ctx)
                );
            }
        } else {
            println!("{}.", "Structural deadlock".bright_red().bold().plain(pp));
        }

        Ok(())
    }
}
