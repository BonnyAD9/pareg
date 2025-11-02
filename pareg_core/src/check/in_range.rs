use std::{
    fmt::Display,
    ops::{Bound, RangeBounds},
};

use crate::{Result, SetFromRead, reader::ReadFmt};

/// Wraps [`SetFromRead`] implementation of type, so that it chechs that its
/// value is in the given range.
pub struct InRange<
    'a,
    T: SetFromRead + PartialOrd + Display,
    R: RangeBounds<T>,
>(pub &'a mut T, pub R);

impl<T: SetFromRead + PartialOrd + Display, R: RangeBounds<T>> SetFromRead
    for InRange<'_, T, R>
{
    fn set_from_read<'a>(
        &mut self,
        r: &mut crate::Reader,
        fmt: &'a ReadFmt<'a>,
    ) -> Result<Option<crate::ArgError>> {
        let start_pos = r.pos();
        match self.0.set_from_read(r, fmt) {
            Ok(res) => {
                if self.1.contains(self.0) {
                    Ok(res)
                } else {
                    let range = print_range_bounds(&self.1);
                    r.err_value(format!("Value must be {range}."))
                        .span_start(start_pos)
                        .long_msg(format!(
                            "Invalid value `{}`. Value must be {range}.",
                            self.0,
                        ))
                        .err()
                }
            }
            e => e,
        }
    }
}

fn print_range_bounds<T: Display>(range: &impl RangeBounds<T>) -> String {
    match (range.start_bound(), range.end_bound()) {
        (Bound::Excluded(s), Bound::Excluded(e)) => {
            format!("in exclusive range from `{s}` to `{e}`")
        }
        (Bound::Excluded(s), Bound::Included(e)) => {
            format!("in range from `{s}` (exclusive) to `{e}` (inclusive)")
        }
        (Bound::Excluded(s), Bound::Unbounded) => format!("larger than `{s}`"),
        (Bound::Included(s), Bound::Excluded(e)) => {
            format!("in range from `{s}` to `{e}`")
        }
        (Bound::Included(s), Bound::Included(e)) => {
            format!("in inclusive range from `{s}` to `{e}`")
        }
        (Bound::Included(s), Bound::Unbounded) => {
            format!("larger or equal to `{s}`")
        }
        (Bound::Unbounded, Bound::Excluded(e)) => {
            format!("smaller than `{e}`")
        }
        (Bound::Unbounded, Bound::Included(e)) => {
            format!("smaller or equal to `{e}`")
        }
        (Bound::Unbounded, Bound::Unbounded) => {
            "unbounded".to_string() // shouldn't happen in errors.
        }
    }
}
