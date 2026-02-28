//! This module contains raw implementation of proc macros with `proc_macro2`.

mod err;
mod from_arg;
mod parsef;

pub use self::{err::*, from_arg::*, parsef::*};
