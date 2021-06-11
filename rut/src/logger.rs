use tracing_subscriber::{
    fmt::Subscriber,
    filter::{LevelFilter, EnvFilter, Directive},
};

pub struct Logger {
    crate_directive: String,
    level_filter:    tracing_subscriber::filter::LevelFilter,
    log_filter:      tracing_log::log::LevelFilter,
}

impl Logger {
    pub fn new(global_filter: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let level_filter: LevelFilter = global_filter.parse()?;
        let log_filter: tracing_log::log::LevelFilter = global_filter.parse()?;
        let crate_directive = format!("{}={}", env!("CARGO_CRATE_NAME"), level_filter);

        Ok(Logger { crate_directive, level_filter, log_filter })
    }

    pub fn set_crate_filter(
        &mut self,
        crate_filter: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let crate_directive = format!("{}={}", env!("CARGO_CRATE_NAME"), crate_filter);
        let _: Directive = crate_directive.parse()?;

        self.crate_directive = crate_directive;

        Ok(())
    }

    pub fn install(&self) -> Result<(), Box<dyn std::error::Error>> {
        let env_filter = EnvFilter::from_default_env()
            .add_directive(self.level_filter.into())
            .add_directive(self.crate_directive.parse()?);
        let subscriber = Subscriber::builder().with_env_filter(env_filter).finish();

        tracing::subscriber::set_global_default(subscriber)?;
        tracing_log::LogTracer::init_with_filter(self.log_filter)?;

        Ok(())
    }
}
