static mut BOY_LOGGER: BoyLogger = BoyLogger(log::Level::Warn);

pub struct BoyLogger(log::Level);

impl BoyLogger {
    pub fn init(level: log::Level) {
        unsafe {
            BOY_LOGGER.0 = level;

            if let Err(err) = log::set_logger(&BOY_LOGGER) {
                panic!("ERROR: {}", err)
            }
        }

        log::set_max_level(level.to_level_filter());
    }
}

impl log::Log for BoyLogger {
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
