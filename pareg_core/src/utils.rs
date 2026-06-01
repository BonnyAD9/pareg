use std::{
    ops::{Bound, RangeBounds},
    range::Range,
};

pub fn get_range(r: impl RangeBounds<usize>) -> Range<usize> {
    let start = match r.start_bound() {
        Bound::Included(v) => *v,
        Bound::Excluded(v) => v.saturating_add(1),
        Bound::Unbounded => 0,
    };

    let end = match r.end_bound() {
        Bound::Included(v) => v.saturating_sub(1),
        Bound::Excluded(v) => *v,
        Bound::Unbounded => usize::MAX,
    };

    Range { start, end }
}
