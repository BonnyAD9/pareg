use crate::{
    ArgErrCtx, ArgErrKind, ColorMode, FromRead, Reader,
    arg_into::ArgInto,
    err::{ArgError, Result},
    from_arg::FromArg,
};

/// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
/// also parsed to the given type.
///
/// In case that there is no `'='`, value is `None`.
///
/// # Examples
/// ```rust
/// use pareg_core::key_mval_arg;
///
/// assert_eq!(
///     ("key", Some("value")),
///     key_mval_arg::<&str, &str>("key=value", '=').unwrap()
/// );
/// assert_eq!(
///     (5, Some(0.25)),
///     key_mval_arg::<i32, f64>("5:0.25", ':').unwrap()
/// );
/// assert_eq!(
///     ("only_key".to_owned(), None),
///     key_mval_arg::<String, f64>("only_key", '=').unwrap()
/// );
/// ```
pub fn key_mval_arg<'a, K, V>(
    arg: &'a str,
    sep: char,
) -> Result<(K, Option<V>)>
where
    K: FromArg<'a>,
    V: FromArg<'a>,
{
    let Some((k, v)) = arg.split_once(sep) else {
        return Ok((K::from_arg(arg)?, None));
    };

    Ok((
        K::from_arg(k).map_err(|e| e.shift_span(0, arg.to_string()))?,
        Some(V::from_arg(v).map_err(|e| {
            e.shift_span(k.len() + sep.len_utf8(), arg.to_string())
        })?),
    ))
}

/// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
/// also parsed to the given type.
///
/// In case that there is no `'='`, returns [`ArgError::NoValue`].
///
/// # Examples
/// ```rust
/// use pareg_core::key_val_arg;
///
/// assert_eq!(
///     ("key", "value"),
///     key_val_arg::<&str, &str>("key=value", '=').unwrap()
/// );
/// assert_eq!(
///     (5, 0.25),
///     key_val_arg::<i32, f64>("5:0.25", ':').unwrap()
/// );
/// ```
pub fn key_val_arg<'a, K, V>(arg: &'a str, sep: char) -> Result<(K, V)>
where
    K: FromArg<'a>,
    V: FromArg<'a>,
{
    let Some((k, v)) = arg.split_once(sep) else {
        return ArgError::new(ArgErrCtx {
            args: vec![arg.into()],
            error_span: 0..arg.len(),
            inline_msg: Some(format!("Missing separator `{sep}`.").into()),
            long_msg: Some(format!("Missing separator `{sep}` for key value pair.").into()),
            hint: Some(format!("Use the separator `{sep}` to split the argument into key and value.").into()),
            ..ArgErrCtx::new(ArgErrKind::NoValue)
        }).err();
    };

    Ok((
        K::from_arg(k).map_err(|e| e.shift_span(0, arg.to_string()))?,
        V::from_arg(v).map_err(|e| {
            e.shift_span(k.len() + sep.len_utf8(), arg.to_string())
        })?,
    ))
}

/// Parse bool value in a specific way. If the value of lowercase `arg` is
/// equal to `t` returns true, if it is equal to `f` returns false and
/// otherwise returns error.
///
/// # Examples
/// ```rust
/// use pareg_core::bool_arg;
///
/// assert_eq!(true, bool_arg("true", "false", "true").unwrap());
/// assert_eq!(true, bool_arg("yes", "no", "yes").unwrap());
/// assert_eq!(false, bool_arg("always", "never", "never").unwrap());
/// ```
pub fn bool_arg(t: &str, f: &str, arg: &str) -> Result<bool> {
    let lower = arg.to_lowercase();
    if lower == t {
        Ok(true)
    } else if lower == f {
        Ok(false)
    } else {
        ArgError::new(ArgErrCtx {
            args: vec![arg.into()],
            error_span: 0..arg.len(),
            inline_msg: Some("Invalid value.".into()),
            long_msg: Some(format!("Invalid value `{arg}`").into()),
            hint: Some(format!("Expected `{t}` or `{f}`").into()),
            ..ArgErrCtx::new(ArgErrKind::FailedToParse)
        })
        .err()
    }
}

