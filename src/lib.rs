mod loggable;

pub use chrono;
use chrono::Utc;
use loggable::{ErasedLoggable, Loggable};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use strum::AsRefStr;

/// The level of a log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    /// The time this log was registered. This is _not_ the same as the time it was `.log`ged, as it
    /// might take some time for it to actually be processed by the backing thread.
    pub time: chrono::DateTime<Utc>,
    pub level: Level,
    pub message: String,
}

struct LogBuilder {
    level: Level,
    loggable: ErasedLoggable,
}

/// A type which can be used for logging events. It is cheaply clonable and cloning it will create
/// a logger that shares the same storage as this one for logs.
#[derive(Clone)]
pub struct Logger {
    logs: Arc<Mutex<VecDeque<Log>>>,
    sender: flume::Sender<LogBuilder>,
}

impl Logger {
    /// Creates a new [`Logger`] with a given `limit` for the amount of logs. Behind the scenes, this
    /// spawns a thread for which this logger will send all it's logging tasks to.
    ///
    /// When the limit is exceeded, the oldest logs are deleted.
    pub fn new(limit: Option<usize>) -> Self {
        let (sender, receiver) = flume::unbounded::<LogBuilder>();
        let logs = Arc::new(Mutex::new(VecDeque::with_capacity(limit.unwrap_or(0))));

        std::thread::spawn({
            let logs = logs.clone();

            move || {
                while let Ok(builder) = receiver.recv() {
                    let mut buf = String::new();
                    builder.loggable.log_to(&mut buf).expect("logging ok");
                    buf.shrink_to_fit();

                    let mut logs = logs.lock().expect("lock is not poisoned");
                    if limit.is_some_and(|limit| logs.len() == limit) {
                        logs.pop_front();
                    }

                    logs.push_back(Log {
                        time: Utc::now(),
                        level: builder.level,
                        message: buf,
                    });

                    std::mem::drop(logs);
                }
            }
        });

        Self { logs, sender }
    }

    /// Logs a value `l` with the given [`Level`]. This method might allocate depending on the size
    /// of `l` - values smaller than or equal to 24 bytes do not allocate.
    #[inline]
    pub fn log<L>(&self, level: Level, l: L)
    where
        L: Loggable + 'static,
    {
        self.sender
            .send(LogBuilder {
                level,
                loggable: ErasedLoggable::new(l),
            })
            .expect("channel is open");
    }

    /// Calls a function with read access to all the [`Log`]s. Note that this might not show a
    /// recently logged value as it might not have been processed by the backing thread yet.
    ///
    /// The logs are split into two slices because of implementation details but they are sequential.
    #[inline]
    pub fn with_logs<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&VecDeque<Log>) -> O,
    {
        let logs = self.logs.lock().expect("lock is not poisoned");
        f(&logs)
    }

    /// Clear the log buffer.
    #[inline]
    pub fn clear(&self) {
        let mut logs = self.logs.lock().expect("lock is not poisoned");
        logs.clear();
        logs.shrink_to_fit();
    }
}

#[macro_export]
macro_rules! debug {
    ($logger:expr, $value:expr) => {
        $logger.log($crate::Level::Debug, $value);
    };
    ($logger:expr, $s:literal, $_0:expr) => {{
        let _0 = $_0;
        $logger.log(
            $crate::Level::Debug,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        $logger.log(
            $crate::Level::Debug,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        $logger.log(
            $crate::Level::Debug,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr, $_3:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        let _3 = $_3;
        $logger.log(
            $crate::Level::Debug,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2, _3),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr, $_3:expr, $_4:expr $(,)*) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        let _3 = $_3;
        let _4 = $_4;
        $logger.log(
            $crate::Level::Debug,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2, _3, _$),
        );
    }};
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $value:expr) => {
        $logger.log($crate::Level::Info, $value);
    };
    ($logger:expr, $s:literal, $_0:expr) => {{
        let _0 = $_0;
        $logger.log(
            $crate::Level::Info,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        $logger.log(
            $crate::Level::Info,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        $logger.log(
            $crate::Level::Info,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr, $_3:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        let _3 = $_3;
        $logger.log(
            $crate::Level::Info,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2, _3),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr, $_3:expr, $_4:expr $(,)*) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        let _3 = $_3;
        let _4 = $_4;
        $logger.log(
            $crate::Level::Info,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2, _3, _$),
        );
    }};
}

#[macro_export]
macro_rules! warn {
    ($logger:expr, $value:expr) => {
        $logger.log($crate::Level::Warn, $value);
    };
    ($logger:expr, $s:literal, $_0:expr) => {{
        let _0 = $_0;
        $logger.log(
            $crate::Level::Warn,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        $logger.log(
            $crate::Level::Warn,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        $logger.log(
            $crate::Level::Warn,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr, $_3:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        let _3 = $_3;
        $logger.log(
            $crate::Level::Warn,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2, _3),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr, $_3:expr, $_4:expr $(,)*) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        let _3 = $_3;
        let _4 = $_4;
        $logger.log(
            $crate::Level::Warn,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2, _3, _$),
        );
    }};
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $value:expr) => {
        $logger.log($crate::Level::Error, $value);
    };
    ($logger:expr, $s:literal, $_0:expr) => {{
        let _0 = $_0;
        $logger.log(
            $crate::Level::Error,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        $logger.log(
            $crate::Level::Error,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        $logger.log(
            $crate::Level::Error,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr, $_3:expr) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        let _3 = $_3;
        $logger.log(
            $crate::Level::Error,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2, _3),
        );
    }};
    ($logger:expr, $s:literal, $_0:expr, $_1:expr, $_2:expr, $_3:expr, $_4:expr $(,)*) => {{
        let _0 = $_0;
        let _1 = $_1;
        let _2 = $_2;
        let _3 = $_3;
        let _4 = $_4;
        $logger.log(
            $crate::Level::Error,
            move |writer: &mut dyn ::std::fmt::Write| write!(writer, $s, _0, _1, _2, _3, _$),
        );
    }};
}

#[cfg(test)]
mod test {
    use crate::Logger;
    use std::{thread::sleep, time::Duration};

    #[test]
    fn simple() {
        let logger = Logger::new(None);
        debug!(logger, "hello there: {}", 0);

        sleep(Duration::from_secs_f32(0.1));

        logger.with_logs(|logs| {
            dbg!(logs);
        });
    }
}
