use super::decoder::to_lower_null_decoded;
use super::token::TestToken;
use cool_thing::tokenizer::TagPreview;

#[derive(Debug, PartialEq, Eq)]
pub enum TagType {
    StartTag,
    EndTag,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TestTagPreview {
    name: String,
    name_hash: Option<u64>,
    tag_type: TagType,
}

impl TestTagPreview {
    pub fn try_from_test_token(token: &TestToken) -> Option<Self> {
        macro_rules! new_preview {
            ($ty:ident, $name:ident, $name_hash:ident) => {
                Some(TestTagPreview {
                    name: $name.clone(),
                    name_hash: *$name_hash,
                    tag_type: TagType::$ty,
                })
            };
        }

        match token {
            TestToken::StartTag {
                name, name_hash, ..
            } => new_preview!(StartTag, name, name_hash),
            TestToken::EndTag {
                name, name_hash, ..
            } => new_preview!(EndTag, name, name_hash),
            _ => None,
        }
    }

    pub fn from_tag_preview(tag_preview: &TagPreview) -> Self {
        let mut tag_type = TagType::StartTag;

        let tag_name_info = match tag_preview {
            TagPreview::StartTag(name_info) => name_info,
            TagPreview::EndTag(name_info) => {
                tag_type = TagType::EndTag;
                name_info
            }
        };

        TestTagPreview {
            name: to_lower_null_decoded(tag_name_info.get_name()),
            name_hash: tag_name_info.name_hash,
            tag_type,
        }
    }
}
