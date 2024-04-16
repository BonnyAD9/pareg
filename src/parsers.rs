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
