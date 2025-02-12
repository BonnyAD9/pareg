use std::{fmt::Display, ops::Range};

use crate::{ParseF, Result};

pub struct InRange<'a, T: ParseF + PartialOrd + Display>(
    pub &'a mut T,
    pub Range<T>,
);

impl<T: ParseF + PartialOrd + Display> ParseF for InRange<'_, T> {
    fn set_from_read(
        &mut self,
        r: &mut crate::Reader,
    ) -> Result<Option<crate::ArgError>> {
        let start_pos = r.pos().unwrap_or_default();
        match self.0.set_from_read(r) {
            Ok(res) => {
                if self.1.contains(self.0) {
                    Ok(res)
                } else {
                    r.err_value(format!(
                        "Value must be in range from `{}` to `{}`",
                        self.1.start, self.1.end
                    ))
                    .span_start(start_pos)
                    .main_msg(format!(
                        "Invalid value `{}`. \
                        Value must be in range from `{}` to `{}`.",
                        self.0, self.1.start, self.1.end,
                    ))
                    .err()
                }
            }
            e => e,
        }
    }
}