/// Parse bool value in a specific way. If the value of lowercase `arg` is
/// equal to `t` returns true, if it is equal to `f` returns false and
/// if it is equal to `n` returns [`None`]. Otherwise returns error.
///
/// # Examples
/// ```rust
/// use pareg_core::opt_bool_arg;
///
/// assert_eq!(
///     Some(true),
///     opt_bool_arg("always", "never", "auto", "always").unwrap()
/// );
/// assert_eq!(
///     Some(false),
///     opt_bool_arg("always", "never", "auto", "never").unwrap()
/// );
/// assert_eq!(
///     None,
///     opt_bool_arg("always", "never", "auto", "auto").unwrap()
/// );
/// ```
pub fn opt_bool_arg(
    t: &str,
    f: &str,
    n: &str,
    arg: &str,
) -> Result<Option<bool>> {
    let lower = arg.to_lowercase();
    if lower == t {
        Ok(Some(true))
    } else if lower == f {
        Ok(Some(false))
    } else if lower == n {
        Ok(None)
    } else {
        ArgError::new(ArgErrCtx {
            args: vec![arg.into()],
            error_span: 0..arg.len(),
            inline_msg: Some("Invalid value.".into()),
            long_msg: Some(format!("Invalid value `{arg}`").into()),
            hint: Some(format!("Expected `{t}`, `{f}` or `{n}`").into()),
            color: ColorMode::default(),
            ..ArgErrCtx::new(ArgErrKind::FailedToParse)
        })
        .err()
    }
}

/// Parses the given argument using the [`FromArg`] trait.
///
/// # Examples
/// ```rust
/// use pareg_core::parse_arg;
///
/// assert_eq!("hello", parse_arg::<&str>("hello").unwrap());
/// assert_eq!(10, parse_arg::<i32>("10").unwrap());
/// assert_eq!(0.25, parse_arg::<f64>("0.25").unwrap());
/// ```
#[inline(always)]
pub fn parse_arg<'a, T>(arg: &'a str) -> Result<T>
where
    T: FromArg<'a>,
{
    arg.arg_into()
}

/// If sep was `'='`, parses `"key=value"` into `"key"` and discards `value`.
///
/// In case that there is no `'='`, parses the whole input.
///
/// # Examples
/// ```rust
/// use pareg_core::key_arg;
///
/// assert_eq!(
///     "key",
///     key_arg::<&str>("key=value", '=').unwrap()
/// );
/// assert_eq!(
///     5,
///     key_arg::<i32>("5:0.25", ':').unwrap()
/// );
/// ```
#[inline(always)]
pub fn key_arg<'a, T>(arg: &'a str, sep: char) -> Result<T>
where
    T: FromArg<'a>,
{
    Ok(key_mval_arg::<_, &str>(arg, sep)?.0)
}

/// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
/// given type.
///
/// In case that there is no `'='`, returns [`ArgError::NoValue`].
///
/// # Examples
/// ```rust
/// use pareg_core::val_arg;
///
/// assert_eq!(
///     "value",
///     val_arg::<&str>("key=value", '=').unwrap()
/// );
/// assert_eq!(
///     0.25,
///     val_arg::<f64>("5:0.25", ':').unwrap()
/// );
/// ```
#[inline(always)]
pub fn val_arg<'a, T>(arg: &'a str, sep: char) -> Result<T>
where
    T: FromArg<'a>,
{
    Ok(key_val_arg::<&str, _>(arg, sep)?.1)
}

