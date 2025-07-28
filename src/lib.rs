#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "ufmt"))]
use core::fmt::Debug;
#[cfg(not(feature = "ufmt"))]
use core::fmt::Display;
#[cfg(feature = "ufmt")]
use core::time::Duration;

#[cfg(feature = "ufmt")]
use ufmt::{uDebug, uWrite};

macro_rules! define_colors {
    ($($name:ident => $color:expr),* $(,)?) => {
        $(
            #[cfg(feature = "colors")]
            const $name: &str = $color;
            #[cfg(not(feature = "colors"))]
            const $name: &str = "";
        )*
    };
}

define_colors! {
    RESET => "\x1b[0m",
    LIGHT_GREEN => "\x1b[92m",
    LIGHT_BLUE => "\x1b[94m",
    LIGHT_RED => "\x1b[91m",
    LIGHT_YELLOW => "\x1b[93m",
    RED => "\x1b[31m",
}

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

macro_rules! impl_status_format {
    (
        $self:expr,$formatter:ident, $write_macro:ident,
        $($variant:ident => $symbol:expr, $color:expr),* $(,)?
    ) => {
        match $self {
            $(
                StatusLevel::$variant => $write_macro!($formatter, "{}{}&:{}", $color, $symbol, RESET)?,
            )*
        }
    };
}

#[cfg(feature = "ufmt")]
impl uDebug for StatusLevel {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        use ufmt::uwrite;

        impl_status_format!(self,f, uwrite,
            Ok => "O", LIGHT_GREEN,
            Info => "I", LIGHT_BLUE,
            Error => "E", LIGHT_RED,
            Warning => "W", LIGHT_YELLOW,
            Critical => "C", RED,
        );

        Ok(())
    }
}

#[cfg(not(feature = "ufmt"))]
impl Debug for StatusLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        impl_status_format!(self,f, write,
            Ok => "O", LIGHT_GREEN,
            Info => "I", LIGHT_BLUE,
            Error => "E", LIGHT_RED,
            Warning => "W", LIGHT_YELLOW,
            Critical => "C", RED,
        );
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

#[cfg(not(feature = "ufmt"))]
macro_rules! impl_log_methods {
    ($($method:ident => $level:expr),* $(,)?) => {
        $(
            pub fn $method(&mut self, args: impl Display) {
                self.logdisp($level, args);
            }
        )*
    };
}

macro_rules! impl_try_get {
    ($error_bound:path, owned) => {
        // For Logger - takes ownership
        #[cfg(feature = "std")]
        pub fn try_get<O, E: $error_bound>(
            mut self, // Takes ownership
            tryresult: Result<O, E>,
            redirectfn: fn(Self) -> (),
        ) -> (O, Self) {
            match tryresult {
                Ok(x) => (x, self),
                Err(err) => {
                    self.log(StatusLevel::Warning, err);
                    redirectfn(self);
                    std::process::exit(1);
                }
            }
        }

        #[cfg(not(feature = "std"))]
        pub fn try_get<O, E: $error_bound>(
            mut self, // Takes ownership
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
    };

    ($error_bound:path, cloned) => {
        // For MultiLogger - needs cloning
        #[cfg(feature = "std")]
        pub fn try_get<O, E: $error_bound>(
            &mut self, // Takes reference
            tryresult: Result<O, E>,
            redirectfn: fn(Self) -> (),
        ) -> (O, Self) {
            let mut new_self = self.clone();
            match tryresult {
                Ok(x) => (x, new_self),
                Err(err) => {
                    new_self.log(StatusLevel::Warning, err);
                    redirectfn(new_self);
                    std::process::exit(1);
                }
            }
        }

        #[cfg(not(feature = "std"))]
        pub fn try_get<O, E: $error_bound>(
            &mut self, // Takes reference
            tryresult: Result<O, E>,
            redirectfn: fn(Self) -> (),
        ) -> (O, Self) {
            let mut new_self = self.clone();
            match tryresult {
                Ok(x) => (x, new_self),
                Err(err) => {
                    new_self.log(StatusLevel::Warning, err);
                    redirectfn(new_self);
                    loop {}
                }
            }
        }
    };
}

/// Same as Logger but Clonable for tryrun
#[derive(Clone)]
pub struct MultiLogger<T: TimeProvider + Clone, S: StorageProvider + Clone>(pub T, pub S);

pub struct Logger<T: TimeProvider, S: StorageProvider>(pub T, pub S);

#[cfg(not(feature = "ufmt"))]
impl<'a, T: TimeProvider + Clone, S: StorageProvider + Clone> MultiLogger<T, S>
where
    Self: Clone,
{
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

    impl_log_methods! {
        log_err => StatusLevel::Error,
        log_ok => StatusLevel::Ok,
        log_warn => StatusLevel::Warning,
        log_info => StatusLevel::Info,
    }

    pub fn try_run<O, E: core::fmt::Debug>(&mut self, tryresult: Result<O, E>) {
        // Returns nothing
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    impl_try_get!(core::fmt::Debug, cloned);
}

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

    impl_log_methods! {
        log_err => StatusLevel::Error,
        log_ok => StatusLevel::Ok,
        log_warn => StatusLevel::Warning,
        log_info => StatusLevel::Info,
    }

    pub fn try_run<O, E: core::fmt::Debug>(&mut self, tryresult: Result<O, E>) {
        // Returns nothing
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    impl_try_get!(core::fmt::Debug, owned);
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
macro_rules! impl_log_methods_ufmt {
    ($($method:ident => $level:expr),* $(,)?) => {
        $(
            pub fn $method(&mut self, args: &str) {
                self.logdisp($level, args);
            }
        )*
    };
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

    impl_log_methods_ufmt! {
        log_err => StatusLevel::Error,
        log_ok => StatusLevel::Ok,
        log_warn => StatusLevel::Warning,
        log_info => StatusLevel::Info,
    }

    pub fn try_run<O, E: ufmt::uDebug>(&mut self, tryresult: Result<O, E>) {
        // Returns nothing
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    impl_try_get!(ufmt::uDebug, owned);
}
#[cfg(feature = "ufmt")]
impl<'a, T: TimeProvider + Clone, S: StorageProvider + Clone> MultiLogger<T, S>
where
    Self: Clone,
{
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

    impl_log_methods_ufmt! {
        log_err => StatusLevel::Error,
        log_ok => StatusLevel::Ok,
        log_warn => StatusLevel::Warning,
        log_info => StatusLevel::Info,
    }

    pub fn try_run<O, E: ufmt::uDebug>(&mut self, tryresult: Result<O, E>) {
        // Returns nothing
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    impl_try_get!(ufmt::uDebug, cloned);
}
