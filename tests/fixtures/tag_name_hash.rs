use cool_thing::TagNameHash;

test_fixture!("Tag name hash", {
    test("Should invalidate hash for non-ASCII aplhanum values", {
        assert_eq!(TagNameHash::get("div@&"), None);
    });

    test("Should invalidate hash for long values", {
        assert_eq!(TagNameHash::get("aaaaaaaaaaaaaa"), None);
    });

    test("Precalculated hash values use current hashing algorithm", {
        assert_eq!(TagNameHash::get("svg").unwrap(), TagNameHash::Svg);
        assert_eq!(TagNameHash::get("math").unwrap(), TagNameHash::Math);
        assert_eq!(TagNameHash::get("h1").unwrap(), TagNameHash::H1);
    });
});
