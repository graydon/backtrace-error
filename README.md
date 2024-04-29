# backtrace-error


This is a tiny crate that provides a tiny error-wrapper struct
`BacktraceError` with only two features:

  - Captures a backtrace on `From`-conversion from its wrapped type (if
    `RUST_BACKTRACE` is on etc.)
  - Pretty-prints that backtrace in its `Display` implementation.

It also includes an extension trait `ResultExt` that you can `use` to give
you `.unwrap_or_backtrace` and `.expect_or_backtrace` methods on any
`Result<T, BacktraceError<E>>`. These methods do do the same as `unwrap`
or `expect` on `Result` except they pretty-print the backtrace on `Err`,
before panicking.

Finally, it provides a _dynamic_ variant in case you want to type-erase the
error type, `DynBacktraceError`. This works the same as `BacktraceError<E>`
but wraps a `Box<dyn Error + Send + Sync + 'static>` instead of requiring a
specific error type `E`, so is therefore potentially more expensive but also
more flexible and usable as an "any error" catchall type since it has an
`impl<E:Error + Send + Sync + 'static> From<E>` conversion.

## Example

Usage is straightforward: put some existing error type in it. No macros!

```rust
use backtrace_error::{BacktraceError,ResultExt};
use std::{io,fs};

type IOError = BacktraceError<io::Error>;

fn open_file() -> Result<fs::File, IOError> {
   Ok(fs::File::open("/does-not-exist.nope")?)
}

fn do_stuff() -> Result<fs::File, IOError>
{
    open_file()
}

fn main()
{
    // This will panic but first print a backtrace of
    // the error site, then a backtrace of the panic site.
    let file = do_stuff().unwrap_or_backtrace();
}
```

or dynamically:

```rust
use backtrace_error::{DynBacktraceError,ResultExt};
use std::{io,fs};

type AppErr = DynBacktraceError;

fn open_file() -> Result<fs::File, AppErr> {
   Ok(fs::File::open("/does-not-exist.nope")?)
}

fn parse_number() -> Result<i32, AppErr> {
   Ok(i32::from_str_radix("not-a-number", 10)?)
}

fn do_stuff() -> Result<(), AppErr>
{
    open_file()?;
    parse_number()?;
    Ok(())
}

fn main()
{
    // This will panic but first print a backtrace of
    // the error site, then a backtrace of the panic site.
    do_stuff().unwrap_or_backtrace();
}
```

I am very sorry for having written Yet Another Rust Error Crate but
strangely everything I looked at either doesn't capture backtraces, doesn't
print them, only debug-prints them on a failed unwrap (which is illegible),
provides a pile of features I don't want through expensive macros, or some
combination thereof. I don't need any of that, I just want to capture
backtraces for errors when they occur, and print them out sometime later.

I figured maybe someone out there has the same need, so am publishing it.

License: MIT OR Apache-2.0
