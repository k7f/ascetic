use std::error::Error;
use aces::{Context, ContentOrigin, CEStructure, AcesError};
use super::{App, Command};

pub struct Validate {
    glob_path:    String,
    do_abort:     bool,
    syntax_only:  bool,
    is_recursive: bool,
    verbosity:    u64,
}

impl Validate {
    pub(crate) fn new(app: &App) -> Self {
        let glob_path = app.value_of("GLOB_PATH").unwrap_or_else(|| unreachable!()).to_owned();
        let do_abort = app.is_present("abort");
        let syntax_only = app.is_present("syntax");
        let is_recursive = app.is_present("recursive");
        let verbosity = app.occurrences_of("verbose").max(app.occurrences_of("log"));

        Self { glob_path, do_abort, syntax_only, is_recursive, verbosity }
    }

    pub fn new_command(app: &App) -> Box<dyn Command> {
        Box::new(Self::new(app))
    }
}

impl Command for Validate {
    fn name_of_log_file(&self) -> String {
        "ascesis-validation.log".to_owned()
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // FIXME
        let ref glob_path = format!("{}/*.cex", self.glob_path);

        let mut num_bad_files = 0;

        let ctx = Context::new_toplevel("validate", ContentOrigin::cex_stream());

        if self.is_recursive {
            // FIXME
        }

        match glob::glob(glob_path) {
            Ok(path_list) => {
                for entry in path_list {
                    match entry {
                        Ok(ref path) => {
                            if self.verbosity >= 1 {
                                info!("> {}", path.display());
                            }

                            ctx.lock().unwrap().reset(ContentOrigin::cex_script(path));

                            let result = CEStructure::from_file(&ctx, path);
                            match result {
                                Ok(ces) => {
                                    if self.verbosity >= 2 {
                                        debug!("{:?}", ces);
                                    }

                                    if !self.syntax_only && !ces.is_coherent() {
                                        let err = AcesError::IncoherentStructure(
                                            ces.get_name().unwrap_or("anonymous").to_owned(),
                                        );

                                        if self.do_abort {
                                            warn!("Aborting on structural error");
                                            return Err(err.into())
                                        } else {
                                            error!(
                                                "Structural error in file '{}'...\n\t{}",
                                                path.display(),
                                                err
                                            );
                                            num_bad_files += 1;
                                        }
                                    }
                                }
                                Err(err) => {
                                    if self.do_abort {
                                        warn!("Aborting on syntax error");
                                        return Err(err)
                                    } else {
                                        error!(
                                            "Syntax error in file '{}'...\n\t{}",
                                            path.display(),
                                            err
                                        );
                                        num_bad_files += 1;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error!("Bad entry in path list: {}", err);
                        }
                    }
                }

                if num_bad_files > 0 {
                    println!(
                        "... Done ({} bad file{}).",
                        num_bad_files,
                        if num_bad_files == 1 { "" } else { "s" },
                    );
                } else {
                    println!("... Done (no bad files).");
                }

                if self.verbosity >= 3 {
                    if self.verbosity >= 4 {
                        trace!("{:?}", ctx.lock().unwrap());
                    } else {
                        // FIXME visibility of `get_nodes()`
                        // trace!("{:?}", ctx.lock().unwrap().get_nodes());
                    }
                }

                Ok(())
            }
            Err(err) => panic!("Invalid glob pattern: {}", err),
        }
    }
}
