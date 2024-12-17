# CHANGELOG

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
