# pareg
Helpful utilities for parsing command line arguments.

Currently this crate doesn't contain any magic derive macro that would generate
code that parses your arguments. There are many ways that arguments may be used
and so there are only helper functions, traits and structures that help with
the parsing in a more manual way. (But there may be such derive macro in
the future.)

## Example usage
```rust

use crate::{err::Result, iter::ArgIterator, parsers::key_val_arg};

// You can define enums, and have them automaticaly derive FromArg where each
// enum variant will be parsed from case insensitive strings of the same name
// (e.g. `"Auto"` will parse into `Auto`, `"always"` into `Always`, `"NEVER"`
// into `Never`)
#[derive(FromArg)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

// create your struct that will hold the arguments
struct Args<'a> {
    name: &'a str,
    count: usize,
    colors: ColorMode,
}

impl<'a> Args<'a> {
    // create function that takes the arguments as ArgIterator
    pub fn parse<I>(mut args: I) -> Result<'a, Self> where I: ArgIterator<'a> {
        // initialize with default values
        let mut res = Args {
            name: "pareg",
            count: 1,
            colors: ColorMode::Auto,
        };

        while let Some(arg) = args.next() {
            match arg {
                // when there is the argument `count`, parse the next value
                "-c" | "--count" => res.count = args.next_arg()?,
                a if a.starts_with("--color=") => {
                    // This will accept
                    res.colors = key_val_arg::<&str, _>(a, '=')?.1;
                }
                // if the argument is unknown, just set it as name
                _ => res.name = arg,
            }
        }

        Ok(res)
    }
}

// Now you can call your parse method:
fn main() -> Result<'static, ()> {
    // you need to collect the arguments first so that you can refer to
    // them by reference
    let args: Vec<_> = std::env::args().collect();
    // just pass in any iterator of `&str` and it will parse it as
    // arguments
    let args = Args::parse(args.iter().map(|a| a.as_str()))
        // You need to map the error in this case to get the owned
        // version.
        .map_err(|e| e.into_owned())?;

    // Now you can use your arguments:
    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
    Ok(())
}
```
