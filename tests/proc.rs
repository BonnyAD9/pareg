use std::{
    path::PathBuf,
    str::FromStr,
    sync::atomic::{self, AtomicBool},
};

use pareg::{ArgError, FromArg, FromArgs, Pareg, check, parsef, parsef_part};

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

static HELPED: AtomicBool = AtomicBool::new(false);

#[test]
pub fn test_from_args() {
    #[derive(FromArgs)]
    #[from_args(match start {
        "-h" | "-?" | "--help" => HELPED.store(true, atomic::Ordering::Relaxed)
    })]
    #[from_args(unnamed_guard)]
    struct Args {
        #[from_args("-o", "--output", default = "output.png".into())]
        output: PathBuf,
        #[from_args("-v", "--verbose", flag, default)]
        verbose: bool,
        #[from_args("-i", "--input1", unnamed)]
        input1: PathBuf,
        #[from_args("-i", "--input2", unnamed)]
        input2: PathBuf,
    }

    let mut args = Pareg::new(vec![
        "-o".into(),
        "test.png".into(),
        "-i".into(),
        "img.png".into(),
        "img2.png".into(),
    ]);
    let parsed: Args = args.next_sub().unwrap();

    assert_eq!(parsed.output, PathBuf::from("test.png"));
    assert_eq!(parsed.input1, PathBuf::from("img.png"));
    assert_eq!(parsed.input2, PathBuf::from("img2.png"));
    assert_eq!(parsed.verbose, false);
    assert!(!HELPED.load(atomic::Ordering::Relaxed));

    let mut args =
        Pareg::new(vec!["-v".into(), "img2.png".into(), "img3.png".into()]);
    let parsed: Args = args.next_sub().unwrap();

    assert_eq!(parsed.output, PathBuf::from("output.png"));
    assert_eq!(parsed.input1, PathBuf::from("img2.png"));
    assert_eq!(parsed.input2, PathBuf::from("img3.png"));
    assert_eq!(parsed.verbose, true);
    assert!(!HELPED.load(atomic::Ordering::Relaxed));

    let mut args = Pareg::new(vec![
        "--input2".into(),
        "img2.png".into(),
        "img1.png".into(),
    ]);
    let parsed: Args = args.next_sub().unwrap();

    assert_eq!(parsed.output, PathBuf::from("output.png"));
    assert_eq!(parsed.input1, PathBuf::from("img1.png"));
    assert_eq!(parsed.input2, PathBuf::from("img2.png"));
    assert_eq!(parsed.verbose, false);
    assert!(!HELPED.load(atomic::Ordering::Relaxed));

    let mut args = Pareg::new(vec![
        "--input2".into(),
        "img2.png".into(),
        "-i".into(),
        "img1.png".into(),
    ]);
    let parsed: Args = args.next_sub().unwrap();

    assert_eq!(parsed.output, PathBuf::from("output.png"));
    assert_eq!(parsed.input1, PathBuf::from("img1.png"));
    assert_eq!(parsed.input2, PathBuf::from("img2.png"));
    assert_eq!(parsed.verbose, false);
    assert!(!HELPED.load(atomic::Ordering::Relaxed));

    let mut args = Pareg::new(vec!["-h".into(), "img.png".into()]);
    assert!(args.next_sub::<Args>().is_err());

    let mut args = Pareg::new(vec![
        "-h".into(),
        "img.png".into(),
        "img2.png".into(),
        "img3.png".into(),
    ]);
    assert!(args.next_sub::<Args>().is_err());

    let mut args = Pareg::new(vec![
        "-h".into(),
        "img.png".into(),
        "img2.png".into(),
        "-i".into(),
        "img3.png".into(),
    ]);
    assert!(args.next_sub::<Args>().is_err());

    let mut args = Pareg::new(vec!["-h".into(), "--lol".into()]);
    assert!(args.next_sub::<Args>().is_err());
    assert!(HELPED.load(atomic::Ordering::Relaxed));
}

