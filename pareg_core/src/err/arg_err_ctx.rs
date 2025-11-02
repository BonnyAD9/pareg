use std::{
    borrow::Cow, cell::LazyCell, collections::VecDeque, fmt::Display,
    ops::Range,
};

use termal::{writemc, writemcln};

use crate::ArgErrKind;

use super::ColorMode;

#[cfg(not(feature = "no-anounce"))]
pub const DEFAULT_ANOUNCE: bool = true;

#[cfg(feature = "no-anounce")]
pub const DEFAULT_ANOUNCE: bool = false;

/// Information about error in command line arguments. Implements [`Display`]
/// with user friendly error messages.
#[derive(Debug)]
pub struct ArgErrCtx {
    pub kind: ArgErrKind,
    /// All command line arguments.
    pub args: Vec<String>,
    /// Index of the errornous argument in [`Self::args`]
    pub error_idx: usize,
    /// Range within the argument that is invalid.
    pub error_span: Range<usize>,
    /// INLINE. Simple error message describing the kind of the problem.
    pub inline_msg: Option<Cow<'static, str>>,
    /// More descriptive message describing the problem in detail.
    pub long_msg: Option<Cow<'static, str>>,
    /// Hint about how to fix the error.
    pub hint: Option<Cow<'static, str>>,
    /// Determines when color should be used.
    pub color: ColorMode,
    /// Determines whether `error:` is prefixed to the message.
    pub anounce: bool,
}

impl ArgErrCtx {
    pub fn new(kind: impl Into<ArgErrKind>) -> Self {
        Self {
            kind: kind.into(),
            args: vec![],
            error_idx: 0,
            error_span: 0..0,
            inline_msg: None,
            long_msg: None,
            hint: None,
            color: ColorMode::default(),
            anounce: true,
        }
    }

    pub fn from_inner<E: Display>(
        kind: ArgErrKind,
        e: E,
        arg: String,
    ) -> Self {
        Self::from_msg(kind, e.to_string(), arg)
    }

    /// Creates simple error with just message and the errornous argument.
    /// (this is the INLINE message)
    pub fn from_msg(
        kind: ArgErrKind,
        message: impl Into<Cow<'static, str>>,
        arg: String,
    ) -> Self {
        Self {
            error_span: 0..arg.len(),
            args: vec![arg],
            inline_msg: Some(message.into()),
            ..Self::new(kind)
        }
    }

    /// Moves the span in the error message by `cnt` and changes the
    /// errornous argument to `new_arg`.
    pub fn shift_span(&mut self, cnt: usize, new_arg: String) {
        self.error_span.start += cnt;
        self.error_span.end += cnt;
        self.args[self.error_idx] = new_arg;
    }

    /// Sets new argument. If the original argument is substring of this,
    /// span will be adjusted.
    pub fn part_of(&mut self, arg: String) {
        if self.args[self.error_idx].len() == arg.len() {
            self.error_span = 0..arg.len();
            self.args[self.error_idx] = arg;
            return;
        }
        if let Some(shift) = arg.find(&self.args[self.error_idx]) {
            self.error_span.start += shift;
            self.error_span.end += shift;
        }
        self.args[self.error_idx] = arg;
    }

    /// Add arguments to the error so that it may have better error message.
    /// Mostly useful internaly in pareg.
    pub fn add_args(&mut self, args: Vec<String>, idx: usize) {
        if self.args[self.error_idx].len() != args[idx].len()
            && let Some(shift) = args[idx].find(&self.args[self.error_idx])
        {
            self.error_span.start += shift;
            self.error_span.end += shift;
        }
        self.args = args;
        self.error_idx = idx;
    }

    /// Adds hint to the error message.
    pub fn hint(&mut self, hint: impl Into<Cow<'static, str>>) {
        self.hint = Some(hint.into());
    }

    /// Adds span to the error message.
    pub fn spanned(&mut self, span: Range<usize>) {
        self.error_span = span;
    }

    /// Sets the start value of the span
    pub fn span_start(&mut self, start: usize) {
        self.error_span.start = start.min(self.error_span.end);
    }

    /// Sets the short message that is inlined with the code.
    pub fn inline_msg(&mut self, msg: impl Into<Cow<'static, str>>) {
        self.inline_msg = Some(msg.into());
    }

    /// Sets the primary (non inline) message.
    pub fn long_msg(&mut self, msg: impl Into<Cow<'static, str>>) {
        self.long_msg = Some(msg.into());
    }

    /// Set the color mode.
    pub fn color_mode(&mut self, mode: ColorMode) {
        self.color = mode;
    }

    /// Disable color.
    pub fn no_color(&mut self) {
        self.color_mode(ColorMode::Never);
    }

    /// Changes the current argument to be postfix of this whole argument.
    pub fn postfix_of(&mut self, arg: String) {
        let al = self.args[self.error_idx].len();
        match al.cmp(&arg.len()) {
            std::cmp::Ordering::Less => self.shift_span(arg.len() - al, arg),
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                let d = al - arg.len();
                self.error_span.start += d;
                self.error_span.end += d;
            }
        }
    }
}

impl Display for ArgErrCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const MAX_WIDTH: usize = 80;
        const WIDTH: usize = MAX_WIDTH - 11;
        let mut color = self.color.use_color();
        if f.sign_minus() {
            color = false;
        }
        if f.sign_plus() {
            color = true;
        }

        let kind_msg = LazyCell::new(|| self.kind.to_string());
        let long_message = self
            .long_msg
            .as_deref()
            .or(self.inline_msg.as_deref())
            .unwrap_or_else(|| &kind_msg);

        let args = if self.args.is_empty() {
            if self.anounce {
                writemcln!(
                    f,
                    color,
                    "{'r}error:{'_ bold} {long_message}{'_}"
                )?;
            } else {
                writemcln!(f, color, "{'bold}{long_message}{'_}")?;
            }
            if let Some(hint) = &self.hint {
                writemcln!(f, color, "{'c}hint:{'_} {hint}")?;
            }
            return Ok(());
        } else {
            &self.args
        };
        let error_idx = self.error_idx.clamp(0, args.len() - 1);

        let lengths: Vec<_> = args.iter().map(|a| a.chars().count()).collect();

        if self.anounce {
            writemcln!(
                f,
                color,
                "{'r}argument error:{'_ bold} {long_message}{'_}"
            )?;
        } else {
            writemcln!(f, color, "{'bold}{long_message}{'_}")?;
        }

        writemcln!(
            f,
            color,
            "{'b}--> {'_}arg{}:{}..{}",
            error_idx,
            self.error_span.start,
            self.error_span.end
        )?;
        writemcln!(f, color, "{'b} |{'_}")?;

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
            writemc!(f, color, " {'b}${'_} ")?;
            3
        } else {
            writemc!(f, color, " {'b}$ {'gr}...{'_} ")?;
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
            writemcln!(f, color, " {'gr}...{'_}")?;
        } else {
            writeln!(f)?;
        }

        err_pos -= 2;
        let err_len = self.error_span.len();
        writemcln!(
            f,
            color,
            " {'b}|{: >err_pos$}{'r}{:^>err_len$} {}{'_}",
            ' ',
            '^',
            self.inline_msg.as_deref().unwrap_or_else(|| &kind_msg)
        )?;
        let Some(hint) = &self.hint else {
            return Ok(());
        };

        writemcln!(f, color, "{'c}hint:{'_} {hint}")
    }
}
