# pareg
[![crates.io][version-badge]][crate]
[![donwloads][downloads-badge]][releases]

Helpful utilities for parsing command line arguments and also parsing in
general.

The aim of this crate is to simplify parsing of command line arguments. For
basic argument parsing you can use the derive macro `FromArgs`. If that doesn't
suffice, you can use pareg to automate the more manual way of parsing command
line arguments.

### Main constructs:
- `Pareg`: istruct that will help with parsing of arguments.
- `FromArg`: trait simmilar to `FromStr`. It is used by all the parsing
  functionality in this crate. There is also simple derive macro for enums.
    - It is implemented for all types in standard library that implement
      `FromStr` and there is simple trait to just mark `FromStr` implementation
      as also `FromArg`: `FromArgStr`.
- `FromArgs`: trait for types that can parse from multiple arguments.
- macros `starts_any` and `has_any_key`: useful for checking argument types.

### Example error message
```txt
argument error: Unknown option `no`.
--> arg1:8..10
 |
 $ my-program --color=no
 |                    ^^ Unknown option.
hint: Valid options are: `auto`, `always`, `never`.
```

## Usage

If you want to see the more manual usage see [docs][docs]. This shows basic
parsing with the `FromArgs` derive macro.

```rust
use std::path::PathBuf;
use pareg::{self, Pareg, FromArgs};

// Derive `FromArgs`.
#[derive(FromArgs)]
struct Args {
    // Specify the argument names as plain strings.
    // You can specify custom default value. If no default value is set the
    // argument is required.
    #[from_args("-o", "--output", default = "output.png".into())]
    output: PathBuf,
    // Flag argument. For flag arguments the default value is implied to be
    // false.
    #[from_args("-v", "--verbose", flag)]
    verbose: bool,
    // Another flag argument, when it is encountered, do the action specified
    // with `act`.
    #[from_args("-h", "-?", "--help", flag, act = println!("help"))]
    helped: bool,
}

impl Args {
    pub fn parse(mut args: Pareg) -> pareg::Result<Self> {
        args.next_sub()
    }
}
```

You can do much more with the derive macro. For more information see the
[documentation][from-args-doc] of the derive macro itself.

### Features
The features in this macro are mainly used to modfy some default behaviour:
- `default`: `color-auto-stderr`
- `color-auto-stderr`: Enable colored errors if stderr is terminal.
- `color-auto-stdin`: Enable colored errors if stdout is terminal.
- `color-never`: Disable colored errors.
- `color-always`: Enable colored errors.
- `no-anounce`: Don't print `argument error:` or `error:` before the error.
- `short-errors`: Always print only the error message.

## How to get it
It is available on [crates.io][crate]:

### With cargo
```shell
cargo add pareg
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
[from-args-doc]: https://docs.rs/pareg/latest/pareg/derive.FromArgs.html
