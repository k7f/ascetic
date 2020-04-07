static mut TOY_LOGGER: ToyLogger = ToyLogger(log::Level::Warn);

pub struct ToyLogger(log::Level);

impl ToyLogger {
    pub fn init(level: log::Level) {
        unsafe {
            TOY_LOGGER.0 = level;

            if let Err(err) = log::set_logger(&TOY_LOGGER) {
                panic!("ERROR: {}", err)
            }
        }

        log::set_max_level(level.to_level_filter());
    }
}

impl log::Log for ToyLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.0
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
