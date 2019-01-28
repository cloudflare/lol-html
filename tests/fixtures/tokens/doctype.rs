use cool_thing::token::Doctype;

test_fixture!("Doctype token", {
    test("Serialization", {
        let src = r#"<!DOCTYPE html SYSTEM "hey">"#;

        let test_cases = |doctypes: Vec<Doctype<'_>>, _| {
            vec![(
                "Parsed",
                doctypes[0].to_owned(),
                r#"<!DOCTYPE html SYSTEM "hey">"#,
            )]
        };

        serialization_test!(Doctype, DOCTYPES, src, test_cases);
    });
});
