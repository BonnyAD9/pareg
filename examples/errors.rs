use std::process::ExitCode;

use pareg::{FromArg, Pareg, Result, key_val_arg, starts_any};

fn main() -> ExitCode {
    match start() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

#[derive(FromArg, Copy, Clone)]
enum EnableColor {
    Auto,
    Always,
    Never,
}

fn start() -> Result<()> {
    let mut args = Pareg::args();

    let mut _enable_color = EnableColor::Auto;
    let mut _output = String::new();
    let mut _num_pair = (0, 0);

    while let Some(arg) = args.next() {
        match arg {
            v if starts_any!(v, "--color=", "--colour=") => {
                _enable_color = args.cur_val('=')?;
            }
            "-o" | "--output" => {
                _output = args.next_arg()?;
            }
            v if v.starts_with("-D") => {
                _num_pair = args.cur_manual(|arg| {
                    let s = arg.strip_prefix("-D").unwrap();
                    key_val_arg(s, '=')
                })?;
            }
            _ => Err(args.err_unknown_argument())?,
        }
    }

    Ok(())
}
