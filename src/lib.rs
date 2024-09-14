//! # pareg
//! Helpful utilities for parsing command line arguments.
//!
//! The aim of this crate is not to have some magic derive macro that will do
//! all of the parsing for you. Instead pareg will let you choose exactly how
//! to parse the arguments, but it will help as much as possible.
//!
//! Pareg also comes with user friendly errors so that you don't have to worry
//! about writing the error messages while parsing the arguments. For example
//! running the program below like this:
//! ```sh
//! my-program --color=no
//! ```
//! will output the following error message:
//! ```txt
//! argument error: Unknown option `no`.
//! --> arg1:8..10
//!  |
//!  $ my-program --color=no
//!  |                    ^^ Unknown option.
//! hint: Valid options are: `auto`, `always`, `never`.
//! ```
//!
//! ## Example usage
//! ```rust
//! use std::process::ExitCode;
//!
//! use pareg::{Result, Pareg, FromArg, starts_any};
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
//! struct Args {
//!     name: String,
//!     count: usize,
//!     colors: ColorMode,
//! }
//!
//! impl Args {
//!     // create function that takes the arguments as ArgIterator
//!     pub fn parse(mut args: Pareg) -> Result<Self>
//!     {
//!         // initialize with default values
//!         let mut res = Args {
//!             name: "pareg".to_string(),
//!             count: 1,
//!             colors: ColorMode::Auto,
//!         };
//!
//!         while let Some(arg) = args.next() {
//!             match arg {
//!                 // when there is the argument `count`, parse the next value
//!                 "-c" | "--count" => res.count = args.next_arg()?,
//!                 // if the argument starts with either `--color` or
//!                 // `--colour`, parse its value.
//!                 a if starts_any!(a, "--color=", "--colour=") => {
//!                     res.colors = args.cur_val('=')?;
//!                 }
//!                 // it seems that this is flag, but it is not recognized
//!                 a if a.starts_with('-') => {
//!                     Err(args.err_unknown_argument())?
//!                 },
//!                 // if the argument is unknown, just set it as name
//!                 _ => res.name = arg.to_string(),
//!             }
//!         }
//!
//!         Ok(res)
//!     }
//! }
//!
//! // Now you can call your parse method:
//! fn start() -> Result<()> {
//!     // just pass in any iterator of string reference that has lifetime
//!     let args = Args::parse(Pareg::args())?;
//!
//!     // Now you can use your arguments:
//!     for _ in 0..args.count {
//!         println!("Hello {}!", args.name);
//!     }
//!     Ok(())
//! }
//!
//! fn main() -> ExitCode {
//!     match start() {
//!         Ok(_) => ExitCode::SUCCESS,
//!         Err(e) => {
//!             eprint!("{e}");
//!             ExitCode::FAILURE
//!         }
//!     }
//! }
//! ```

pub use pareg_core::*;
pub use pareg_proc::FromArg;

#[cfg(test)]
mod tests {
    use crate::{self as pareg, FromArg, Pareg, Result};

    #[derive(FromArg, PartialEq, Debug)]
    enum ColorMode {
        Always,
        Never,
        Auto,
    }

    #[test]
    fn arg_iterator() -> Result<()> {
        let args = ["hello", "10", "0.25", "always"];
        let mut args =
            Pareg::new(args.iter().map(|a| a.to_string()).collect());

        assert_eq!("hello", args.next_arg::<&str>()?);
        assert_eq!(10, args.next_arg::<usize>()?);
        assert_eq!(0.25, args.next_arg::<f64>()?);
        assert_eq!(ColorMode::Always, args.next_arg::<ColorMode>()?);

        Ok(())
    }

    #[test]
    fn has_any_key() {
        use pareg_core::has_any_key;

        let s = "ahoj";
        let sep = ':';
        assert!(has_any_key!("hello", '=', "hello", s));
        assert!(has_any_key!("hello=", '=', "hello", s));
        assert!(has_any_key!("ahoj:lol", sep, "hello", s));
        assert!(!has_any_key!("greeting=ahoj", '=', "greet", s));
    }
}
