use pareg::{
    FromArg, Reader, Result, SetFromRead,
    check::{self, CheckRef},
};

#[test]
fn test_check_ref() {
    let mut n = 0_u32;

    let check_fun = |r: &Reader, pos: usize, a: &u32| -> Result<()> {
        if a % 2 == 0 {
            Ok(())
        } else {
            r.err_value("").span_start(pos).err()
        }
    };

    assert!(matches!(
        CheckRef(&mut n, &check_fun)
            .set_from_read(&mut "8".into(), &"".into()),
        Ok(None)
    ));
    assert_eq!(n, 8);
    assert!(matches!(
        CheckRef(&mut n, &check_fun)
            .set_from_read(&mut "9".into(), &"".into()),
        Err(_)
    ));
    assert_eq!(n, 9);
}

#[test]
fn test_in_range_i() {
    assert_eq!(
        check::InRangeI::<isize, 0, 100>::from_arg("20").unwrap().0,
        20
    );
    assert_eq!(
        check::InRangeI::<usize, 0, 100>::from_arg("0").unwrap().0,
        0
    );
    assert_eq!(
        check::InRangeI::<isize, 0, 100>::from_arg("99").unwrap().0,
        99
    );
    assert!(matches!(
        check::InRangeI::<isize, 0, 100>::from_arg("100"),
        Err(_)
    ));
    assert!(matches!(
        check::InRangeI::<isize, 0, 100>::from_arg("-1"),
        Err(_)
    ));
}

#[test]
fn test_in_range() {
    let mut n = 0_i32;

    assert!(matches!(
        check::InRange(&mut n, 0..100)
            .set_from_read(&mut "20".into(), &"".into()),
        Ok(_)
    ));
    assert_eq!(n, 20);
    assert!(matches!(
        check::InRange(&mut n, 0..100)
            .set_from_read(&mut "0".into(), &"".into()),
        Ok(_)
    ));
    assert_eq!(n, 0);
    assert!(matches!(
        check::InRange(&mut n, 0..100)
            .set_from_read(&mut "99".into(), &"".into()),
        Ok(_)
    ));
    assert_eq!(n, 99);
    assert!(matches!(
        check::InRange(&mut n, 0..100)
            .set_from_read(&mut "-1".into(), &"".into()),
        Err(_)
    ));
    assert_eq!(n, -1);
    assert!(matches!(
        check::InRange(&mut n, 0..100)
            .set_from_read(&mut "100".into(), &"".into()),
        Err(_)
    ));
    assert_eq!(n, 100);
}
