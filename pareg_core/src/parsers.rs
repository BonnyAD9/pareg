use crate::{
    err::{ArgError, Result},
    from_arg::FromArg,
};

/// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
/// also parsed to the given type.
///
/// In case that there is no `'='`, value is `None`.
pub fn key_mval_arg<'a, T>(
    arg: &'a str,
    sep: char,
) -> Result<(&'a str, Option<T>)>
where
    T: FromArg<'a>,
{
    let Some((k, v)) = arg.split_once(sep) else {
        return Ok((arg, None));
    };

    Ok((k, Some(T::from_arg(v)?)))
}

/// If sep was `'='`, parses `"key=value"` into `"key"` and `value` that is
/// also parsed to the given type.
///
/// In case that there is no `'='`, returns [`ArgError::NoValue`].
pub fn key_val_arg<'a, T>(arg: &'a str, sep: char) -> Result<(&'a str, T)>
where
    T: FromArg<'a>,
{
    let Some((k, v)) = arg.split_once(sep) else {
        return Err(ArgError::NoValue(arg.into()));
    };

    Ok((k, T::from_arg(v)?))
}

/// Parse bool value in a specific way. If the value of lowercase `arg` is
/// equal to `t` returns true, if it is equal to `f` returns false and
/// otherwise returns error.
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
