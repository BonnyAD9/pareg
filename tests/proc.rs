use std::str::FromStr;

use pareg::{check, ArgError, FromArg};
use pareg_proc::parsef_part;

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
            parsef_part!(
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
}
