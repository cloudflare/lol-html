use cool_thing::{Doctype, UserData};
use encoding_rs::UTF_8;

test_fixture!("Doctype token", {
    test("User data", {
        parse_token!("<!doctype>", UTF_8, Doctype, |d: &mut Doctype| {
            d.set_user_data(42usize);

            assert_eq!(
                *d.user_data().unwrap().downcast_ref::<usize>().unwrap(),
                42usize
            );
        });
    });

    test("Serialization", {
        serialization_test!(
            r#"<!DOCTYPE html SYSTEM "hey">"#,
            Doctype,
            &[(
                "Parsed",
                Box::new(|_, _| {}),
                r#"<!DOCTYPE html SYSTEM "hey">"#,
            )]
        );
    });
});
