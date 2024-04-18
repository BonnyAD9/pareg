# pareg
[![crates.io][version-badge]][crate]
[![donwloads][downloads-badge]][releases]

Helpful utilities for parsing command line arguments.

Currently this crate doesn't contain any magic derive macro that would generate
code that parses your arguments. There are many ways that arguments may be used
and so there are only helper functions, traits and structures that help with
the parsing in a more manual way. (But there may be such derive macro in
the future.)

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