#[test]
pub fn test_from_args_collect() {
    #[derive(FromArgs)]
    #[from_args(unnamed_guard)]
    struct Args {
        #[from_args("-o", "--output", default = "output.png".into(), no_rewrite)]
        output: PathBuf,
        #[from_args("-v", "--verbose", flag, default)]
        verbose: bool,
        #[from_args("-i", "--input", unnamed, collect = 2..)]
        inputs: Vec<PathBuf>,
    }

    let mut args = Pareg::new(vec![
        "-o".into(),
        "test.png".into(),
        "-i".into(),
        "img.png".into(),
        "img2.png".into(),
    ]);
    let parsed: Args = args.next_sub().unwrap();

    assert_eq!(parsed.output, PathBuf::from("test.png"));
    assert_eq!(
        parsed.inputs,
        vec![PathBuf::from("img.png"), PathBuf::from("img2.png")]
    );
    assert_eq!(parsed.verbose, false);

    let mut args =
        Pareg::new(vec!["-v".into(), "img2.png".into(), "img3.png".into()]);
    let parsed: Args = args.next_sub().unwrap();

    assert_eq!(parsed.output, PathBuf::from("output.png"));
    assert_eq!(
        parsed.inputs,
        vec![PathBuf::from("img2.png"), PathBuf::from("img3.png")]
    );
    assert_eq!(parsed.verbose, true);

    let mut args = Pareg::new(vec!["img.png".into()]);
    assert!(args.next_sub::<Args>().is_err());

    let mut args = Pareg::new(vec![
        "img.png".into(),
        "img2.png".into(),
        "img3.png".into(),
    ]);
    let parsed: Args = args.next_sub().unwrap();

    assert_eq!(parsed.output, PathBuf::from("output.png"));
    assert_eq!(
        parsed.inputs,
        vec![
            PathBuf::from("img.png"),
            PathBuf::from("img2.png"),
            PathBuf::from("img3.png")
        ]
    );
    assert_eq!(parsed.verbose, false);

    let mut args = Pareg::new(vec![
        "img.png".into(),
        "img2.png".into(),
        "-i".into(),
        "img3.png".into(),
    ]);
    let parsed: Args = args.next_sub().unwrap();

    assert_eq!(parsed.output, PathBuf::from("output.png"));
    assert_eq!(
        parsed.inputs,
        vec![
            PathBuf::from("img.png"),
            PathBuf::from("img2.png"),
            PathBuf::from("img3.png")
        ]
    );
    assert_eq!(parsed.verbose, false);

    let mut args = Pareg::new(vec!["--lol".into()]);
    assert!(args.next_sub::<Args>().is_err());

    let mut args = Pareg::new(vec![
        "-o".into(),
        "in.png".into(),
        "-o".into(),
        "in2.png".into(),
    ]);
    assert!(args.next_sub::<Args>().is_err());
}

#[test]
pub fn test_from_args_option() {
    #[derive(FromArgs)]
    #[from_args(unnamed_guard)]
    struct Args {
        #[from_args("-v", "--verbose", option)]
        verbose: Option<bool>,
        #[from_args("-i", "--input", unnamed, option, collect)]
        input: Option<Vec<String>>,
    }

    let mut args = Pareg::new(vec!["-v".into(), "true".into()]);
    let parsed: Args = args.next_sub().unwrap();
    assert_eq!(parsed.verbose, Some(true));
    assert_eq!(parsed.input, None);

    let mut args = Pareg::new(vec!["-v".into(), "false".into()]);
    let parsed: Args = args.next_sub().unwrap();
    assert_eq!(parsed.verbose, Some(false));
    assert_eq!(parsed.input, None);

    let mut args = Pareg::new(vec!["one".into(), "two".into()]);
    let parsed: Args = args.next_sub().unwrap();
    assert_eq!(parsed.verbose, None);
    assert_eq!(
        parsed.input,
        Some(vec!["one".to_string(), "two".to_string()])
    );
}

#[test]
pub fn test_from_args_check() {
    #[derive(Debug, Clone, PartialEq, Eq, FromArg, Default)]
    enum Mode {
        #[default]
        Mode1,
        Mode2,
        Mode3,
    }

    #[derive(FromArgs)]
    #[from_args(unnamed_guard)]
    #[from_args(
        check = mode != Some(Mode::Mode3)
            || extension.as_deref() != Some("lol"))
    ]
    struct Args {
        #[from_args("-m", "--mode", default)]
        mode: Mode,
        #[from_args(
            "-e", check = matches!(mode, Some(Mode::Mode2 | Mode::Mode3)))
        ]
        extension: String,
    }

    let mut args = Pareg::new(vec![
        "-m".into(),
        "mode2".into(),
        "-e".into(),
        "lol".into(),
    ]);
    let parsed: Args = args.next_sub().unwrap();
    assert_eq!(parsed.mode, Mode::Mode2);
    assert_eq!(parsed.extension, "lol");

    let mut args = Pareg::new(vec![
        "-m".into(),
        "mode3".into(),
        "-e".into(),
        "lo2".into(),
    ]);
    let parsed: Args = args.next_sub().unwrap();
    assert_eq!(parsed.mode, Mode::Mode3);
    assert_eq!(parsed.extension, "lo2");

    let mut args = Pareg::new(vec![
        "-m".into(),
        "mode3".into(),
        "-e".into(),
        "lol".into(),
    ]);
    assert!(args.next_sub::<Args>().is_err());

    let mut args = Pareg::new(vec!["-e".into(), "lol".into()]);
    assert!(args.next_sub::<Args>().is_err());
}
