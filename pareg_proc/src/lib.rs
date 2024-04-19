use proc_macro::TokenStream;

/// Derives the [`pareg_core::FromArg`] macro for an enum. The enum must not be
/// generic and the enum members cannot contain any  fields.
///
/// The parsing is case insensitive.
///
/// The arguments for the `arg` attribute must be lowercase to match properly.
///
/// # Examples
/// ```
/// use pareg_core::{self as pareg, FromArg};
/// use pareg_proc::FromArg;
///
/// #[derive(FromArg, PartialEq, Debug)]
/// enum ColorMode {
///     Auto,
///     #[arg("yes" | "ok")]
///     Always,
///     #[arg("no")]
///     Never,
/// }
///
/// assert_eq!(ColorMode::Auto, ColorMode::from_arg("auto").unwrap());
/// assert_eq!(ColorMode::Always, ColorMode::from_arg("Always").unwrap());
/// assert_eq!(ColorMode::Never, ColorMode::from_arg("NEVER").unwrap());
/// assert_eq!(ColorMode::Always, ColorMode::from_arg("yes").unwrap());
/// assert_eq!(ColorMode::Always, ColorMode::from_arg("oK").unwrap());
/// assert_eq!(ColorMode::Never, ColorMode::from_arg("NO").unwrap());
/// assert_eq!(ColorMode::Auto, ColorMode::from_arg("AuTo").unwrap());
/// ```
#[proc_macro_derive(FromArg, attributes(arg))]
pub fn derive_from_arg(item: TokenStream) -> TokenStream {
    pareg_core::proc::from_arg::derive_from_arg(item.into()).into()
}
