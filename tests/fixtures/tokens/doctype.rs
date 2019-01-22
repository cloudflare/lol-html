use cool_thing::token::Doctype;

test_fixture!("Doctype token", {
    test("Serialization", {
        serialization_test!(
            Doctype,
            DOCTYPES,
            r#"<!DOCTYPE html SYSTEM "hey">"#,
            |doctype: Doctype<'_>| vec![(
                "Parsed",
                doctype.to_owned(),
                r#"<!DOCTYPE html SYSTEM "hey">"#
            )]
        );
    });
});
