use std::error::Error;
use ascesis::{Contextual, Runner, Multiplicity};
use super::{App, Command, Solve};

pub struct Go {
    solve:    Solve,
    triggers: Vec<(String, Multiplicity)>,
}

impl Go {
    pub(crate) fn new(app: &mut App) -> Self {
        let solve = Solve::new(app);
        let mut triggers = Vec::new();

        if let Some(values) = app.values_of("TRIGGER") {
            triggers.extend(values.map(|name| (name.to_owned(), Multiplicity::one())));
        }

        app.apply_props(solve.get_context());
        app.accept_selectors(&["SEMANTICS", "MAX_STEPS"]);

        Self { solve, triggers }
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
        self.solve.name_of_log_file()
    }

    fn console_level(&self) -> Option<log::LevelFilter> {
        self.solve.console_level()
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.solve.run()?;

        let ces = self.solve.get_ces();

        if let Some(fset) = ces.get_firing_set() {
            let mut runner = Runner::new(
                ces.get_context(),
                self.triggers.iter().map(|(name, mul)| (name, *mul)),
            );

            runner.go(fset)?;

            let fcs = runner.get_firing_sequence();
            let num_steps = fcs.len();
            let mut state = runner.get_initial_state().clone();
            let ctx = ces.get_context();

            for (i, fc) in fcs.iter(fset).enumerate() {
                if i == 0 {
                    println!("Go from {}", state.with(&ctx));
                } else {
                    println!("   =>   {}", state.with(&ctx));
                }

                println!("{:4}. {}", i + 1, fc.with(ctx));

                fc.fire(&mut state)?;
            }

            if num_steps < runner.get_max_steps() {
                println!("   =>   {}", state.with(&ctx));
                println!("Deadlock after {} steps.", num_steps);
            } else {
                println!("Stop at {}", state.with(&ctx));
                println!("Done after {} steps.", num_steps);
            }
        } else {
            println!("Structural deadlock.");
        }

        Ok(())
    }
}
