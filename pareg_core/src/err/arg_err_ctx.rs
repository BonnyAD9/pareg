use std::{borrow::Cow, collections::VecDeque, fmt::Display, ops::Range};

/// Information about error in command line arguments. Implements [`Display`]
/// with user friendly error messages.
#[derive(Debug)]
pub struct ArgErrCtx {
    /// All command line arguments.
    pub args: Vec<String>,
    /// Index of the errornous argument in [`Self::args`]
    pub error_idx: usize,
    /// Range within the argument that is invalid.
    pub error_span: Range<usize>,
    /// Simple error message describing the kind of the problem.
    pub message: Cow<'static, str>,
    /// More descriptive message describing the problem in detail.
    pub long_message: Option<Cow<'static, str>>,
    /// Hint about how to fix the error.
    pub hint: Option<Cow<'static, str>>,
}

impl ArgErrCtx {
    /// Creates simple error with just message and the errornous argument.
    pub fn from_msg(message: Cow<'static, str>, arg: String) -> Self {
        Self {
            error_span: 0..arg.len(),
            args: vec![arg],
            error_idx: 0,
            long_message: Some(message.clone()),
            message,
            hint: None,
        }
    }

    /// Moves the span in the error message by `cnt` and changes the
    /// errornous argument to `new_arg`.
    pub fn shift_span(mut self, cnt: usize, new_arg: String) -> Self {
        self.error_span.start += cnt;
        self.error_span.end += cnt;
        self.args[self.error_idx] = new_arg;
        self
    }

    /// Add arguments to the error so that it may have better error message.
    /// Mostly useful internaly in pareg.
    pub fn add_args(mut self, args: Vec<String>, idx: usize) -> Self {
        if self.args[self.error_idx].len() != args[idx].len() {
            if let Some(shift) = args[idx].find(&self.args[self.error_idx]) {
                self.error_span.start += shift;
                self.error_span.end += shift;
            }
        }
        self.args = args;
        self.error_idx = idx;
        self
    }

    /// Adds hint to the error message.
    pub fn hint(mut self, hint: impl Into<Cow<'static, str>>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl Display for ArgErrCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const MAX_WIDTH: usize = 80;
        const WIDTH: usize = MAX_WIDTH - 11;

        let args = vec!["".to_string()];
        let args = if self.args.is_empty() {
            &args
        } else {
            &self.args
        };
        let error_idx = self.error_idx.clamp(0, args.len() - 1);

        let lengths: Vec<_> = args.iter().map(|a| a.chars().count()).collect();

        let long_message = self.long_message.as_ref().unwrap_or(&self.message);

        writeln!(f, "argument error: {long_message}")?;
        writeln!(
            f,
            "--> arg{}:{}..{}",
            error_idx, self.error_span.start, self.error_span.end
        )?;
        writeln!(f, " |")?;

        let mut to_print = VecDeque::new();
        to_print.push_back(error_idx);
        let mut width = lengths[error_idx];
        let mut start_idx = error_idx;
        let mut end_idx = error_idx;

        loop {
            let mut start_end = false;
            if start_idx > 0 {
                start_idx -= 1;
                let ad_len = args[start_idx].len() + 1;
                if width + ad_len > WIDTH {
                    start_idx += 1;
                    break;
                }
                width += ad_len;
                to_print.push_front(start_idx);
            } else {
                start_end = true;
            }

            if end_idx + 1 < args.len() {
                end_idx += 1;
                let ad_len = args[end_idx].len() + 1;
                if width + ad_len > WIDTH {
                    end_idx -= 1;
                    break;
                }
                width += ad_len;
                to_print.push_back(end_idx);
            } else if start_end {
                break;
            }
        }

        let mut err_pos = if start_idx == 0 {
            write!(f, " $ ")?;
            3
        } else {
            write!(f, " $ ... ")?;
            7
        };

        for &i in &to_print {
            match i {
                i if i < error_idx => {
                    write!(f, "{} ", args[i])?;
                    err_pos += lengths[i] + 1;
                }
                i if i == error_idx => {
                    write!(f, "{}", args[i])?;
                    let arg = &args[i];
                    err_pos += arg[..self.error_span.start.min(arg.len())]
                        .chars()
                        .count();
                }
                i => {
                    write!(f, " {}", args[i])?;
                }
            }
        }

        if end_idx != args.len() - 1 {
            writeln!(f, " ...")?;
        } else {
            writeln!(f)?;
        }

        err_pos -= 2;
        let err_len = self.error_span.len();
        writeln!(
            f,
            " |{: >err_pos$}{:^>err_len$} {}",
            ' ', '^', self.message
        )?;
        let Some(hint) = &self.hint else {
            return Ok(());
        };

        writeln!(f, "hint: {hint}")
    }
}