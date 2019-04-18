mod ast;
mod attribute_matcher;
mod compiler;
mod error;
mod parse;
mod stack;

use crate::html::LocalName;
use std::ops::Range;

pub use self::ast::*;
pub use self::attribute_matcher::AttributeMatcher;
pub use self::error::SelectorError;

type AddressRange = Range<usize>;

pub struct BranchState<P> {
    pub matched_payload: Option<P>,
    pub jumps: Option<AddressRange>,
    pub hereditary_jumps: Option<AddressRange>,
}

type AttributesRequest<P> = Box<dyn Fn(AttributeMatcher<'_>) -> Option<BranchState<P>>>;
type InstrResult<P> = Result<Option<BranchState<P>>, AttributesRequest<P>>;
type Instr<P> = Box<dyn Fn(LocalName<'_>) -> InstrResult<P>>;

pub struct Program<P> {
    instructions: Vec<Instr<P>>,
    entry_points: AddressRange,
}

impl<P> Default for Program<P> {
    #[inline]
    fn default() -> Self {
        Program {
            instructions: Vec::default(),
            entry_points: 0..0,
        }
    }
}
