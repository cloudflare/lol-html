use super::token::TestToken;

#[derive(Debug)]
pub struct TestTagNameInfo {
    pub name: String,
    pub name_hash: Option<u64>,
}

#[derive(Debug)]
pub enum TestTagPreview {
    StartTag(TestTagNameInfo),
    EndTag(TestTagNameInfo),
}

impl TestTagPreview {
    pub fn try_from_test_token(token: &TestToken) -> Option<TestTagPreview> {
        match token {
            TestToken::StartTag {
                name, name_hash, ..
            } => Some(TestTagPreview::StartTag(TestTagNameInfo {
                name: name.clone(),
                name_hash: *name_hash,
            })),
            TestToken::EndTag {
                name, name_hash, ..
            } => Some(TestTagPreview::EndTag(TestTagNameInfo {
                name: name.clone(),
                name_hash: *name_hash,
            })),
            _ => None,
        }
    }
}
