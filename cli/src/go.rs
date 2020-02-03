use std::error::Error;
use ascesis::{Contextual, Runner, Multiplicity};
use super::{App, Command, Solve, Styled};

pub struct Go {
    solve_command:  Solve,
    start_triggers: Vec<(String, Multiplicity)>,
    stop_triggers:  Vec<(String, Multiplicity)>,
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
}

impl Command for Go {
    fn name_of_log_file(&self) -> String {
        self.solve_command.name_of_log_file()
    }

    fn console_level(&self) -> Option<log::LevelFilter> {
        self.solve_command.console_level()
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.solve_command.run()?;

        let ces = self.solve_command.get_ces();

        if let Some(fset) = ces.get_firing_set() {
            let mut runner = Runner::new(
                ces.get_context(),
                self.start_triggers.iter().map(|(name, mul)| (name, *mul)),
            )
            .with_goal(self.stop_triggers.iter().map(|(name, mul)| (name, *mul)))?;

            println!("{}", "Go from".bright_green().bold());
            println!("{} {}", "=>".bright_yellow().bold(), runner.get_initial_state());

            runner.go(fset)?;

            let fcs = runner.get_firing_sequence();
            let mut state = runner.get_initial_state().clone();

            for (i, fc) in fcs.iter(fset).enumerate() {
                if i > 0 {
                    println!("{} {}", "=>".bright_yellow().bold(), state);
                }
                println!(
                    "{}. {}",
                    format!("{:4}", (i + 1)).bright_yellow().bold(),
                    fc.with(ces.get_context())
                );

                fc.fire(&mut state)?;
            }

            let num_steps = fcs.len();

            if runner.goal_is_reached().is_some() {
                print!("{}", "Goal!".bright_cyan().bold());
            } else if num_steps < runner.get_max_steps() {
                print!("{}", "Stuck".bright_red().bold());
            } else if num_steps == runner.get_max_steps() {
                print!("{}", "Pause".bright_green().bold());
            } else {
                unreachable!()
            }

            println!(" after {} steps at", num_steps);
            println!("{} {}", "=>".bright_yellow().bold(), state);
        } else {
            println!("{}.", "Structural deadlock".bright_red().bold());
        }

        Ok(())
    }
}
