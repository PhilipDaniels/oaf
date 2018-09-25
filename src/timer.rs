use std::time;
use std::fmt;
use std::io::Write;
use log;

/// This module implements macros which allow blocks of code to be timed.
/// Macros log the timings.
#[derive(Debug, Clone)]
pub struct Timer<'a> {
    name: &'a str,
    start_time: time::Instant,
    log_level: log::Level,
    log_on_drop: bool,

        //units: Units // Auto, msec, usec, nsec
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        Timer {
            name: name,
            start_time: time::Instant::now(),
            log_level: log::Level::Trace,
            log_on_drop: true
        }
    }

    /// Construct a new Timer that logs a message at the start
    /// as well as at the end. You must specify the log level
    /// because the first message is logged immediately.
    pub fn bracket_timer(name: &'a str, log_level: log::Level) -> Timer<'a> {
        let mut tmr = Self::new(name);
        tmr.log_level = log_level;
        tmr.output(format!("Starting {}", name));
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

    fn subsec_nanos(&self) -> u64 {
        self.elapsed().subsec_nanos() as u64
    }






    /// Returns the current message of the timer.
    pub fn display(&self) -> String {
        let elapsed = self.elapsed();
        let secs = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        let msecs = secs * 1000.0;
        format!("Completed {}, Time={:.3}", self.name, msecs)
    }

    /// Private function to log at the appropriate level.
    fn output<S>(&self, message: S)
        where S: AsRef<str>
    {
        log!(self.log_level, "{}", message.as_ref());
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        if self.log_on_drop {
            self.output(self.display());
        }
    }
}


/*
Our cases are:

0. We want to control the log level: info or debug is sufficient.
   We want to control the units.

1. We want to log a pair of FIXED messages.
   Starting {thing}.
   Completed {thing}, time = {time}.

2. We want to log a single FIXED completion message.
   Completed {thing}, time = {time}.

3. We want to log a message containing information that we don't know until we have finished.
   Completed {thing}, time = {time}. Wrote {n} lines to MRU file {file}.

4. Alternative form of 3.
   Wrote {n} lines to MRU file {file}.
   Completed {thing}, time = {time}.

1 and 2 we can handle with the previous macros.
4 might be acceptable to start with, but 3 is better.
It appears we can do this by just adding a message() function that implements the formatting trick.

    let _timer = timer!("MyName");

    let _timer2 = end_timer!("MyQuietTimer");
    _timer2.message("Wrote {} lines to MRU file {}", num_lines, filename);

// Bonus methods. Get the string that will be written.
let msg = _timer2.display();
_timer2.suppress_logging();

*/

/*
impl<'a> ExecutionTimer<'a> {
}

}

#[macro_use]
mod macros {
    /// Creates a timer that logs a starting and completed message.
    #[macro_export]
    macro_rules! timer {
        ($str:expr) => { ::execution_timer::ExecutionTimer::with_start_message($str) }
    }

    /// Creates a quiet timer that does not log a starting message, only a completed one.
    #[macro_export]
    macro_rules! quiet_timer {
        ($str:expr) => { ::execution_timer::ExecutionTimer::new($str) }
    }
}
*/
