use std::error::Error;
use ascesis::{ContextHandle, Contextual, CEStructure, Runner, StopCondition, Multiplicity, FiringSet};
use super::{App, Command, Solve, Styled};

pub struct Go {
    pub(crate) solve_command: Solve,
    start_triggers:           Vec<(String, Multiplicity)>,
    stop_triggers:            Vec<(String, Multiplicity)>,
}

impl Go {
    fn trigger_parse<S: AsRef<str>>(name: S) -> (String, Multiplicity) {
        let name = name.as_ref();

        if let Some(pos) = name.rfind(':') {
            if pos > 0 && pos < name.len() {
                if let Ok(weight) = name[pos + 1..].parse::<Multiplicity>() {
                    return (name[..pos].to_owned(), weight)
                }
            }
        }

        (name.to_owned(), Multiplicity::one())
    }

    pub(crate) fn new(app: &mut App) -> Self {
        let solve_command = Solve::new(app);
        let mut start_triggers = Vec::new();
        let mut stop_triggers = Vec::new();

        if let Some(values) = app.values_of("START") {
            start_triggers.extend(values.map(Self::trigger_parse));
        }

        if let Some(values) = app.values_of("GOAL") {
            stop_triggers.extend(values.map(Self::trigger_parse));
        }

        app.apply_props(solve_command.get_context());
        app.accept_selectors(&["SEMANTICS", "MAX_STEPS"]);

        Self { solve_command, start_triggers, stop_triggers }
    }

    /// Creates a [`Go`] instance and returns it as a [`Command`]
    /// trait object.
    ///
    /// The [`App`] argument is modified, because [`Go`] is a
    /// top-level [`Command`] which accepts a set of CLI selectors
    /// (see [`App::accept_selectors()`]) and specifies an application
    /// mode.
    pub fn new_command(app: &mut App) -> Box<dyn Command> {
        app.set_mode("Go");

        Box::new(Self::new(app))
    }

    pub(crate) fn init(&mut self) -> Result<Option<(Runner, FiringSet)>, Box<dyn Error>> {
        self.solve_command.run()?;

        if let Some(fset) = self.get_ces().get_firing_set().cloned() {
            let mut runner = Runner::new(
                self.get_ces().get_context(),
                self.start_triggers.iter().map(|(name, mul)| (name, *mul)),
            );

            if !self.stop_triggers.is_empty() {
                runner =
                    runner.with_goal(self.stop_triggers.iter().map(|(name, mul)| (name, *mul)))?;
            }

            Ok(Some((runner, fset)))
        } else {
            let pp = self.plain_printout();
            println!("{}.", "Structural deadlock".bright_red().bold().plain(pp));

            Ok(None)
        }
    }

    #[inline]
    pub fn get_context(&self) -> &ContextHandle {
        self.solve_command.get_context()
    }

    #[inline]
    pub fn get_ces(&self) -> &CEStructure {
        &self.solve_command.get_ces()
    }

    #[inline]
    pub fn plain_printout(&self) -> bool {
        self.solve_command.plain_printout()
    }
}

impl Command for Go {
    #[inline]
    fn name_of_log_file(&self) -> String {
        self.solve_command.name_of_log_file()
    }

    #[inline]
    fn console_level(&self) -> Option<log::LevelFilter> {
        self.solve_command.console_level()
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some((mut runner, fset)) = self.init()? {
            let pp = self.solve_command.plain_printout();

            println!("{}", "Go from".bright_green().bold().plain(pp));
            println!("{} {}", "=>".bright_yellow().bold().plain(pp), runner.get_initial_state());

            let stop_condition = runner.go(&fset)?;

            let ces = self.solve_command.get_ces();
            let fcs = runner.get_firing_sequence();
            let mut state = runner.get_initial_state().clone();

            for (i, fc) in fcs.iter(&fset).enumerate() {
                if i > 0 {
                    println!("{} {}", "=>".bright_yellow().bold().plain(pp), state);
                }
                println!(
                    "{}. {}",
                    format!("{:4}", (i + 1)).bright_yellow().bold().plain(pp),
                    fc.with(ces.get_context())
                );

                fc.fire(&mut state)?;
            }

            if let Some(num_steps) = match stop_condition {
                StopCondition::GoalReached(node_id, num_steps) => {
                    print!(
                        "{} reached (node \"{}\")",
                        "Goal".bright_cyan().bold().plain(pp),
                        node_id.with(ces.get_context()),
                    );
                    Some(num_steps)
                }
                StopCondition::Stalemate(num_steps) => {
                    print!("{}", "Stuck".bright_red().bold().plain(pp));
                    Some(num_steps)
                }
                StopCondition::Pause(num_steps) => {
                    print!("{}", "Paused".bright_green().bold().plain(pp));
                    Some(num_steps)
                }
                StopCondition::UnimplementedFeature(feature) => {
                    println!(
                        "{}: {} isn't implemented yet.",
                        "Failed".bright_red().bold().plain(pp),
                        feature
                    );
                    None
                }
            } {
                println!(" after {} steps at", num_steps);
                println!("{} {}", "=>".bright_yellow().bold().plain(pp), state);
            }
        }

        Ok(())
    }
}
