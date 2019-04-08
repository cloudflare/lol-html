use cool_thing::{LocalNameHash, TAG_STR_PAIRS};

test_fixture!("Local name hash", {
    test("Should invalidate hash for non-ASCII alphanum values", {
        assert!(LocalNameHash::from("div@&").is_empty());
    });

    test("Should invalidate hash for long values", {
        assert!(LocalNameHash::from("aaaaaaaaaaaaaa").is_empty());
    });

    test("Precalculated hash values use current hashing algorithm", {
        for &(tag, tag_string) in TAG_STR_PAIRS {
            assert_eq!(LocalNameHash::from(tag_string), tag);
        }
    });
});
