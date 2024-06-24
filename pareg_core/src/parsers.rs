use crate::{
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

    Ok((K::from_arg(k)?, Some(V::from_arg(v)?)))
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
        return Err(ArgError::NoValue(arg.into()));
    };

    Ok((K::from_arg(k)?, V::from_arg(v)?))
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
pub fn bool_arg<'a>(t: &str, f: &str, arg: &'a str) -> Result<'a, bool> {
    let lower = arg.to_lowercase();
    if arg == t {
        Ok(true)
    } else if arg == f {
        Ok(false)
    } else {
        Err(ArgError::FailedToParse {
            typ: "bool",
            value: lower.into(),
            msg: Some(format!("Value must be '{t}' or '{f}'").into()),
        })
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
pub fn opt_bool_arg<'a>(
    t: &str,
    f: &str,
    n: &str,
    arg: &'a str,
) -> Result<'a, Option<bool>> {
    let lower = arg.to_lowercase();
    if arg == t {
        Ok(Some(true))
    } else if arg == f {
        Ok(Some(false))
    } else if arg == n {
        Ok(None)
    } else {
        Err(ArgError::FailedToParse {
            typ: "bool",
            value: lower.into(),
            msg: Some(format!("Value must be '{t}' or '{f}'").into()),
        })
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
pub fn parse_arg<'a, T>(arg: &'a str) -> Result<'a, T>
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
pub fn key_arg<'a, T>(arg: &'a str, sep: char) -> Result<'a, T> where T: FromArg<'a> {
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
pub fn val_arg<'a, T>(arg: &'a str, sep: char) -> Result<'a, T> where T: FromArg<'a> {
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
pub fn mval_arg<'a, T>(arg: &'a str, sep: char) -> Result<'a, Option<T>> where T: FromArg<'a> {
    Ok(key_mval_arg::<&str, _>(arg, sep)?.1)
}
