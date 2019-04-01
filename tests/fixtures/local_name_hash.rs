use cool_thing::{LocalNameHash, Tag};

test_fixture!("Local name hash", {
    test("Should invalidate hash for non-ASCII alphanum values", {
        assert_eq!(LocalNameHash::from("div@&"), LocalNameHash::default());
    });

    test("Should invalidate hash for long values", {
        assert_eq!(
            LocalNameHash::from("aaaaaaaaaaaaaa"),
            LocalNameHash::empty()
        );
    });

    test("Precalculated hash values use current hashing algorithm", {
        assert_eq!(LocalNameHash::from("svg"), Tag::Svg);
        assert_eq!(LocalNameHash::from("math"), Tag::Math);
        assert_eq!(LocalNameHash::from("h1"), Tag::H1);
    });
});
