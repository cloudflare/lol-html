#[macro_use]
mod state_machine;

mod lexer;
mod tag_scanner;
mod tree_builder_simulator;

use self::lexer::Lexer;
pub(crate) use self::lexer::{
    AttributeBuffer, AttributeOutline, Lexeme, LexemeSink, NonTagContentLexeme,
    NonTagContentTokenOutline, TagLexeme, TagTokenOutline,
};
use self::state_machine::StateMachine;
pub(crate) use self::state_machine::{ActionError, ActionResult};
pub(crate) use self::tag_scanner::TagHintSink;
use self::tag_scanner::TagScanner;
pub use self::tree_builder_simulator::ParsingAmbiguityError;
use self::tree_builder_simulator::{TreeBuilderFeedback, TreeBuilderSimulator};
use crate::memory::SharedMemoryLimiter;
use crate::rewriter::RewritingError;
use cfg_if::cfg_if;

// NOTE: tag scanner can implicitly force parser to switch to
// the lexer mode if it fails to get tree builder feedback. It's up
// to consumer to switch the parser back to the tag scan mode in
// the tag handler.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ParserDirective {
    WherePossibleScanForTagsOnly,
    Lex,
}

pub(crate) struct ParserContext<S> {
    output_sink: S,
    memory_limiter: SharedMemoryLimiter,
    tree_builder_simulator: TreeBuilderSimulator,
    /// Amount of bytes consumed by previous calls to `parse()`,
    /// i.e. number of bytes from the start of the document until the start of the current input slice
    previously_consumed_byte_count: usize,
}

pub(crate) trait ParserOutputSink: LexemeSink + TagHintSink {}

// Pub only for integration tests
pub struct Parser<S> {
    lexer: Lexer<S>,
    tag_scanner: TagScanner<S>,
    current_directive: ParserDirective,
    context: ParserContext<S>,
}

// public only for integration tests
#[allow(private_bounds, private_interfaces)]
impl<S: ParserOutputSink> Parser<S> {
    #[must_use]
    #[inline(never)]
    pub fn new(
        output_sink: S,
        initial_directive: ParserDirective,
        strict: bool,
        memory_limiter: SharedMemoryLimiter,
    ) -> Self {
        let context = ParserContext {
            output_sink,
            memory_limiter,
            previously_consumed_byte_count: 0,
            tree_builder_simulator: TreeBuilderSimulator::new(strict),
        };

        Self {
            lexer: Lexer::new(),
            tag_scanner: TagScanner::new(),
            current_directive: initial_directive,
            context,
        }
    }

    // generic methods tend to be inlined, but this one is called from a couple of places,
    // and has cheap-to-pass non-constants args, so it won't benefit from being merged into its callers.
    // It's better to outline it, and let its callers be inlined.
    #[inline(never)]
    pub fn parse(&mut self, input: &[u8], last: bool) -> Result<usize, RewritingError> {
        let mut parse_result = match self.current_directive {
            ParserDirective::WherePossibleScanForTagsOnly => {
                self.tag_scanner
                    .run_parsing_loop(&mut self.context, input, last)
            }
            ParserDirective::Lex => self.lexer.run_parsing_loop(&mut self.context, input, last),
        };

        loop {
            let unboxed = match parse_result {
                Ok(unreachable) => match unreachable {},
                Err(boxed) => *boxed,
            };
            match unboxed {
                ActionError::EndOfInput {
                    consumed_byte_count,
                } => {
                    self.context.previously_consumed_byte_count += consumed_byte_count;
                    return Ok(consumed_byte_count);
                }
                ActionError::ParserDirectiveChangeRequired(new_directive, sm_bookmark) => {
                    self.current_directive = new_directive;

                    trace!(@continue_from_bookmark sm_bookmark, self.current_directive, input);

                    parse_result = match self.current_directive {
                        ParserDirective::WherePossibleScanForTagsOnly => self
                            .tag_scanner
                            .continue_from_bookmark(&mut self.context, input, last, sm_bookmark),
                        ParserDirective::Lex => self.lexer.continue_from_bookmark(
                            &mut self.context,
                            input,
                            last,
                            sm_bookmark,
                        ),
                    };
                }
                ActionError::RewritingError(err) => return Err(err),
                ActionError::Internal(err) => {
                    return Err(RewritingError::ContentHandlerError(err.into()));
                }
            }
        }
    }

