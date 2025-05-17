use std::str::FromStr;

use pareg::{ArgError, FromArg, check, parsef, parsef_part};

#[test]
pub fn test_from_arg() {
    #[derive(PartialEq, Eq, Debug, FromArg)]
    enum Answer {
        #[arg("yes")]
        Always,
        #[arg("no")]
        Never,
        Auto,
    }

    assert_eq!(Answer::from_arg("always").unwrap(), Answer::Always);
    assert_eq!(Answer::from_arg("Always").unwrap(), Answer::Always);
    assert_eq!(Answer::from_arg("ALWAYS").unwrap(), Answer::Always);
    assert_eq!(Answer::from_arg("ALwAyS").unwrap(), Answer::Always);
    assert_eq!(Answer::from_arg("yes").unwrap(), Answer::Always);
    assert_eq!(Answer::from_arg("Yes").unwrap(), Answer::Always);
    assert_eq!(Answer::from_arg("YES").unwrap(), Answer::Always);
    assert_eq!(Answer::from_arg("YeS").unwrap(), Answer::Always);
    assert_eq!(Answer::from_arg("never").unwrap(), Answer::Never);
    assert_eq!(Answer::from_arg("no").unwrap(), Answer::Never);
    assert_eq!(Answer::from_arg("auto").unwrap(), Answer::Auto);
}

#[test]
pub fn test_parsef() {
    #[derive(Debug, Default, PartialEq)]
    struct Address {
        adr: (u8, u8, u8, u8),
        mask: u8,
    }

    impl FromStr for Address {
        type Err = ArgError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut res = Self::default();
            parsef!(
                &mut s.into(),
                "{}.{}.{}.{}/{}",
                &mut res.adr.0,
                &mut res.adr.1,
                &mut res.adr.2,
                &mut res.adr.3,
                &mut check::InRange(&mut res.mask, 0..33),
            )?;

            Ok(res)
        }
    }

    assert_eq!(
        Address::from_str("127.5.20.1/24").unwrap(),
        Address {
            adr: (127, 5, 20, 1),
            mask: 24
        }
    );

    let mut adr = Address::default();
    let res = parsef_part!(
        &mut "127.5.20.1/24some other stuff".into(),
        "{}.{}.{}.{}/{}",
        &mut adr.adr.0,
        &mut adr.adr.1,
        &mut adr.adr.2,
        &mut adr.adr.3,
        &mut check::InRange(&mut adr.mask, 0..33),
    );
    assert!(res.is_ok());

    assert_eq!(
        adr,
        Address {
            adr: (127, 5, 20, 1),
            mask: 24
        }
    );

    let mut a = (0., 0., 0.);
    let res = parsef!(
        &mut "3.1415/1.5E3/-.2".into(),
        "{}/{}/{}",
        &mut a.0,
        &mut a.1,
        &mut a.2,
    );

    assert!(res.is_ok());
    assert_eq!(a.0, 3.1415);
    assert_eq!(a.1, 1.5E3);
    assert_eq!(a.2, -0.2);
}

#[test]
pub fn test_format() {
    let mut num: u32 = 0;
    let res = parsef!(&mut "fea".into(), "{num:X}");

    assert!(res.is_ok());
    assert_eq!(num, 0xfea);

    let res = parsef_part!(&mut "123".into(), "{num:2}");

    assert!(res.is_ok());
    assert_eq!(num, 12);

    let res = parsef!(&mut " 123".into(), "{num}");
    assert!(res.is_err());

    let res = parsef!(&mut " 123".into(), "{num:>}");

    assert!(res.is_ok());
    assert_eq!(num, 123);

    let mut s = String::new();
    let res = parsef!(&mut "  ab    ".into(), "{s:^2..4}");

    assert!(res.is_ok());
    assert_eq!(s, "ab");

    let res = parsef!(&mut "  ab c  ".into(), "{s:^2..4}");

    assert!(res.is_ok());
    assert_eq!(s, "ab c");

    let res = parsef!(&mut "  ab    ".into(), "{s:^4}");

    assert!(res.is_ok());
    assert_eq!(s, "ab  ");

    let res = parsef!(&mut "  ab    ".into(), "{s:^3..4}");

    assert!(res.is_ok());
    assert_eq!(s, "ab ");
}
