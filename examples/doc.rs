use std::process::ExitCode;

use pareg::{FromArg, Pareg, Result, starts_any};

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
struct Args {
    name: String,
    count: usize,
    colors: ColorMode,
}

impl Args {
    // create function that takes the arguments as ArgIterator
    pub fn parse(mut args: Pareg) -> Result<Self> {
        // initialize with default values
        let mut res = Args {
            name: "pareg".to_string(),
            count: 1,
            colors: ColorMode::Auto,
        };

        while let Some(arg) = args.next() {
            match arg {
                // when there is the argument `count`, parse the next value
                "-c" | "--count" => res.count = args.next_arg()?,
                // if the argument starts with either `--color` or
                // `--colour`, parse its value.
                a if starts_any!(a, "--color=", "--colour=") => {
                    res.colors = args.cur_val('=')?;
                }
                // it seems that this is flag, but it is not recognized
                a if a.starts_with('-') => Err(args.err_unknown_argument())?,
                // if the argument is unknown, just set it as name
                _ => res.name = arg.to_string(),
            }
        }

        Ok(res)
    }
}

// Now you can call your parse method:
fn start() -> Result<()> {
    // just pass in any iterator of string reference that has lifetime
    let args = Args::parse(Pareg::args())?;

    // Now you can use your arguments:
    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
    Ok(())
}

fn main() -> ExitCode {
    match start() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprint!("{e}");
            ExitCode::FAILURE
        }
    }
}