    pub fn get_dispatcher(&mut self) -> &mut S {
        &mut self.context.output_sink
    }
}

cfg_if! {
    if #[cfg(feature = "_integration_test")] {
        use crate::html::{LocalNameHash, TextType};

        #[allow(private_bounds)]
        impl<S: ParserOutputSink> Parser<S> {
            pub fn switch_text_type(&mut self, text_type: TextType) {
                match self.current_directive {
                    ParserDirective::WherePossibleScanForTagsOnly => {
                        self.tag_scanner.switch_text_type(text_type);
                    }
                    ParserDirective::Lex => self.lexer.switch_text_type(text_type),
                }
            }

            pub fn set_last_start_tag_name_hash(&mut self, name_hash: LocalNameHash) {
                match self.current_directive {
                    ParserDirective::WherePossibleScanForTagsOnly => {
                        self.tag_scanner.set_last_start_tag_name_hash(name_hash);
                    }
                    ParserDirective::Lex => self.lexer.set_last_start_tag_name_hash(name_hash),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::{LocalName, Namespace};

    struct Sink;

    impl LexemeSink for Sink {
        fn handle_tag(&mut self, _: &TagLexeme<'_>) -> ActionResult<ParserDirective> {
            Ok(ParserDirective::Lex)
        }

        fn handle_non_tag_content(&mut self, _: &NonTagContentLexeme<'_>) -> ActionResult {
            Ok(())
        }
    }

    impl TagHintSink for Sink {
        fn handle_start_tag_hint(
            &mut self,
            _: LocalName<'_>,
            _: Namespace,
        ) -> Result<ParserDirective, RewritingError> {
            Ok(ParserDirective::Lex)
        }

        fn handle_end_tag_hint(
            &mut self,
            _: LocalName<'_>,
        ) -> Result<ParserDirective, RewritingError> {
            Ok(ParserDirective::Lex)
        }
    }

    impl ParserOutputSink for Sink {}

    #[test]
    fn attribute_charge_is_released_on_emission_and_drop() {
        let limiter = SharedMemoryLimiter::new(4096);
        let mut parser = Parser::new(Sink, ParserDirective::Lex, false, limiter.clone());
        parser.parse(b"<div a b", false).unwrap();
        assert!(limiter.current_usage() > 0);
        parser.parse(b"<div a b>", false).unwrap();
        assert_eq!(limiter.current_usage(), 0);
        parser.parse(b"<div a b", false).unwrap();
        assert!(limiter.current_usage() > 0);
        drop(parser);
        assert_eq!(limiter.current_usage(), 0);
    }

    #[test]
    fn attribute_charge_is_released_after_failure() {
        let limit = 8 * size_of::<AttributeOutline>();
        let limiter = SharedMemoryLimiter::new(limit);
        let mut parser = Parser::new(Sink, ParserDirective::Lex, false, limiter.clone());
        let input = format!("<div{}>", " a".repeat(9));
        assert!(matches!(
            parser.parse(input.as_bytes(), false),
            Err(RewritingError::MemoryLimitExceeded(_))
        ));
        assert_eq!(limiter.current_usage(), limit);
        drop(parser);
        assert_eq!(limiter.current_usage(), 0);
        let mut parser = Parser::new(Sink, ParserDirective::Lex, false, limiter.clone());
        parser.parse(b"<div a>", false).unwrap();
        assert_eq!(limiter.current_usage(), 0);
    }
}
