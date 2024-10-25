mod arg_err_ctx;
mod arg_error;
mod color_mode;

pub use self::{arg_err_ctx::*, arg_error::*, color_mode::*};

/// Pareg result type. It is [`std::result::Result<T, ArgError<'a>>`]
pub type Result<T> = std::result::Result<T, ArgError>;
