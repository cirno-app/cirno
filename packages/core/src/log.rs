use ::log::{Log, Metadata, Record, set_boxed_logger, set_max_level};
use arc_swap::ArcSwap;
use std::sync::Arc;

pub struct CombinedLogger {
    loggers: ArcSwap<Vec<Arc<dyn Log>>>,
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
        set_max_level(log::LevelFilter::Trace);
        combined_logger
    }

    fn new() -> CombinedLogger {
        CombinedLogger {
            loggers: ArcSwap::from_pointee(vec![]),
        }
    }

    pub fn push(&self, logger: Arc<dyn Log>) {
        self.loggers.rcu(|l_old| {
            let mut l_new = Vec::clone(l_old);
            l_new.push(logger.clone());
            l_new
        });
    }
}

impl Log for CombinedLoggerLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        for logger in self.intl.loggers.load().iter() {
            if logger.enabled(metadata) {
                return true;
            }
        }

        false
    }

    fn log(&self, record: &Record) {
        for logger in self.intl.loggers.load().iter() {
            logger.log(record);
        }
    }

    fn flush(&self) {
        for logger in self.intl.loggers.load().iter() {
            logger.flush();
        }
    }
}
