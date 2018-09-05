use cool_thing::tag_name_hash::{get_tag_name_hash, TagNameHash};

test_fixture!("Tag name hash", {
    test("Should invalidate hash on non-ASCII aplhanum values", {
        assert_eq!(get_tag_name_hash("div@&"), None);
    });

    test("Should invalidate hash on long values", {
        assert_eq!(get_tag_name_hash("aaaaaaaaaaaaaa"), None);
    });

    test("Precalculated hash values use current hashing algorithm", {
        assert_eq!(get_tag_name_hash("svg").unwrap(), TagNameHash::Svg as u64);
        assert_eq!(get_tag_name_hash("math").unwrap(), TagNameHash::Math as u64);
        assert_eq!(get_tag_name_hash("h1").unwrap(), TagNameHash::H1 as u64);
    });
});
