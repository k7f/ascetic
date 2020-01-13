#![allow(clippy::toplevel_ref_arg)]

#[macro_use]
extern crate log;

mod solve;
mod go;
mod validate;

use ascesis::{ContextHandle, Semantics};

pub use solve::Solve;
pub use go::Go;
pub use validate::Validate;

pub trait Command {
    fn name_of_log_file(&self) -> String;

    fn console_level(&self) -> Option<log::LevelFilter> {
        None
    }

    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct App<'a> {
    app_name:           String,
    bin_name:           Option<String>,
    cli_args:           clap::ArgMatches<'a>,
    mode:               Option<String>,
    accepted_selectors: Vec<String>,
    delayed_warnings:   Vec<String>, // Accumulates warnings delayed until after logger's setup.
}

impl<'a> App<'a> {
    pub fn from_clap<'b>(clap_app: clap::App<'a, 'b>) -> Self {
        let cli_app = clap_app
            .name(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"));

        let app_name = cli_app.get_name().to_owned();
        let bin_name = cli_app.get_bin_name().map(|s| s.to_owned());
        let cli_args = cli_app.get_matches();
        let mode = None;
        let accepted_selectors = Vec::new();
        let delayed_warnings = Vec::new();

        Self { app_name, bin_name, cli_args, mode, accepted_selectors, delayed_warnings }
    }

    pub fn get_name(&self) -> &str {
        self.app_name.as_str()
    }

    pub fn get_bin_name(&self) -> Option<&str> {
        self.bin_name.as_ref().map(|s| s.as_str())
    }

    pub fn subcommand_name(&self) -> Option<&str> {
        self.cli_args.subcommand_name()
    }

    pub fn value_of<S: AsRef<str>>(&self, key: S) -> Option<&str> {
        self.cli_args.subcommand().1.unwrap_or(&self.cli_args).value_of(key)
    }

    pub fn values_of<S: AsRef<str>>(&self, key: S) -> Option<clap::Values> {
        self.cli_args.subcommand().1.unwrap_or(&self.cli_args).values_of(key)
    }

    pub fn occurrences_of<S: AsRef<str>>(&self, key: S) -> u64 {
        self.cli_args.subcommand().1.unwrap_or(&self.cli_args).occurrences_of(key)
    }

    pub fn is_present<S: AsRef<str>>(&self, key: S) -> bool {
        self.cli_args.subcommand().1.unwrap_or(&self.cli_args).is_present(key)
    }

    pub fn set_mode<S: AsRef<str>>(&mut self, mode: S) {
        self.mode = Some(mode.as_ref().to_owned());
    }

    pub fn get_mode(&self) -> Option<&str> {
        self.mode.as_ref().map(|mode| mode.as_str())
    }

    /// Accepts a given set of selectors, incrementally.
    ///
    /// A selector is a top-level CLI argument which is declared in a
    /// `.cli` file.  All selectors are opt-in: if an unaccepted
    /// selector occurs in a command line, it will be ignored, with a
    /// warning.
    pub fn accept_selectors(&mut self, selectors: &[&str]) {
        for &selector in selectors {
            let selector = selector.to_string();

            if let Err(pos) = self.accepted_selectors.binary_search(&selector) {
                self.accepted_selectors.insert(pos, selector);
            }
        }
    }

    pub fn check_selectors(&mut self, all_selectors: &[&str]) {
        if let Some(ref mode) = self.mode {
            for &selector in all_selectors {
                let selector = selector.to_string();

                if self.accepted_selectors.binary_search(&selector).is_err()
                    && self.is_present(&selector)
                {
                    warn!(
                        "Argument \"{}\" has no meaning in mode \"{}\" and is ignored",
                        selector, mode
                    );
                }
            }
        }
    }

    pub fn apply_props(&self, ctx: &ContextHandle) {
        let mut ctx = ctx.lock().expect("Can't acquire Context when applying Props.");

        if let Some(v) = self.value_of("SEMANTICS") {
            match v {
                "seq" => ctx.set_semantics(Semantics::Sequential),
                "par" => ctx.set_semantics(Semantics::Parallel),
                _ => unreachable!(),
            }
        }

        if let Some(v) = self.value_of("MAX_STEPS") {
            match v.parse::<usize>() {
                Ok(val) => ctx.set_max_steps(val),
                Err(err) => {
                    panic!("The argument '{}' isn't a valid value of MAX_STEPS ({})", v, err)
                }
            }
        }
    }

    pub fn post_warnings(&self) {
        for warning in self.delayed_warnings.iter() {
            warn!("{}", warning);
        }
    }
}
