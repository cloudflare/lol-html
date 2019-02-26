use cool_thing::content::Doctype;

test_fixture!("Doctype token", {
    test("Serialization", {
        serialization_test!(
            r#"<!DOCTYPE html SYSTEM "hey">"#,
            Doctype,
            &[(
                "Parsed",
                Box::new(|_| {}),
                r#"<!DOCTYPE html SYSTEM "hey">"#,
            )]
        );
    });
});
