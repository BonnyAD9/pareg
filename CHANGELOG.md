# CHANGELOG

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
