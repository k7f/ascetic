use std::{rc::Rc, path::PathBuf, error::Error};
use ascesis::{Context, CEStructure, AscesisFormat, YamlFormat};
use super::{App, Command, AppError};

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
        let mut glob_path = PathBuf::from(&self.glob_path);

        if self.is_recursive {
            glob_path.push("**");
        }
        glob_path.push("*.ce[sx]");

        let ref glob_pattern = glob_path.to_string_lossy();
        let mut glob_options = glob::MatchOptions::new();
        glob_options.case_sensitive = false;

        let mut num_all_files = 0;
        let mut num_bad_files = 0;

        let ctx = Context::new_toplevel("validate");

        match glob::glob_with(glob_pattern, glob_options) {
            Ok(path_list) => {
                for entry in path_list {
                    match entry {
                        Ok(ref path) => {
                            if self.verbosity >= 1 {
                                info!("> {}", path.display());
                            }

                            ctx.lock().unwrap().reset();

                            let result = CEStructure::from_file(
                                &ctx,
                                path,
                                &[
                                    Rc::new(YamlFormat::from_path(path)),
                                    Rc::new(AscesisFormat::from_path(path)),
                                ],
                            );

                            num_all_files += 1;

                            match result {
                                Ok(ces) => {
                                    if self.verbosity >= 2 {
                                        debug!("{:?}", ces);
                                    }

                                    if !self.syntax_only {
                                        if let Err(err) = ces.check_coherence() {
                                            if self.do_abort {
                                                warn!("Aborting on structural error");
                                                return Err(err.into())
                                            } else {
                                                let ref header = format!(
                                                    "Structural error in file '{}'...",
                                                    path.display()
                                                );
                                                AppError::report_with_header(err.into(), header);
                                                num_bad_files += 1;
                                            }
                                        }
                                    }
                                }
                                Err(err) => {
                                    if self.do_abort {
                                        warn!("Aborting on syntax error");
                                        return Err(err)
                                    } else {
                                        let ref header =
                                            format!("Syntax error in file '{}'...", path.display());
                                        AppError::report_with_header(err, header);
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
                        "... Done ({} bad file{} out of {} checked).",
                        num_bad_files,
                        if num_bad_files == 1 { "" } else { "s" },
                        num_all_files,
                    );
                } else {
                    println!("... Done (no bad files out of {} checked).", num_all_files);
                }

                if self.verbosity >= 3 {
                    if self.verbosity >= 4 {
                        trace!("{:?}", ctx.lock().unwrap());
                    } else {
                        // FIXME replace `get_nodes()` with `Node`s iterator
                        // trace!("{:?}", ctx.lock().unwrap().get_nodes());
                    }
                }

                Ok(())
            }
            Err(err) => panic!("Invalid glob pattern: {}", err),
        }
    }
}
