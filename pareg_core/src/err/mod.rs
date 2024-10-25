mod arg_err_ctx;
mod arg_error;

pub use self::{arg_err_ctx::*, arg_error::*};

/// Pareg result type. It is [`std::result::Result<T, ArgError<'a>>`]
pub type Result<T> = std::result::Result<T, ArgError>;
