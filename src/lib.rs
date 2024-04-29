// Copyright 2021-2024 Graydon Hoare <graydon@pobox.com>
// Licensed under ASL2 or MIT

//!
//! This is a tiny crate that provides a tiny error-wrapper struct
//! `BacktraceError` with only two features:
//!
//!   - Captures a backtrace on `From`-conversion from its wrapped type (if
//!     `RUST_BACKTRACE` is on etc.)
//!   - Pretty-prints that backtrace in its `Display` implementation.
//!
//! It also includes an extension trait `ResultExt` that you can `use` to give
//! you `.unwrap_or_backtrace` and `.expect_or_backtrace` methods on any
//! `Result<T, BacktraceError<E>>`. These methods do do the same as `unwrap`
//! or `expect` on `Result` except they pretty-print the backtrace on `Err`,
//! before panicking.
//!
//! Finally, it provides a _dynamic_ variant in case you want to type-erase the
//! error type, `DynBacktraceError`. This works the same as `BacktraceError<E>`
//! but wraps a `Box<dyn Error + Send + Sync + 'static>` instead of requiring a
//! specific error type `E`, so is therefore potentially more expensive but also
//! more flexible and usable as an "any error" catchall type since it has an
//! `impl<E:Error + Send + Sync + 'static> From<E>` conversion.
//!
//! # Example
//!
//! Usage is straightforward: put some existing error type in it. No macros!
//!
//! ```should_panic
//! use backtrace_error::{BacktraceError,ResultExt};
//! use std::{io,fs};
//!
//! type IOError = BacktraceError<io::Error>;
//!
//! fn open_file() -> Result<fs::File, IOError> {
//!    Ok(fs::File::open("/does-not-exist.nope")?)
//! }
//!
//! fn do_stuff() -> Result<fs::File, IOError>
//! {
//!     open_file()
//! }
//!
//! fn main()
//! {
//!     // This will panic but first print a backtrace of
//!     // the error site, then a backtrace of the panic site.
//!     let file = do_stuff().unwrap_or_backtrace();
//! }
//! ```
//!
//! or dynamically:
//!
//! ```should_panic
//! use backtrace_error::{DynBacktraceError,ResultExt};
//! use std::{io,fs};
//!
//! type AppErr = DynBacktraceError;
//!
//! fn open_file() -> Result<fs::File, AppErr> {
//!    Ok(fs::File::open("/does-not-exist.nope")?)
//! }
//!
//! fn parse_number() -> Result<i32, AppErr> {
//!    Ok(i32::from_str_radix("not-a-number", 10)?)
//! }
//!
//! fn do_stuff() -> Result<(), AppErr>
//! {
//!     open_file()?;
//!     parse_number()?;
//!     Ok(())
//! }
//!
//! fn main()
//! {
//!     // This will panic but first print a backtrace of
//!     // the error site, then a backtrace of the panic site.
//!     do_stuff().unwrap_or_backtrace();
//! }
//! ```
//!
//! I am very sorry for having written Yet Another Rust Error Crate but
//! strangely everything I looked at either doesn't capture backtraces, doesn't
//! print them, only debug-prints them on a failed unwrap (which is illegible),
//! provides a pile of features I don't want through expensive macros, or some
//! combination thereof. I don't need any of that, I just want to capture
//! backtraces for errors when they occur, and print them out sometime later.
//!
//! I figured maybe someone out there has the same need, so am publishing it.

use std::{
    backtrace::Backtrace,
    error::Error,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

pub struct BacktraceError<E: Error> {
    pub inner: E,
    pub backtrace: Box<Backtrace>,
}

impl<E: Error> Display for BacktraceError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Initial error: {:}", self.inner)?;
        writeln!(f, "Error context:")?;
        writeln!(f, "{:}", self.backtrace)
    }
}

impl<E: Error> Debug for BacktraceError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl<E: Error + 'static> Error for BacktraceError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.inner)
    }
}

// Someday we'll also support the "Provider" API, but not today
// since it is not stable and I don't want to bother tracking
// its stability.
/*
impl<E:Error + 'static> std::any::Provider for BacktraceError<E> {
    fn provide<'a>(&'a self, demand: &mut std::any::Demand<'a>) {
        demand.provide_ref::<Backtrace>(self.backtrace)
        .provide_value::<Backtrace>(|| self.backtrace)
    }
}
*/

impl<E: Error + 'static> From<E> for BacktraceError<E> {
    fn from(inner: E) -> Self {
        let backtrace = Box::new(Backtrace::capture());
        Self { inner, backtrace }
    }
}

pub trait ResultExt: Sized {
    type T;
    fn unwrap_or_backtrace(self) -> Self::T {
        self.expect_or_backtrace("ResultExt::unwrap_or_backtrace found Err")
    }
    fn expect_or_backtrace(self, msg: &str) -> Self::T;
}

impl<T, E: Error> ResultExt for Result<T, BacktraceError<E>> {
    type T = T;
    fn expect_or_backtrace(self, msg: &str) -> T {
        match self {
            Ok(ok) => ok,
            Err(bterr) => {
                eprintln!("{}", msg);
                eprintln!("");
                eprintln!("{:}", bterr);
                panic!("{}", msg);
            }
        }
    }
}

pub struct DynBacktraceError {
    inner: Box<dyn Error + Send + Sync + 'static>,
    backtrace: Box<Backtrace>,
}

impl<E: Error + Send + Sync + 'static> From<E> for DynBacktraceError {
    fn from(inner: E) -> Self {
        let backtrace = Box::new(Backtrace::capture());
        Self {
            inner: Box::new(inner),
            backtrace,
        }
    }
}

impl Deref for DynBacktraceError {
    type Target = dyn Error + Send + Sync + 'static;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl DerefMut for DynBacktraceError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

impl Display for DynBacktraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Initial error: {:}", self.inner)?;
        writeln!(f, "Error context:")?;
        writeln!(f, "{:}", self.backtrace)
    }
}

impl Debug for DynBacktraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

impl ResultExt for Result<(), DynBacktraceError> {
    type T = ();
    fn expect_or_backtrace(self, msg: &str) -> () {
        match self {
            Ok(()) => (),
            Err(bterr) => {
                eprintln!("{}", msg);
                eprintln!("");
                eprintln!("{:}", bterr);
                panic!("{}", msg);
            }
        }
    }
}
