use pareg::{arg_list, parsef_part, FromRead, ParseResult};

#[test]
fn test_arg_list() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct Pair(i32, i32);

    impl FromRead for Pair {
        fn from_read(r: &mut pareg::Reader) -> ParseResult<Self> {
            let mut v = Pair::default();
            let r = parsef_part!(r, "({},{})", &mut v.0, &mut v.1);
            ParseResult::new(v, r)
        }
    }

    assert_eq!(
        arg_list::<Pair>("(1,2),(3,4),(5,6)", ",").unwrap(),
        vec![Pair(1, 2), Pair(3, 4), Pair(5, 6)]
    );
}
