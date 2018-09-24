use std::time::{Duration, Instant};
use std::fmt;
use std::io::Write;

/// This module implements macros which allow blocks of code to be timed.
/// Macros log the timings.

pub struct ElapsedDuration(Duration);

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


*/

/*
// When this struct is dropped, it logs a message stating its name and how long, in seconds,
// execution time was. Can be used to time functions or other critical areas.
pub struct ExecutionTimer<'a> {
        start_time: Instant,
        name: &'a str
}

impl<'a> ExecutionTimer<'a> {
        pub fn new(name: &'a str) -> ExecutionTimer<'a> {
                ExecutionTimer {
                        start_time: Instant::now(),
                        name: name
                }
        }

        // Construct a new ExecutionTimer and prints a message saying execution is starting.
        pub fn with_start_message(name: &'a str) -> ExecutionTimer<'a> {
                debug!("Execution Starting, Name={}", name);
                ExecutionTimer {
                        start_time: Instant::now(),
                        name: name
                }
        }
}

impl<'a> Drop for ExecutionTimer<'a> {
        fn drop(&mut self) {
                let elapsed = self.start_time.elapsed();
        let secs = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        let msecs = secs * 1000.0;
                debug!("Execution Completed, Name={}, MilliSecs={:.3}", self.name, msecs);
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
