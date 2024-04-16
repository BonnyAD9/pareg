pub mod err;
pub mod from_arg;
pub mod iter;
pub mod parsers;

#[cfg(test)]
mod tests {
    use crate::{err::Result, iter::ArgIterator};

    #[test]
    fn arg_iterator() -> Result<'static, ()> {
        let args = ["hello", "10", "0.25"];
        let mut args = args.iter().cloned();

        assert_eq!("hello", args.next_arg::<&str>()?);
        assert_eq!(10, args.next_arg::<usize>()?);
        assert_eq!(0.25, args.next_arg::<f64>()?);

        Ok(())
    }
}
