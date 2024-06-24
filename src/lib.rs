//! # pareg
//! Helpful utilities for parsing command line arguments.
//!
//! Currently this crate doesn't contain any magic derive macro that would generate
//! code that parses your arguments. There are many ways that arguments may be used
//! and so there are only helper functions, traits and structures that help with
//! the parsing in a more manual way. (But there may be such derive macro in
//! the future.)
//!
//! ## Example usage
//! ```rust
//! use pareg::{Result, ArgIterator, ByRef, key_val_arg, FromArg};
//!
//! // You can define enums, and have them automaticaly derive FromArg where each
//! // enum variant will be parsed from case insensitive strings of the same name
//! // (e.g. `"Auto"` will parse into `Auto`, `"always"` into `Always`, `"NEVER"`
//! // into `Never`)
//! #[derive(FromArg)]
//! enum ColorMode {
//!     Auto,
//!     Always,
//!     Never,
//! }
//!
//! // create your struct that will hold the arguments
//! struct Args<'a> {
//!     name: &'a str,
//!     count: usize,
//!     colors: ColorMode,
//! }
//!
//! impl<'a> Args<'a> {
//!     // create function that takes the arguments as ArgIterator
//!     pub fn parse<I>(mut args: ArgIterator<'a, I>) -> Result<Self>
//!     where
//!         I: Iterator,
//!         I::Item: ByRef<&'a str>,
//!     {
//!         // initialize with default values
//!         let mut res = Args {
//!             name: "pareg",
//!             count: 1,
//!             colors: ColorMode::Auto,
//!         };
//!
//!         while let Some(arg) = args.next() {
//!             match arg {
//!                 // when there is the argument `count`, parse the next value
//!                 "-c" | "--count" => res.count = args.next_arg()?,
//!                 a if a.starts_with("--color=") => {
//!                     res.colors = args.cur_key_val::<&str, _>('=')?.1;
//!                 }
//!                 // if the argument is unknown, just set it as name
//!                 _ => res.name = arg,
//!             }
//!         }
//!
//!         Ok(res)
//!     }
//! }
//!
//! // Now you can call your parse method:
//! fn main() -> Result<()> {
//!     // you need to collect the arguments first so that you can refer to
//!     // them by reference
//!     let args: Vec<_> = std::env::args().collect();
//!     // just pass in any iterator of string reference that has lifetime
//!     let args = Args::parse(args.iter().into())?;
//!
//!     // Now you can use your arguments:
//!     for _ in 0..args.count {
//!         println!("Hello {}!", args.name);
//!     }
//!     Ok(())
//! }
//! ```

pub use pareg_core::*;
pub use pareg_proc::FromArg;

#[cfg(test)]
mod tests {
    use crate::{self as pareg, FromArg, ArgIterator, Result};

    #[derive(FromArg, PartialEq, Debug)]
    enum ColorMode {
        Always,
        Never,
        Auto,
    }

    #[test]
    fn arg_iterator() -> Result<()> {
        let args = ["hello", "10", "0.25", "always"];
        let mut args: ArgIterator<_> = args.iter().into();

        assert_eq!("hello", args.next_arg::<&str>()?);
        assert_eq!(10, args.next_arg::<usize>()?);
        assert_eq!(0.25, args.next_arg::<f64>()?);
        assert_eq!(ColorMode::Always, args.next_arg::<ColorMode>()?);

        Ok(())
    }
}
