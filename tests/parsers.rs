use pareg::{ArgError, FromRead, ReadFmt, Result, arg_list, parsef_part};

#[test]
fn test_arg_list() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct Pair(i32, i32);

    impl FromRead for Pair {
        fn from_read<'a>(
            r: &mut pareg::Reader,
            fmt: &'a ReadFmt<'a>,
        ) -> Result<(Self, Option<ArgError>)> {
            let mut v = Pair::default();
            let r = parsef_part!(r, "({:$fmt},{:$fmt})", &mut v.0, &mut v.1)?;
            Ok((v, r))
        }
    }

    assert_eq!(
        arg_list::<Pair>("(1,2),(3,4),(5,6)", ",").unwrap(),
        vec![Pair(1, 2), Pair(3, 4), Pair(5, 6)]
    );
}
