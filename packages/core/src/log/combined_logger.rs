use ::log::{Log, Metadata, Record, set_boxed_logger};
use std::sync::Arc;

pub struct CombinedLogger {
    logger: Vec<Box<dyn Log>>,
}

struct CombinedLoggerLog {
    intl: Arc<CombinedLogger>,
}

impl CombinedLogger {
    pub fn init() -> Arc<CombinedLogger> {
        let combined_logger = Arc::new(CombinedLogger::new());
        let combined_logger_log = CombinedLoggerLog {
            intl: combined_logger.clone(),
        };
        set_boxed_logger(Box::new(combined_logger_log)).unwrap();
        combined_logger
    }

    fn new() -> CombinedLogger {
        CombinedLogger { logger: vec![] }
    }

    pub fn push(&mut self, logger: Box<dyn Log>) {
        self.logger.push(logger);
    }
}

impl Log for CombinedLoggerLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        for logger in &self.intl.logger {
            if logger.enabled(metadata) {
                return true;
            }
        }

        false
    }

    fn log(&self, record: &Record) {
        for logger in &self.intl.logger {
            logger.log(record);
        }
    }

    fn flush(&self) {
        for logger in &self.intl.logger {
            logger.flush();
        }
    }
}
