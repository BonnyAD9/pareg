/// Checks if string starts with any of the given values.
///
/// # Examples
/// ```rust
/// use pareg_core::starts_any;
///
/// let s = "ah";
/// assert!(starts_any!("hello", "he", s));
/// assert!(starts_any!("ahoj", "he", s));
/// assert!(!starts_any!("greetings", "he", s));
/// ```
#[macro_export]
macro_rules! starts_any {
    ($v:expr) => {
        false
    };

    ($v:expr, $($st:expr),* $(,)?) => {
        ($($v.starts_with($st))||*)
    };
}

/// Checks if string is key value with the given separator or just key with one
/// of the keys.
///
/// If you want the separator to be mandatory use [`starts_any`].
///
/// # Examples
/// ```rust
/// use pareg_core::has_any_key;
///
/// let s = "ahoj";
/// let sep = ':';
/// assert!(has_any_key!("hello", '=', "hello", s));
/// assert!(has_any_key!("hello=", '=', "hello", s));
/// assert!(has_any_key!("ahoj:lol", sep, "hello", s));
/// assert!(!has_any_key!("greeting=ahoj", '=', "greet", s));
/// ```
#[macro_export]
macro_rules! has_any_key {
    ($v:expr, $sep:expr) => {
        false
    };

    ($v:expr, $sep:expr, $($key:expr),* $(,)?) => {
        ($(
            $v.strip_prefix($key)
                .map_or(false, |v| v.is_empty() || v.starts_with($sep))
        )||*)
    };
}
