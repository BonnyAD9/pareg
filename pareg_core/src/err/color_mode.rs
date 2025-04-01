use std::io::{IsTerminal, stderr, stdout};

#[cfg(any(
    all(
        feature = "color-always",
        any(
            feature = "color-never",
            feature = "color-auto-stdout",
            feature = "color-auto-stderr"
        )
    ),
    all(
        feature = "color-never",
        any(
            feature = "color-always",
            feature = "color-auto-stdout",
            feature = "color-auto-stderr"
        )
    ),
    all(
        feature = "color-auto-stdout",
        any(
            feature = "color-always",
            feature = "color-never",
            feature = "color-auto-stderr"
        )
    ),
    all(
        feature = "color-auto-stderr",
        any(
            feature = "color-always",
            feature = "color-never",
            feature = "color-auto-stdout"
        )
    ),
))]
compile_error!(
    "Pareg features `color-*` are mutually exclusive. \
    Use only one of the features."
);

#[derive(Copy, Clone, Debug, Default)]
pub enum ColorMode {
    #[cfg_attr(feature = "color-always", default)]
    Always,
    #[cfg_attr(
        any(
            feature = "color-never",
            not(any(
                feature = "color-always",
                feature = "color-never",
                feature = "color-auto-stdout",
                feature = "color-auto-stderr"
            ))
        ),
        default
    )]
    Never,
    #[cfg_attr(feature = "color-auto-stderr", default)]
    AutoStderr,
    #[cfg_attr(feature = "color-auto-stdout", default)]
    AutoStdout,
}

impl ColorMode {
    pub fn use_color(&self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::AutoStderr => stderr().is_terminal(),
            ColorMode::AutoStdout => stdout().is_terminal(),
        }
    }
}
