use std::cell::RefCell;
use std::time;
use std::fmt;
use log;

/// This module implements a Timer which allow blocks of code to be timed.
/// A message is logged when the Timer is dropped. The message can be extended
/// with extra information. Two macros can simplify the creation of timers.
#[derive(Debug, Clone)]
pub struct Timer<'a> {
    name: &'a str,
    start_time: time::Instant,
    log_level: log::Level,
    log_on_drop: bool,
    message: RefCell<Option<String>>
}

impl<'a> Timer<'a> {
    /// Constructs a new timer at Trace log level which logs only when dropped.
    pub fn new(name: &'a str) -> Timer<'a> {
        Timer {
            name: name,
            start_time: time::Instant::now(),
            log_level: log::Level::Trace,
            log_on_drop: true,
            message: RefCell::new(None)
        }
    }

    /// Construct a new Timer that logs a message at the start
    /// as well as at the end. You must specify the log level
    /// because the first message is logged immediately.
    pub fn bracket_timer(name: &'a str, log_level: log::Level) -> Timer<'a> {
        let mut tmr = Self::new(name);
        tmr.log_level = log_level;
        log!(log_level, "Starting {}", name);
        tmr
    }

    /// Sets or unsets the `log_on_drop` flag for this timer.
    /// If false, no message is logged when the timer is dropped.
    /// This may be useful if you have taken over the output yourself.
    pub fn log_on_drop(mut self, log_on_drop: bool) -> Self {
        self.log_on_drop = log_on_drop;
        self
    }

    /// Sets the logging level. The default is `Level::Trace`.
    pub fn at_level(mut self, log_level: log::Level) -> Self {
        self.log_level = log_level;
        self
    }

    /// Returns the current duration the timer has been running for.
    pub fn elapsed(&self) -> time::Duration {
        self.start_time.elapsed()
    }

    /// Number of whole seconds elapsed.
    pub fn seconds(&self) -> u64 {
        self.elapsed().as_secs()
    }

    /// Number of whole milliseconds elapsed.
    pub fn millis(&self) -> u64 {
        self.seconds() * 1000 + self.subsec_nanos() / 1_000_000
    }

    /// Number of whole microseconds elapsed.
    pub fn micros(&self) -> u64 {
        self.seconds() * 1_000_000 + self.subsec_nanos() / 1_000
    }

    /// Number of whole nanoseconds elapsed.
    pub fn nanos(&self) -> u64 {
        self.seconds() * 1_000_000_000 + self.subsec_nanos()
    }

    /// Sets the extra message to be displayed when the timer is dropped.
    /// If there is any existing message, it is completely replaced.
    /// Passing an empty string will clear the message.
    pub fn set_message<S>(&self, message: S)
        where S: Into<String>
    {
        let message = message.into();

        let mut brr = self.message.borrow_mut();
        if message.len() == 0 {
            *brr = None;
        } else {
            *brr = Some(message);
        }
    }

    /// Appends to the extra message to be displayed when the timer is dropped.
    /// It is your responsibility to add any extra whitespace you may need
    /// to produce a nice looking message.
    pub fn append_message<S>(&self, message: S)
        where S: Into<String>
    {
        let message = message.into();

        let mut brr = self.message.borrow_mut();
        if brr.is_none() {
            *brr = Some(message);
        } else {
            let curr_msg = brr.as_mut().unwrap();
            curr_msg.push_str(&message);
        }
    }
    
    fn subsec_nanos(&self) -> u64 {
        self.elapsed().subsec_nanos() as u64
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        if self.log_on_drop {
            log!(self.log_level, "{}", self);
        }
    }
}

/// Implementing fmt::Display gives us an automatic 'to_string()' method.
impl<'a> fmt::Display for Timer<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (d1, d2, t) = if self.seconds() > 0 {
            (self.seconds(), self.millis(), "s")
        } else if self.millis() > 0 {
            (self.millis(), self.micros(), "ms")
        } else if self.micros() > 0 {
            (self.micros(), self.nanos(), "μs")
        } else {
            (self.nanos(), self.nanos() * 1000, "ns")
        };

        let frac_time = d1 as f64 + ((d2 - d1 * 1000) as f64) / 1000.0;
        let msg = self.message.borrow();
        if msg.is_none() {
            write!(f, "Completed {}, elapsed = {:.2} {}", self.name, frac_time, t)
        } else {
            let msg = msg.as_ref().unwrap();
            write!(f, "Completed {}, elapsed = {:.2} {} {}", self.name, frac_time, t, msg)
        }
    }
}


#[macro_use]
mod macros {
    /// Creates a quiet timer that does not log a starting message, only a completed one.
    #[macro_export]
    macro_rules! timer {
        ($name:expr) => { ::timer::Timer::new($name) }
    }

    /// Creates a bracket timer that logs a starting and completed message.
    #[macro_export]
    macro_rules! bracket_timer {
        ($name:expr, $level:expr) => { ::timer::Timer::bracket_timer($name, $level) }
    }
}

