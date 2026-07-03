use crate::{ArgInto, ParegRef, Result};

/// Macro for types that can parse command line arguments.
pub trait FromArgs<'a>: Sized {
    /// Parse the given command line arguments into this type.
    fn parse_args<S: ArgInto<'a>>(args: &mut ParegRef<'a, S>) -> Result<Self>;
}
