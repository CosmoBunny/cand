#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "ufmt"))]
use core::fmt::Debug;
#[cfg(not(feature = "ufmt"))]
use core::fmt::Display;
#[cfg(feature = "ufmt")]
use core::time::Duration;

#[cfg(feature = "ufmt")]
use ufmt::{uDebug, uWrite};

#[cfg(feature = "colors")]
const RESET: &str = "\x1b[0m";
#[cfg(feature = "colors")]
const LIGHT_GREEN: &str = "\x1b[92m";
#[cfg(feature = "colors")]
const LIGHT_BLUE: &str = "\x1b[94m";
#[cfg(feature = "colors")]
const LIGHT_RED: &str = "\x1b[91m";
#[cfg(feature = "colors")]
const LIGHT_YELLOW: &str = "\x1b[93m";
#[cfg(feature = "colors")]
const RED: &str = "\x1b[31m";

#[cfg(not(feature = "colors"))]
const RESET: &str = "";
#[cfg(not(feature = "colors"))]
const LIGHT_GREEN: &str = "";
#[cfg(not(feature = "colors"))]
const LIGHT_BLUE: &str = "";
#[cfg(not(feature = "colors"))]
const LIGHT_RED: &str = "";
#[cfg(not(feature = "colors"))]
const LIGHT_YELLOW: &str = "";
#[cfg(not(feature = "colors"))]
const RED: &str = "";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Ok = 0,
    Info = 1,
    Error = 2,
    Warning = 3,
    Critical = 4,
}

impl StatusLevel {
    fn to_color(&self) -> &'static str {
        match self {
            StatusLevel::Ok => LIGHT_GREEN,
            StatusLevel::Info => LIGHT_BLUE,
            StatusLevel::Error => LIGHT_RED,
            StatusLevel::Warning => LIGHT_YELLOW,
            StatusLevel::Critical => RED,
        }
    }
}

#[cfg(feature = "ufmt")]
impl uDebug for StatusLevel {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        use ufmt::uwrite;

        match self {
            StatusLevel::Ok => uwrite!(f, "{}O&:{}", LIGHT_GREEN, RESET)?,
            StatusLevel::Info => uwrite!(f, "{}I&:{}", LIGHT_BLUE, RESET)?,
            StatusLevel::Error => uwrite!(f, "{}E&:{}", LIGHT_RED, RESET)?,
            StatusLevel::Warning => uwrite!(f, "{}W&:{}", LIGHT_YELLOW, RESET)?,
            StatusLevel::Critical => uwrite!(f, "{}C&:{}", RED, RESET)?,
        }
        Ok(())
    }
}

#[cfg(not(feature = "ufmt"))]
impl Debug for StatusLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StatusLevel::Ok => write!(f, "{}O&:{}", LIGHT_GREEN, RESET)?,
            StatusLevel::Info => write!(f, "{}I&:{}", LIGHT_BLUE, RESET)?,
            StatusLevel::Error => write!(f, "{}E&:{}", LIGHT_RED, RESET)?,
            StatusLevel::Warning => write!(f, "{}W&:{}", LIGHT_YELLOW, RESET)?,
            StatusLevel::Critical => write!(f, "{}C&:{}", RED, RESET)?,
        }
        Ok(())
    }
}

#[cfg(feature = "ufmt")]
pub trait StorageProvider {
    fn write_data(&mut self, d: impl uDebug);
}

#[cfg(not(feature = "ufmt"))]
use core::fmt::Arguments;
#[cfg(not(feature = "ufmt"))]
pub trait StorageProvider {
    /// Write log data directly - single responsibility
    fn write_data(&mut self, args: Arguments);
}

#[cfg(feature = "std")]
#[cfg(not(feature = "ufmt"))]
impl StorageProvider for () {
    fn write_data(&mut self, args: Arguments<'_>) {
        print!("{args}")
    }
}

// Your existing traits:
pub trait TimeProvider {
    fn now() -> Self;
    fn elapsed(&self) -> core::time::Duration;
    fn write(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result;
}

#[cfg(feature = "std")]
use std::time::Instant;

#[cfg(feature = "std")]
impl TimeProvider for Instant {
    fn now() -> Self {
        Instant::now()
    }
    fn elapsed(&self) -> core::time::Duration {
        self.elapsed()
    }
    fn write(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}:", self.elapsed())?;
        Ok(())
    }
}

impl TimeProvider for () {
    fn now() -> Self {}
    fn elapsed(&self) -> core::time::Duration {
        core::time::Duration::ZERO
    }
    fn write(&self, _f: &mut core::fmt::Formatter) -> core::fmt::Result {
        Ok(())
    }
}

pub struct Logger<T: TimeProvider, S: StorageProvider>(pub T, pub S);

