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
    pareg_core::proc::result_to_token_stream(
        pareg_core::proc::derive_from_arg(item.into()),
    )
    .into()
}

/// This macro can be tought of as opposite of [`write!`] or as something like
/// `fscanf` in C.
///
/// As arguments, takes reader to parse, format string and than arguments to
/// which result will be written.
///
/// The format string can contain format strings for the specific arguments
/// after `:`. The format is `CTS..ER` where:
/// - `CT` is optional trim mode.
///     - `C` is optional character to trim. If not present, trim whitespace.
///     - `T` is the side from which to trim. It is the opposite of alignment
///       in format functions:
///         - `<` trim from right.
///         - `>` trim from left.
///         - `^` trim from both sides.
/// - `S..E` is optional length range. The parsing function should use at least
///   `S` and at most `E` characters.
///     - `S`, `E` or both may be omited. In that case `S` will be same as `0`
///       and `E` will be same as max length.
///     - If only `S` is present (without `..E`), it is same as `S..S`.
/// - `R` is optional radix for conversion. It may be:
///     - `D` as decimal.
///     - `X` as hexadecimal.
///     - `O` as octal.
///
/// Anything else after the format is custom format string for the given type.
/// Nothing forces the parsing function to follow the standart formatting and
/// no format is invalid.
///
/// # Returns
/// [`pareg_core::Result<()>`] that indicates success or failure.
///
/// # Example
///
/// ```rust
/// use std::str::FromStr;
/// use pareg_core::{self as pareg, ArgError, check};
/// use pareg_proc::parsef;
///
/// #[derive(Debug, Default, PartialEq)]
/// struct Address {
///     adr: (u8, u8, u8, u8),
///     mask: u8,
/// }
///
/// impl FromStr for Address {
///     type Err = ArgError;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         let mut res = Self::default();
///         parsef!(
///             &mut s.into(),
///             "{}.{}.{}.{}/{}",
///             &mut res.adr.0,
///             &mut res.adr.1,
///             &mut res.adr.2,
///             &mut res.adr.3,
///             &mut check::InRange(&mut res.mask, 0..33),
///         )?;
///
///         Ok(res)
///     }
/// }
///
/// assert_eq!(
///     Address::from_str("127.5.20.1/24").unwrap(),
///     Address {
///         adr: (127, 5, 20, 1),
///         mask: 24
///     }
/// );
/// ```
#[proc_macro]
pub fn parsef(args: TokenStream) -> TokenStream {
    pareg_core::proc::proc_parsef(args.into(), false).into()
}

/// Simmilar to [`parsef!`], but doesn't expect to parse the whole string, but
/// only start of the string. It macro can be tought of as opposite of
/// [`write!`] or as something like `fscanf` in C.
///
/// As arguments, takes reader to parse, format string and than arguments to
/// which result will be written.
///
/// The format string can contain format strings for the specific arguments
/// after `:`. The format is `CTS..ER` where:
/// - `CT` is optional trim mode.
///     - `C` is optional character to trim. If not present, trim whitespace.
///     - `T` is the side from which to trim. It is the opposite of alignment
///       in format functions:
///         - `<` trim from right.
///         - `>` trim from left.
///         - `^` trim from both sides.
/// - `S..E` is optional length range. The parsing function should use at least
///   `S` and at most `E` characters.
///     - `S`, `E` or both may be omited. In that case `S` will be same as `0`
///       and `E` will be same as max length.
///     - If only `S` is present (without `..E`), it is same as `S..S`.
/// - `R` is optional radix for conversion. It may be:
///     - `D` as decimal.
///     - `X` as hexadecimal.
///     - `O` as octal.
///
/// # Returns
/// `pareg_core::Result<Option<pareg_core::ArgError>>` that indicates success
/// or failure. On success, if the string was not fully parsed also returns
/// error that should be raised if it was expected to parse more of the string.
///
/// # Example
/// ```rust
/// use pareg_core::{self as pareg, ArgError, check};
/// use pareg_proc::parsef_part;
///
/// #[derive(Debug, Default, PartialEq)]
/// struct Address {
///     adr: (u8, u8, u8, u8),
///     mask: u8,
/// }
///
/// let mut adr = Address::default();
/// let res = parsef_part!(
///     &mut "127.5.20.1/24some other stuff".into(),
///     "{}.{}.{}.{}/{}",
///     &mut adr.adr.0,
///     &mut adr.adr.1,
///     &mut adr.adr.2,
///     &mut adr.adr.3,
///     &mut check::InRange(&mut adr.mask, 0..33),
/// );
/// assert!(res.is_ok());
///
/// assert_eq!(
///     adr,
///     Address {
///         adr: (127, 5, 20, 1),
///         mask: 24
///     }
/// );
/// ```
#[proc_macro]
pub fn parsef_part(args: TokenStream) -> TokenStream {
    pareg_core::proc::proc_parsef(args.into(), true).into()
}
