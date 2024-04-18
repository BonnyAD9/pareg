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

pub fn parse_arg<'a, T>(arg: &'a str) -> Result<'a, T>
where
    T: FromArg<'a>,
{
    arg.arg_into()
}
