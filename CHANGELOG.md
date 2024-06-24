# CHANGELOG

## v0.4.0
- Add new parsers: `key_arg`, `val_arg`, `mval_arg` and their implementations
  on `ArgIterator`
- Remove lifetime from errors
- Move proc macro to the same namespace as its trait.

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
