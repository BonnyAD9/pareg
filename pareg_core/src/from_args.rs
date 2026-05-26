use crate::{ParegRef, Result};

/// Macro for types that can parse command line arguments.
pub trait FromArgs<'a, S: AsRef<str> = String>: Sized {
    /// Parse the given command line arguments into this type.
    fn parse_args(args: &mut ParegRef<'a, S>) -> Result<Self>;
}
