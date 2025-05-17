use std::{
    io::{Read, stdin},
    process::ExitCode,
};

use pareg::Result;
use pareg_proc::parsef_part;

fn main() -> ExitCode {
    match start() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn start() -> Result<()> {
    let mut ip: (u8, u8, u8, u8) = (0, 0, 0, 0);
    let mut mask = 0_u8;

    let input: Box<dyn Read> = Box::new(stdin());
    parsef_part!(
        &mut input.into(),
        "{}.{}.{}.{}/{mask}\n",
        &mut ip.0,
        &mut ip.1,
        &mut ip.2,
        &mut ip.3
    )?;

    println!("readed: {}.{}.{}.{}/{mask}", ip.0, ip.1, ip.2, ip.3);

    Ok(())
}
