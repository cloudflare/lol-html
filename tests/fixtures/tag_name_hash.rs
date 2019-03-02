use cool_thing::TagName;

test_fixture!("Tag name hash", {
    test("Should invalidate hash for non-ASCII aplhanum values", {
        assert_eq!(TagName::get_hash("div@&"), None);
    });

    test("Should invalidate hash for long values", {
        assert_eq!(TagName::get_hash("aaaaaaaaaaaaaa"), None);
    });

    test("Precalculated hash values use current hashing algorithm", {
        assert_eq!(TagName::get_hash("svg").unwrap(), TagName::Svg);
        assert_eq!(TagName::get_hash("math").unwrap(), TagName::Math);
        assert_eq!(TagName::get_hash("h1").unwrap(), TagName::H1);
    });
});
