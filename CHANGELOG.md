# CHANGELOG

## v0.11.2
### New features
- Add `ArgError::map_ctx()`

## v0.11.1
### New features
- Add `ArgError::kind()`.

## v0.11.0
### New features
- You can now disable error anouncment with `anounce` in `ArgErrCtx` and
  `ArgError`. The default value can be changed with the feature `no-anounce`.
- New constructor `ArgErrCtx::new` to create default error context for the
  given error kind.
- The display implementations of `ArgError` and `ArgErrCtx` use the formats
  `-` and `+` to disable/enable colors.
- New constructors `ArgError::new`, `ArgError::from_msg` and
  `ArgError::too_many_arguments`.
- `ArgError` now implements `From<Box<ArgErrCtx>>`
- `ArgError::color_mode` now takes `impl Into<ColorMode>`.
- `ColorMode` now implements `From<bool>`.

### Breaking changes
- `ArgErrCtx::message` is renamed to `inline_msg`.
- `ArgErrCtx::long_message` is renamed to `long_msg`.
- Add new fields to `ArgErrCtx`: `kind` and `anounce`.
- `ArgErrCtx::inline_msg` is now option.
- `ArgErrCtx::from_inner` now takes additional first argument `kind`.
- `ArgErrCtx::from_msg` now takes additional first argument `kind`.
- Methods on `ArgErrCtx` that previously took argument `self` now take argument
  `&mut self` and don't return `Self`.
- `ArgErrCtx::main_msg` is renamed to `ArgErrCtx::long_msg`.
- `ArgError` is now struct without public fields.
- `ArgError::parse_msg` is renamed to `ArgError::failed_to_parse`.
- `ArgError::value_msg` is renamed to `ArgError::invalid_value`.
- `ArgError::main_msg` is renamed to `ArgError::long_msg`.
- `ArgError::map_ctx` has been removed.
- The error `NoLastArgument` has been removed. Pareg will panic in its place as
  this error condition is certainly bug in code.
- Method arguments on `ArgError` corresponding to argument with type `String`
  are now `impl Into<String>`.

### Changes
- Error kind is now in its separate enum `ArgErrKind`

## v0.10.0
### New features
- parsef macros now support formats for types.
- `Reader` new methods for reader:
    - `map_err_peek`, `err_parse_peek`, `err_value_peek`
    - `trim_left`, `trim_right`
    - `unnext`, `prepend`

### Fixes
- Properly reject empty string as number in parsef macros.

### Breaking changes
- `FromRead` and `SetFromRead` now also have format argument.
- `SetFromRead` is now not implemented automatically for all types that
  implement `FromRead`, but it can be esily implemented with trait
  `AutoSetFromRead`.
- Change the way that position is tracked in `Reader`.

## v0.9.1
### Fixes
- Fix typo in error message.

## v0.9.0
### New features
- Add `FromRead` implementations for `f32`, `f64`, `bool`, `char`, `String`,
  `PathBuf`, `OsString`, `Ipv4Addr` and `SocketAddrV4`.
- Add new methods to `Reader`: `skip_while`, `is_next_some`, `is_next`,
  `chars`, `parse` and `next`.

### Changes
- `Reader` no longer implements the iterator trait. Use `.chars()` to get
  iterator over chars.

## v0.8.0
### New features
- Add parsers `try_set_arg_with` and `try_set_arg`.
- Add pareg methods `try_set_cur_with`, `try_set_next_with`, `try_set_cur` and
  `try_set_next`.
- Add parsef functionality (function and macro).
- Add `Reader` for parsing.
- Add `ParegRef`. It allows parsing to continue even if there are references to
  the original arguments.
- Add checkers: `check::CheckRef`, `check::InRangeI` and `check::InRange`.
- Add new parsers `split_arg` and `arg_list` and their respective methods on
  `ParegRef` and `Pareg`.
- Add new macros `impl_from_str_with_read!` and `impl_from_arg_str_with_read!`.

### Changes
- Rename `Pareg::map_err` to `Pareg::map_res`, add `Pareg::map_err`.

## v0.7.0
+ Add `Pareg::cur_idx` and `Pareg::next_idx`
+ Add `Pareg::peek` and `Pareg::get`
+ Add `Pareg::skip` and `Pareg::skip_all`.
+ Add `Pareg::jump` and `Pareg::reset`.
+ Add `Error::TooManyArguments`.
+ Add `color_mode` and `no_color` to `ArgError` and `ArgErrCtx`.
- Fix `Pareg::remaining` and `Pareg::cur_remaining`.

## v0.6.1
- Fix panic in `Pareg::remaining`.

## v0.6.0
- Add `all_arg`, `remaining` and `cur_remaining` to `Pareg`.
- Add `cur_val_or_next` to `Pareg`.
- Add new error for checking value validity `ArgError::InvalidValue`.
- Add `err_invalid`, `err_invalid_value` and `err_invalid_span` to `Pareg`.
- Make `ArgErrCtx::from_msg` take `impl Into<Cow>` instead of `String`.
- Add `ArgErrCtx::spanned` and `ArgError::spanned`.
- Add `ArgErrCtx::from_inner`.
- Add `part_of`, `inline_msg` and `main_msg` to `ArgErrCtx` and `ArgError`.
- Add `parse_msg` and `err` to `ArgError`
- Require `FromArg` to return `Result<ArgError, T>`
- Remove unncesary mut requirements on `Pareg`.

## v0.5.2
- Make errors store box of ErrCtx to reduce the size of the results.
- Add colors to errors (modify defaults with features).

## v0.5.1
- Fix dependencies

## v0.5.0
- Very user friendly error messages.
- Remove `ArgIterator` in favor of `Pareg`. They both have very similar
  functionality, but work in a sligtly different way that allows the better
  error messages.

## v0.4.0
- Add new parsers: `key_arg`, `val_arg`, `mval_arg` and their implementations
  on `ArgIterator`
- Remove lifetime from errors
- Move proc macro to the same namespace as its trait.
- Add macros `starts_any` and `has_any_key`

## v0.3.0
- Add `arg` attribute to `FromArg` derive macro.

## v0.2.0
- Make `ArgIterator` struct instead of trait
- Remember the last returned value on `ArgIterator`
- More parsing option s on `ArgIterator`

## v0.1.1
- Fix visibility of certain items
- Additional documentation

## v0.1.0
- ArgIterator iterator for command line arguments, supports parsing
- Functions for parsing arguments:
    - key_mval_arg, key_val_arg, bool_arg, opt_bool_arg
- FromArg derive proc macro for enums
