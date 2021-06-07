use tracing::{Metadata, Id, span, event::Event};

pub struct Logger {
    crate_directive: String,
    level_filter:    tracing_subscriber::filter::LevelFilter,
    log_filter:      tracing_log::log::LevelFilter,
}

impl Logger {
    pub fn create(
        crate_filter: &str,
        global_filter: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let crate_directive = format!("ascetic_rut={}", crate_filter);
        let level_filter: tracing_subscriber::filter::LevelFilter = global_filter.parse()?;
        let env_filter = tracing_subscriber::filter::EnvFilter::from_default_env()
            .add_directive(level_filter.into())
            .add_directive(crate_directive.parse()?);

        let subscriber =
            tracing_subscriber::fmt::Subscriber::builder().with_env_filter(env_filter).finish();

        if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
            panic!("ERROR: {}", err)
        }

        let log_filter: tracing_log::log::LevelFilter = global_filter.parse()?;
        if let Err(err) = tracing_log::LogTracer::init_with_filter(log_filter) {
            panic!("ERROR: {}", err)
        }

        Ok(Logger { crate_directive, level_filter, log_filter })
    }
}
