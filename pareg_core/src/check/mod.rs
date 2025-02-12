//! This module provides types that allow checking for values even if the given
//! type is parsed.

mod check_ref;
mod in_range;
mod in_range_i;

pub use self::{check_ref::*, in_range::*, in_range_i::*};
