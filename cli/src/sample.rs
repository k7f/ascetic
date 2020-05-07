use std::{
    collections::{HashMap, BTreeMap},
    iter::FromIterator,
    error::Error,
};
use ascesis::{ContextHandle, CEStructure, StopCondition};
use super::{App, Command, Go, Styled};

pub struct Sample {
    go_command: Go,
    num_passes: usize,
}

impl Sample {
    pub(crate) fn new(app: &mut App) -> Self {
        let go_command = Go::new(app);

        app.apply_props(go_command.get_context());
        app.accept_selectors(&["NUM_PASSES"]);

        if let Some(num_passes) = {
            let ctx = go_command.get_context().lock().unwrap();
            ctx.get_num_passes()
        } {
            Self { go_command, num_passes }
        } else {
            unreachable!("NUM_PASSES registered but unavailable")
        }
    }

    /// Creates a [`Sample`] instance and returns it as a [`Command`]
    /// trait object.
    ///
    /// The [`App`] argument is modified, because [`Sample`] is a
    /// top-level [`Command`] which accepts a set of CLI selectors
    /// (see [`App::accept_selectors()`]) and specifies an application
    /// mode.
    pub fn new_command(app: &mut App) -> Box<dyn Command> {
        app.set_mode("Sample");

        Box::new(Self::new(app))
    }

    #[inline]
    pub fn get_context(&self) -> &ContextHandle {
        self.go_command.get_context()
    }

    #[inline]
    pub fn get_ces(&self) -> &CEStructure {
        &self.go_command.get_ces()
    }

    #[inline]
    pub fn plain_printout(&self) -> bool {
        self.go_command.plain_printout()
    }
}

impl Command for Sample {
    #[inline]
    fn name_of_log_file(&self) -> String {
        self.go_command.name_of_log_file()
    }

    #[inline]
    fn console_level(&self) -> Option<log::LevelFilter> {
        self.go_command.console_level()
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some((mut runner, fset)) = self.go_command.init()? {
            let pp = self.plain_printout();
            let mut goal_sample = HashMap::new();
            let mut pause_sample = HashMap::new();
            let mut stalemate_sample = HashMap::new();

            for pass in 0..self.num_passes {
                let stop_condition = if pass > 0 {
                    runner.restart();
                    runner.resume(&fset)?
                } else {
                    runner.go(&fset)?
                };

                let fcs = runner.get_firing_sequence();
                let mut state = runner.get_initial_state().clone();

                for fc in fcs.iter(&fset) {
                    fc.fire(&mut state)?;
                }

                match stop_condition {
                    StopCondition::GoalReached(_, num_steps) => {
                        *goal_sample.entry(num_steps).or_insert(0) += 1;
                    }
                    StopCondition::Pause(num_steps) => {
                        *pause_sample.entry(num_steps).or_insert(0) += 1;
                    }
                    StopCondition::Stalemate(num_steps) => {
                        *stalemate_sample.entry(num_steps).or_insert(0) += 1;
                    }
                    StopCondition::UnimplementedFeature(feature) => {
                        println!(
                            "{}: {} isn't implemented yet.",
                            "Failed".bright_red().bold().plain(pp),
                            feature
                        );
                        return Ok(())
                    }
                }
            }

            if goal_sample.is_empty() {
                println!("Goal never reached.");
            } else {
                let num_goals: usize = goal_sample.values().sum();
                let fnum = num_goals as f32;
                let goal_sample = BTreeMap::from_iter(
                    goal_sample.into_iter().map(|(k, v)| (k, ((v as f32 / fnum) * 100.) as u8)),
                );

                println!(
                    "{} {} times.  The sample, as (<#steps>: <freq in %>,)*,",
                    "Goal reached".bright_cyan().bold().plain(pp),
                    num_goals
                );
                println!("{:#?}.", goal_sample);
            }

            if pause_sample.is_empty() {
                println!("Never paused.");
            } else {
                let num_pauses = pause_sample.remove(&runner.get_max_steps());

                if pause_sample.is_empty() {
                    println!(
                        "{} {} times.",
                        "Still running".bright_green().bold().plain(pp),
                        num_pauses.unwrap()
                    );
                } else {
                    warn!("Pause anomaly: {:?}", pause_sample);
                }
            }

            if stalemate_sample.is_empty() {
                println!("Never stuck.");
            } else {
                let num_stales: usize = stalemate_sample.values().sum();
                let fnum = num_stales as f32;
                let stalemate_sample = BTreeMap::from_iter(
                    stalemate_sample
                        .into_iter()
                        .map(|(k, v)| (k, ((v as f32 / fnum) * 100.) as u8)),
                );

                println!(
                    "{} {} times.  The sample, as (<#steps>: <freq in %>,)*,",
                    "Stalemate reached".bright_red().bold().plain(pp),
                    num_stales
                );
                println!("{:#?}.", stalemate_sample);
            }
        }

        Ok(())
    }
}