/// If sep was `'='`, parses `"key=value"` into `value` that is parsed to the
/// given type.
///
/// In case that there is no `'='`, value is `None`.
///
/// # Examples
/// ```rust
/// use pareg_core::mval_arg;
///
/// assert_eq!(
///     Some("value"),
///     mval_arg::<&str>("key=value", '=').unwrap()
/// );
/// assert_eq!(
///     Some(0.25),
///     mval_arg::<f64>("5:0.25", ':').unwrap()
/// );
/// assert_eq!(
///     None,
///     mval_arg::<f64>("only_key", '=').unwrap()
/// );
/// ```
#[inline(always)]
pub fn mval_arg<'a, T>(arg: &'a str, sep: char) -> Result<Option<T>>
where
    T: FromArg<'a>,
{
    Ok(key_mval_arg::<&str, _>(arg, sep)?.1)
}

/// Tries to set the value of `res` to some if it is none. Throws error if it
/// is some.
///
/// # Examples
/// ```rust
/// use pareg_core::{try_set_arg_with, ArgInto};
///
/// let mut res: Option<i32> = None;
/// assert!(try_set_arg_with(&mut res, "-20", |a| a.arg_into()).is_ok());
/// assert_eq!(res, Some(-20));
/// assert!(try_set_arg_with(&mut res, "-20", |a| a.arg_into()).is_err());
/// ```
pub fn try_set_arg_with<'a, T>(
    res: &mut Option<T>,
    arg: &'a str,
    f: impl FnOnce(&'a str) -> Result<T>,
) -> Result<()> {
    if res.is_some() {
        ArgError::too_many_arguments(
            "Argument sets value that can be set only once.",
            arg,
        )
        .err()
    } else {
        *res = Some(f(arg)?);
        Ok(())
    }
}

/// Tries to set the value of `res` to some if it is none. Throws error if it
/// is some.
///
/// # Examples
/// ```rust
/// use pareg_core::{try_set_arg};
///
/// let mut res: Option<i32> = None;
/// assert!(try_set_arg(&mut res, "-20").is_ok());
/// assert_eq!(res, Some(-20));
/// assert!(try_set_arg(&mut res, "-20").is_err());
/// ```
pub fn try_set_arg<'a, T: FromArg<'a>>(
    res: &mut Option<T>,
    arg: &'a str,
) -> Result<()> {
    try_set_arg_with(res, arg, T::from_arg)
}

/// Splits `arg` by separator `sep` and parses each word into a resulting
/// vector.
///
/// Difference from [`arg_list`] is that this will first to split and than try
/// to parse.
///
/// # Examples
/// ```rust
/// use pareg_core::split_arg;
///
/// assert_eq!(split_arg::<i32>("1,2,3", ",").unwrap(), vec![1, 2, 3]);
/// ```
pub fn split_arg<'a, T: FromArg<'a>>(
    arg: &'a str,
    sep: &str,
) -> Result<Vec<T>> {
    let mut r = vec![];
    for s in arg.split(sep) {
        r.push(s.arg_into()?);
    }
    Ok(r)
}

/// Parses multiple values in `arg` separated by `sep`.
///
/// Unlike [`split_arg`], this will first try to parse and than check if the
/// separator is present. So valid values may contain contents of `sep`, and
/// it will properly parse the vales, whereas [`split_arg`] would split `arg`
/// and than try to parse.
///
/// # Examples
/// ```ignore
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// struct Pair(i32, i32);
///
/// impl FromRead for Pair {
///     fn from_read(r: &mut pareg::Reader) -> Result<(Self, Option<ArgError>)> {
///         let mut v = Pair::default();
///         let r = parsef_part!(r, "({},{})", &mut v.0, &mut v.1)?;
///         Ok((v, r))
///     }
/// }
///
/// assert_eq!(
///     arg_list::<Pair>("(1,2),(3,4),(5,6)", ",").unwrap(),
///     vec![Pair(1, 2), Pair(3, 4), Pair(5, 6)]
/// );
/// ```
pub fn arg_list<T: FromRead>(arg: &str, sep: &str) -> Result<Vec<T>> {
    let mut res = vec![];
    let mut reader: Reader = arg.into();
    loop {
        let (item, _) = T::from_read(&mut reader, &"".into())?;
        res.push(item);
        if reader.peek()?.is_none() {
            return Ok(res);
        }
        reader.expect(sep)?;
    }
}
