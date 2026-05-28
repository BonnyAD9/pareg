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
///     #[arg("yes", "ok")]
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

/// Derives the [`pareg_core::FromArgs`] trait.
///
/// ## `#[from_args]` on field
/// - `<string literal>`: variant for the given field.
/// - `default`: The field is not required. Use the default implementation for
///   default value.
/// - `default = <expr>`: The field is not required. Use the given expression
///   for default value.
/// - `flag`: The field is [`bool`] which will be set to `true` if the flag is
///   present.
/// - `unnamed`: Specifies that this argument may be specified set by any
///   unknown argument. Unnamed arguments are filled in the order that they
///   are present in the source code. Unnamed arguments can also have names
///   specified to signify option to specify them explicitly. In addition,
///   multiple unnamed arguments may have the same name. In that case that name
///   will fill the first empty unnamed argument with that name.
/// - `collect`: Specifies that this argument is expected to be present
///   multiple times and all occurences will be collected into a collection.
///   The type has to have method `extend` available with the same sematics as
///   that of the trait [`Extend`]. The type must implement [`Default`] or the
///   default value must be specified with `default = <expr>`. This default is
///   representing empty collection. If `collect` is combined with `unnamed`,
///   and there are unnamed fields after collect, the unnamed fields after this
///   one will never be filled as unnamed as the collection will consume all
///   unnamed fields and never move to the next field.
/// - `collect = <range>`: Same as collect. This will also enable verification
///   that the number of items is within the given range. `<range>` may be any
///   expression for which `(<range>).contains(&field.len())` is valid and
///   returns [`bool`] where <field> is variable of the type of this field.
///   This is valid for example for standard ranges (e.g. `2..`) or arrays
///   (e.g. `[2]`), if the collection has method `len` which returns the number
///   of elements as [`usize`]. This range limit doesn't affect the behaviour
///   of combination of `unnamed` and `collect` (collect will consume all
///   remaining unnamed fields no matter the limitation in `<range>`).
/// - `no_rewrite`: Decides how repeating arguments are handled. If set, this
///   field will throw error when it would be set more than once. By default
///   the action is decided by attribute on the `FromArgs` type of which this
///   field is part, which is by default set to overwrite the old value.
/// - `rewrite`: The reverse of `no_rewrite`. This us useful to allow
///   owerwriting the default set by the `FromArgs` type of which is this
///   field.
///
/// ## `#[from_args]` on the type
/// - `match start { <arms> }`: custom match arms that will be before the arms
///   for the fields. All fields are accesible with their name, but they may be
///   option of that type instead of that type itself depending on the
///   configuration of the field.
/// - `match end { <arms> }`: same as `match start` but places the arms after
///   the arms for fields.
/// - `unnamed_guard`: if present, enables guarding of unnamed arguments. This
///   means that unnamed arguments starting with `-` are rejected as unknown
///   argument.
/// - `no_rewrite`: Decides how repeating arguments are handled. If set, fields
///   will throw error when they would be set more than once. By default,
///   rewrites are allowed and the latest value is used.
///
/// # Example
/// ```
/// use std::path::PathBuf;
/// use pareg_core::{self as pareg, Pareg};
/// use pareg_proc::FromArgs;
///
/// #[derive(FromArgs)]
/// #[from_args(match start { "-h" | "-?" | "--help" => println!("help") })]
/// struct Args {
///     #[from_args("-o", "--output", default = "output.png".into())]
///     output: PathBuf,
///     #[from_args("-v", "--verbose", flag, default)]
///     verbose: bool,
/// }
///
/// let mut args = Pareg::new(vec!["-o".into(), "test.png".into()]);
/// let parsed: Args = args.next_sub().unwrap();
///
/// assert_eq!(parsed.output, PathBuf::from("test.png"));
/// assert_eq!(parsed.verbose, false);
///
/// let mut args = Pareg::new(vec!["-v".into()]);
/// let parsed: Args = args.next_sub().unwrap();
///
/// assert_eq!(parsed.output, PathBuf::from("output.png"));
/// assert_eq!(parsed.verbose, true);
///
/// let mut args = Pareg::new(vec!["--lol".into()]);
///
/// assert!(args.next_sub::<Args>().is_err());
/// ```
#[proc_macro_derive(FromArgs, attributes(from_args))]
pub fn derive_from_args(item: TokenStream) -> TokenStream {
    pareg_core::proc::result_to_token_stream(
        pareg_core::proc::derive_from_args(item.into()),
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
