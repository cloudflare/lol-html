use crate::parser::{Lexeme, NextOutputType, TagHint};
use crate::token::{Token, TokenCaptureFlags};

pub trait TransformController {
    fn get_initial_token_capture_flags(&self) -> TokenCaptureFlags;
    fn get_token_capture_flags_for_tag(&mut self, tag_lexeme: &Lexeme<'_>) -> NextOutputType;
    fn get_token_capture_flags_for_tag_hint(&mut self, tag_hint: &TagHint<'_>) -> NextOutputType;
    fn handle_token(&mut self, token: &mut Token<'_>);
}
