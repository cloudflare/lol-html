use cool_thing::Doctype;

test_fixture!("Doctype token", {
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
