pub use pareg_core::*;
pub use pareg_proc::FromArg;

#[cfg(test)]
mod tests {
    use crate as pareg;
    use crate::{FromArg, err::Result, iter::ArgIterator};
    // use pareg_proc::FromArg;

    #[derive(FromArg, PartialEq, Debug)]
    enum ColorMode {
        Always,
        Never,
        Auto,
    }

    #[test]
    fn arg_iterator() -> Result<'static, ()> {
        let args = ["hello", "10", "0.25", "always"];
        let mut args = args.iter().cloned();

        assert_eq!("hello", args.next_arg::<&str>()?);
        assert_eq!(10, args.next_arg::<usize>()?);
        assert_eq!(0.25, args.next_arg::<f64>()?);
        assert_eq!(ColorMode::Always, args.next_arg::<ColorMode>()?);

        Ok(())
    }
}
