# pareg
[![crates.io][version-badge]][crate]
[![donwloads][downloads-badge]][releases]

Helpful utilities for parsing command line arguments.

The aim of this crate is not to automate parsing of command line arguments
because there are many ways to construct a CLI for your application and
universal parser is not would be as hard to use as just writing it yourself.
Instead this crate provides useful types and parsing funcitonality to make the
process of writing your own code to parse command line arguments as simple as
possible: It provides special iterator for the arguments that can parse them
in various ways and plenty of useful parsing functions and macros. Everything
is made to minimize the repetetive part of the code for parsing the arguments
and with performance in mind. If you wan't to see examples see [docs][docs].

Main constructs:
- `ArgIterator`: iterator over arguments that can also parse them.
- `FromArg`: trait simmilar to `FromStr`. It is used by all the parsing
  functionality in this crate. There is also simple derive macro for enums.
    - It is implemented for all types in standard library that implement
      `FromStr` and there is simple trait to just mark `FromStr` implementation
      as also `FromArg`: `FromArgStr`.
- macros `starts_any` and `has_any_key`: useful for checking argument types.

## How to use it
Documentation and examples are available at the [docs][docs].

## How to get it
It is available on [crates.io][crate]:

### With cargo
```shell
cargo add pareg
```

### In Cargo.toml
```toml
[dependencies]
pareg = "0.1.0"
```

## Links
- **Author:** [BonnyAD9][author]
- **GitHub repository:** [BonnyAD/pareg][repo]
- **Package:** [crates.io][crate]
- **Documentation:** [docs.rs][docs]
- **My Website:** [bonnyad9.github.io][my-web]

[version-badge]: https://img.shields.io/crates/v/pareg
[downloads-badge]: https://img.shields.io/crates/d/pareg
[author]: https://github.com/BonnyAD9
[repo]: https://github.com/BonnyAD9/pareg
[docs]: https://docs.rs/pareg/latest/pareg/
[crate]: https://crates.io/crates/pareg
[my-web]: https://bonnyad9.github.io/
[releases]: https://github.com/BonnyAD9/pareg/releases