#[cfg(not(feature = "ufmt"))]
impl<'a, T: TimeProvider, S: StorageProvider> Logger<T, S> {
    pub fn log(&mut self, level: StatusLevel, args: impl Debug) {
        self.1.write_data(format_args!(
            "{:?}{} {}{:?}{}\n",
            level,
            TimeFormatter(&self.0),
            level.to_color(),
            args,
            RESET
        ));
    }

    pub fn logdisp(&mut self, level: StatusLevel, args: impl Display) {
        self.1.write_data(format_args!(
            "{:?}{} {}{}{}\n",
            level,
            TimeFormatter(&self.0),
            level.to_color(),
            args,
            RESET
        ));
    }

    pub fn log_err(&mut self, args: impl Display) {
        self.logdisp(StatusLevel::Error, args);
    }
    pub fn log_ok(&mut self, args: impl Display) {
        self.logdisp(StatusLevel::Ok, args);
    }
    pub fn log_warn(&mut self, args: impl Display) {
        self.logdisp(StatusLevel::Warning, args);
    }
    pub fn log_info(&mut self, args: impl Display) {
        self.logdisp(StatusLevel::Info, args);
    }

    #[cfg(feature = "std")]
    pub fn try_run<O, E: core::fmt::Debug>(
        mut self,
        tryresult: Result<O, E>,
        redirectfn: fn(Self) -> (),
    ) -> (O, Self) {
        match tryresult {
            Ok(x) => (x, self),
            Err(err) => {
                self.log(StatusLevel::Error, err);
                redirectfn(self);
                std::process::exit(1);
            }
        }
    }
    #[cfg(not(feature = "std"))]
    pub fn try_run<O, E: core::fmt::Debug>(
        mut self,
        tryresult: Result<O, E>,
        redirectfn: fn(Self) -> (),
    ) -> (O, Self) {
        match tryresult {
            Ok(x) => (x, self),
            Err(err) => {
                self.log(StatusLevel::Warning, err);
                redirectfn(self);
                loop {}
            }
        }
    }
}

// Helper struct to use TimeProvider's write method with Display trait
#[cfg(not(feature = "ufmt"))]
struct TimeFormatter<'a, T: TimeProvider>(&'a T);

#[cfg(not(feature = "ufmt"))]
impl<'a, T: TimeProvider> core::fmt::Display for TimeFormatter<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.write(f)
    }
}

#[cfg(feature = "ufmt")]
pub struct UDebugStr<'a>(pub &'a str);

#[cfg(feature = "ufmt")]
impl<'a> uDebug for UDebugStr<'a> {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        f.write_str(self.0)?;
        Ok(())
    }
}

#[cfg(feature = "ufmt")]
struct UDebugDuration(Duration);

#[cfg(feature = "ufmt")]
impl uDebug for UDebugDuration {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        use ufmt::uwrite;

        let nanos = self.0.as_nanos();
        let micros = self.0.as_micros();
        let millis = self.0.as_millis();
        let secs = self.0.as_secs();

        if nanos < 1_000 {
            // Nanoseconds: 0-999ns
            uwrite!(f, "{}ns", nanos as u32)
        } else if micros < 1_000 {
            // Microseconds: 1-999μs
            uwrite!(f, "{}μs", micros as u32)
        } else if millis < 1_000 {
            // Milliseconds: 1-999ms
            uwrite!(f, "{}ms", millis as u32)
        } else if secs < 60 {
            // Seconds: 1.000s - 59.999s
            let remaining_millis = (millis % 1000) as u32;
            uwrite!(f, "{}.{}s", secs, remaining_millis)
        } else if secs < 3600 {
            // Minutes: 1:00 - 59:59
            let mins = secs / 60;
            let remaining_secs = secs % 60;
            uwrite!(f, "{}:{}min", mins, remaining_secs)
        } else if secs < 86400 {
            // Hours: 1:00:00 - 23:59:59
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let remaining_secs = secs % 60;
            uwrite!(f, "{}:{}:{}", hours, mins, remaining_secs)
        } else {
            // Days: 1d, 1d2h, etc.
            let days = secs / 86400;
            let hours = (secs % 86400) / 3600;
            if hours > 0 {
                uwrite!(f, "{}d{}h", days, hours)
            } else {
                uwrite!(f, "{}d", days)
            }
        }
    }
}

#[cfg(feature = "ufmt")]
impl<'a, T: TimeProvider, S: StorageProvider> Logger<T, S> {
    pub fn log(&mut self, level: StatusLevel, args: impl uDebug) {
        let timestamp = self.0.elapsed();
        // Format timestamp and level + args + write to store_log

        self.1.write_data(level);
        self.1.write_data(UDebugDuration(timestamp));
        self.1.write_data(UDebugStr(level.to_color()));
        self.1.write_data(args);
        self.1.write_data(UDebugStr(RESET));
        self.1.write_data(UDebugStr("\n"));
    }

    pub fn logdisp(&mut self, level: StatusLevel, args: &str) {
        let timestamp = self.0.elapsed();

        self.1.write_data(level);
        self.1.write_data(UDebugDuration(timestamp));
        self.1.write_data(UDebugStr(level.to_color()));
        self.1.write_data(UDebugStr(args)); // Clean string, no quotes
        self.1.write_data(UDebugStr(RESET));
        self.1.write_data(UDebugStr("\n"));
    }

    pub fn log_err(&mut self, args: &str) {
        self.logdisp(StatusLevel::Error, args);
    }
    pub fn log_ok(&mut self, args: &str) {
        self.logdisp(StatusLevel::Ok, args);
    }
    pub fn log_warn(&mut self, args: &str) {
        self.logdisp(StatusLevel::Warning, args);
    }
    pub fn log_info(&mut self, args: &str) {
        self.logdisp(StatusLevel::Info, args);
    }

    #[cfg(feature = "std")]
    pub fn try_run<O, E: core::fmt::Debug>(
        mut self,
        tryresult: Result<O, E>,
        redirectfn: fn(Self) -> (),
    ) -> (O, Self) {
        match tryresult {
            Ok(x) => (x, self),
            Err(err) => {
                // Use UDebugStr wrapper for clean error display
                let err_str = format!("{:?}", err);
                self.log_err(&err_str);
                redirectfn(self);
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(feature = "std"))]
    pub fn try_run<O, E: core::fmt::Debug>(
        mut self,
        tryresult: Result<O, E>,
        redirectfn: fn(Self) -> (),
    ) -> (O, Self) {
        match tryresult {
            Ok(x) => (x, self),
            Err(err) => {
                self.log(StatusLevel::Warning, err);
                redirectfn(self);
                loop {}
            }
        }
    }
}
