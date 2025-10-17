#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt::{Debug, Display};

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

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "ufmt")]
pub trait UStorageProvider {
    fn write_data(&mut self, d: impl uDebug);
}

use core::fmt::Arguments;

pub trait StorageProvider {
    /// Write log data directly - single responsibility
    fn write_data(&mut self, args: Arguments, debuglevel: &StatusLevel);
}

#[cfg(feature = "std")]
impl StorageProvider for () {
    fn write_data(&mut self, args: Arguments<'_>, _debuglevel: &StatusLevel) {
        print!("{args}")
    }
}

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
        #[cfg(feature = "std")]
        #[cfg(feature = "ufmt")]
        pub fn try_get<O>(
            mut self, // Takes ownership
            tryresult: Result<O, Box<dyn core::error::Error>>,
            redirectfn: fn(Self) -> (),
        ) -> (O, Self) {
            match tryresult {
                Ok(x) => (x, self),
                Err(err) => {
                    self.log(StatusLevel::Warning, UDebugStr(&err.to_string()));
                    redirectfn(self);
                    std::process::exit(1);
                }
            }
        }

        #[cfg(feature = "std")]
        #[cfg(not(feature = "ufmt"))]
        pub fn try_get<O>(
            mut self, // Takes ownership
            tryresult: Result<O, Box<dyn core::error::Error>>,
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
        #[cfg(feature = "std")]
        #[cfg(not(feature = "ufmt"))]
        pub fn try_get<O>(
            &mut self, // Takes reference
            tryresult: Result<O, Box<dyn core::error::Error>>,
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
        #[cfg(feature = "std")]
        #[cfg(feature = "ufmt")]
        pub fn try_get<O>(
            &mut self, // Takes reference
            tryresult: Result<O, Box<dyn core::error::Error>>,
            redirectfn: fn(Self) -> (),
        ) -> (O, Self) {
            let mut new_self = self.clone();
            match tryresult {
                Ok(x) => (x, new_self),
                Err(err) => {
                    new_self.log(StatusLevel::Warning, UDebugStr(&err.to_string()));
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

#[derive(Clone)]
pub struct MultiLogger<T: TimeProvider + Clone, S: StorageProvider + Clone>(pub T, pub S);

pub struct Logger<T: TimeProvider, S: StorageProvider>(pub T, pub S);

impl<'a, T: TimeProvider + Clone, S: StorageProvider + Clone> MultiLogger<T, S>
where
    Self: Clone,
{
    pub fn log(&mut self, level: StatusLevel, args: impl Debug) {
        self.1.write_data(
            format_args!(
                "{:?}{} {}{:?}{}\n",
                level,
                TimeFormatter(&self.0),
                level.to_color(),
                args,
                RESET
            ),
            &level,
        );
    }

    pub fn logdisp(&mut self, level: StatusLevel, args: impl Display) {
        self.1.write_data(
            format_args!(
                "{:?}{} {}{}{}\n",
                level,
                TimeFormatter(&self.0),
                level.to_color(),
                args,
                RESET
            ),
            &level,
        );
    }

    impl_log_methods! {
        log_err => StatusLevel::Error,
        log_ok => StatusLevel::Ok,
        log_warn => StatusLevel::Warning,
        log_info => StatusLevel::Info,
    }

    #[cfg(feature = "alloc")]
    pub fn try_run<O>(&mut self, tryresult: Result<O, Box<dyn core::error::Error>>) {
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn try_run<O, E: core::fmt::Debug>(&mut self, tryresult: Result<O, E>) {
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    impl_try_get!(core::fmt::Debug, cloned);
}

impl<'a, T: TimeProvider, S: StorageProvider> Logger<T, S> {
    pub fn log(&mut self, level: StatusLevel, args: impl Debug) {
        self.1.write_data(
            format_args!(
                "{:?}{} {}{:?}{}\n",
                level,
                TimeFormatter(&self.0),
                level.to_color(),
                args,
                RESET
            ),
            &level,
        );
    }

    pub fn logdisp(&mut self, level: StatusLevel, args: impl Display) {
        self.1.write_data(
            format_args!(
                "{:?}{} {}{}{}\n",
                level,
                TimeFormatter(&self.0),
                level.to_color(),
                args,
                RESET
            ),
            &level,
        );
    }

    impl_log_methods! {
        log_err => StatusLevel::Error,
        log_ok => StatusLevel::Ok,
        log_warn => StatusLevel::Warning,
        log_info => StatusLevel::Info,
    }

    #[cfg(feature = "alloc")]
    pub fn try_run<O>(&mut self, tryresult: Result<O, Box<dyn core::error::Error>>) {
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn try_run<O, E: core::fmt::Debug>(&mut self, tryresult: Result<O, E>) {
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    impl_try_get!(core::fmt::Debug, owned);
}

struct TimeFormatter<'a, T: TimeProvider>(&'a T);

impl<'a, T: TimeProvider> core::fmt::Display for TimeFormatter<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.write(f)
    }
}

#[cfg(feature = "ufmt")]
pub struct UDebugStr<'a>(pub &'a str);

#[cfg(feature = "ufmt")]
impl<'a> Debug for UDebugStr<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

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
            uwrite!(f, "{}ns", nanos as u32)
        } else if micros < 1_000 {
            uwrite!(f, "{}μs", micros as u32)
        } else if millis < 1_000 {
            uwrite!(f, "{}ms", millis as u32)
        } else if secs < 60 {
            let remaining_millis = (millis % 1000) as u32;
            uwrite!(f, "{}.{}s", secs, remaining_millis)
        } else if secs < 3600 {
            let mins = secs / 60;
            let remaining_secs = secs % 60;
            uwrite!(f, "{}:{}min", mins, remaining_secs)
        } else if secs < 86400 {
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let remaining_secs = secs % 60;
            uwrite!(f, "{}:{}:{}", hours, mins, remaining_secs)
        } else {
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

#[cfg(feature = "std")]
#[cfg(feature = "ufmt")]
struct StdWriter<'a>(&'a mut dyn std::io::Write);

#[cfg(feature = "std")]
#[cfg(feature = "ufmt")]
impl<'a> uWrite for StdWriter<'a> {
    type Error = ();
    fn write_str(&mut self, s: &str) -> Result<(), ()> {
        self.0.write_all(s.as_bytes()).map_err(|_| ())
    }
}

#[cfg(feature = "std")]
#[cfg(feature = "ufmt")]
impl UStorageProvider for () {
    fn write_data(&mut self, d: impl uDebug) {
        use std::io::{self};
        let mut stdout = io::stdout();
        let mut writer = StdWriter(&mut stdout);
        let _ = d.fmt(&mut ufmt::Formatter::new(&mut writer));
    }
}

#[cfg(feature = "ufmt")]
pub struct ULogger<T: TimeProvider, S: UStorageProvider>(pub T, pub S);

#[cfg(feature = "ufmt")]
#[derive(Clone)]
pub struct MultiULogger<T: TimeProvider + Clone, S: UStorageProvider + Clone>(pub T, pub S);

#[cfg(feature = "ufmt")]
impl<T: TimeProvider, S: UStorageProvider> ULogger<T, S> {
    pub fn log(&mut self, level: StatusLevel, args: impl uDebug) {
        let timestamp = self.0.elapsed();
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
        self.1.write_data(UDebugStr(args));
        self.1.write_data(UDebugStr(RESET));
        self.1.write_data(UDebugStr("\n"));
    }

    impl_log_methods_ufmt! {
        log_err => StatusLevel::Error,
        log_ok => StatusLevel::Ok,
        log_warn => StatusLevel::Warning,
        log_info => StatusLevel::Info,
    }

    #[cfg(feature = "alloc")]
    #[cfg(not(feature = "ufmt"))]
    pub fn try_run<O>(&mut self, tryresult: Result<O, Box<dyn core::error::Error>>) {
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    #[cfg(feature = "alloc")]
    #[cfg(feature = "ufmt")]
    pub fn try_run<O>(&mut self, tryresult: Result<O, Box<dyn core::error::Error>>) {
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, UDebugStr(&err.to_string()));
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn try_run<O, E: ufmt::uDebug>(&mut self, tryresult: Result<O, E>) {
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, err);
        }
    }

    impl_try_get!(ufmt::uDebug, owned);
}

#[cfg(feature = "ufmt")]
impl<T: TimeProvider + Clone, S: UStorageProvider + Clone> MultiULogger<T, S>
where
    Self: Clone,
{
    pub fn log(&mut self, level: StatusLevel, args: impl uDebug) {
        let timestamp = self.0.elapsed();
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
        self.1.write_data(UDebugStr(args));
        self.1.write_data(UDebugStr(RESET));
        self.1.write_data(UDebugStr("\n"));
    }

    impl_log_methods_ufmt! {
        log_err => StatusLevel::Error,
        log_ok => StatusLevel::Ok,
        log_warn => StatusLevel::Warning,
        log_info => StatusLevel::Info,
    }

    #[cfg(feature = "alloc")]
    pub fn try_run<O>(&mut self, tryresult: Result<O, Box<dyn core::error::Error>>) {
        if let Err(err) = tryresult {
            self.log(StatusLevel::Error, UDebugStr(&err.to_string()));
        }
    }

    impl_try_get!(ufmt::uDebug, cloned);
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! black_box_cand {
    () => {
        ::std::panic::set_hook(Box::new(|info| {
            let mut logger = $crate::Logger(::std::time::Instant::now(), ());
            let payload = if let Some(s) = info.payload().downcast_ref::<&'static str>() {
                *s
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.as_str()
            } else {
                "unknown panic payload"
            };
            let (before, after) = if let Some(pos) = payload.find(": ") {
                (&payload[0..pos + 2], &payload[pos + 2..])
            } else {
                ("", payload)
            };
            let message = if let Some(location) = info.location() {
                format!(
                    "\x1b[0mpanicked at {}:{}:{}:\n\x1b[0m{}\x1b[31m{}\x1b[0m",
                    location.file(),
                    location.line(),
                    location.column(),
                    before,
                    after
                )
            } else {
                format!(
                    "\x1b[0mpanicked at unknown location:\n\x1b[0m{}\x1b[31m{}\x1b[0m",
                    before, after
                )
            };
            logger.logdisp($crate::StatusLevel::Critical, &message);
        }));
    };

    ($logger_expr:expr) => {
        ::std::panic::set_hook(Box::new(|info| {
            let mut logger = $logger_expr;
            let payload = if let Some(s) = info.payload().downcast_ref::<&'static str>() {
                *s
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.as_str()
            } else {
                "unknown panic payload"
            };
            let (before, after) = if let Some(pos) = payload.find(": ") {
                (&payload[0..pos + 2], &payload[pos + 2..])
            } else {
                ("", payload)
            };
            let message = if let Some(location) = info.location() {
                format!(
                    "\x1b[0mpanicked at {}:{}:{}:\n\x1b[0m{}\x1b[31m{}\x1b[0m",
                    location.file(),
                    location.line(),
                    location.column(),
                    before,
                    after
                )
            } else {
                format!(
                    "\x1b[0mpanicked at unknown location:\n\x1b[0m{}\x1b[31m{}\x1b[0m",
                    before, after
                )
            };
            logger.logdisp($crate::StatusLevel::Critical, &message);
        }))
    };
}

#[cfg(feature = "std")]
#[macro_export]
macro_rules! black_box_cand_global {
    ($logger:expr) => {
        let mut logger = $logger;
        ::std::panic::set_hook(Box::new(move |info| {
            let payload = if let Some(s) = info.payload().downcast_ref::<&'static str>() {
                *s
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                s.as_str()
            } else {
                "unknown panic payload"
            };
            let (before, after) = if let Some(pos) = payload.find(": ") {
                (&payload[0..pos + 2], &payload[pos + 2..])
            } else {
                ("", payload)
            };
            let message = if let Some(location) = info.location() {
                format!(
                    "\x1b[0mpanicked at {}:{}:{}:\n\x1b[0m{}\x1b[31m{}\x1b[0m",
                    location.file(),
                    location.line(),
                    location.column(),
                    before,
                    after
                )
            } else {
                format!(
                    "\x1b[0mpanicked at unknown location:\n\x1b[0m{}\x1b[31m{}\x1b[0m",
                    before, after
                )
            };
            if let Ok(mut guard) = logger.lock() {
                guard.logdisp(cand::StatusLevel::Critical, &message);
            }
        }));
    };
}
