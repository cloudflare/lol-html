use super::attribute_matcher::AttributeMatcher;
use super::compiler::{CompiledAttributeExpr, CompiledNonAttributeExpr};
use crate::html::LocalName;
use std::marker::PhantomData;
use std::ops::Range;

pub type AddressRange = Range<usize>;

pub type InstrResult<'i, P> = Option<&'i ExecutionBranch<P>>;

pub enum ExecOutcome<'i, P> {
    Result(InstrResult<'i, P>),
    AttributesRequested,
}

pub struct ExecutionBranch<P> {
    pub payload: Option<Vec<P>>,
    pub jumps: Option<AddressRange>,
    pub hereditary_jumps: Option<AddressRange>,
}

pub trait Instr<P> {
    fn exec<'i>(
        &'i self,
        local_name: &LocalName<'_>,
        attr_matcher: Option<&AttributeMatcher<'_>>,
    ) -> ExecOutcome<'i, P>;

    fn exec_with_attrs<'i>(&'i self, attr_matcher: &AttributeMatcher<'_>) -> InstrResult<'i, P>;
}

pub struct Program<P> {
    pub instructions: Vec<Box<dyn Instr<P>>>,
    pub entry_points: AddressRange,
}

pub struct InstrStub<P>(PhantomData<P>);

impl<P: 'static> InstrStub<P> {
    #[inline]
    pub fn new_boxed() -> Box<Self> {
        Box::new(InstrStub(PhantomData))
    }
}

impl<P> Instr<P> for InstrStub<P> {
    fn exec<'i>(
        &'i self,
        _: &LocalName<'_>,
        _: Option<&AttributeMatcher<'_>>,
    ) -> ExecOutcome<'i, P> {
        unreachable!("Instruction stub should never be executed");
    }

    fn exec_with_attrs<'i>(&'i self, attr_matcher: &AttributeMatcher<'_>) -> InstrResult<'i, P> {
        unreachable!("Instruction stub should never be executed");
    }
}

macro_rules! do_match {
    ($exprs:expr, $arg:ident, $branch:expr) => {
        if $exprs.iter().all(|e| e($arg)) {
            Some(&$branch)
        } else {
            None
        }
    };
}

pub struct NonAttrExprMatchingInstr<P> {
    associated_branch: ExecutionBranch<P>,
    exprs: Vec<CompiledNonAttributeExpr>,
}

impl<P> NonAttrExprMatchingInstr<P> {
    #[inline]
    pub fn new_boxed(
        associated_branch: ExecutionBranch<P>,
        exprs: Vec<CompiledNonAttributeExpr>,
    ) -> Box<Self> {
        Box::new(NonAttrExprMatchingInstr {
            associated_branch,
            exprs,
        })
    }
}

impl<P> Instr<P> for NonAttrExprMatchingInstr<P> {
    #[inline]
    fn exec<'i>(
        &'i self,
        local_name: &LocalName<'_>,
        _: Option<&AttributeMatcher<'_>>,
    ) -> ExecOutcome<'i, P> {
        ExecOutcome::Result(do_match!(self.exprs, local_name, self.associated_branch))
    }

    fn exec_with_attrs<'i>(&'i self, attr_matcher: &AttributeMatcher<'_>) -> InstrResult<'i, P> {
        unreachable!("Non attribute matching instruction should never request attributes");
    }
}

pub struct AttrExprMatchingInstr<P> {
    associated_branch: ExecutionBranch<P>,
    exprs: Vec<CompiledAttributeExpr>,
}

impl<P> AttrExprMatchingInstr<P> {
    #[inline]
    pub fn new_boxed(
        associated_branch: ExecutionBranch<P>,
        exprs: Vec<CompiledAttributeExpr>,
    ) -> Box<Self> {
        Box::new(AttrExprMatchingInstr {
            associated_branch,
            exprs,
        })
    }
}

impl<P> Instr<P> for AttrExprMatchingInstr<P> {
    #[inline]
    fn exec<'i>(
        &'i self,
        _: &LocalName<'_>,
        attr_matcher: Option<&AttributeMatcher<'_>>,
    ) -> ExecOutcome<'i, P> {
        match attr_matcher {
            Some(m) => ExecOutcome::Result(do_match!(self.exprs, m, self.associated_branch)),
            None => ExecOutcome::AttributesRequested,
        }
    }

    fn exec_with_attrs<'i>(&'i self, attr_matcher: &AttributeMatcher<'_>) -> InstrResult<'i, P> {
        do_match!(self.exprs, attr_matcher, self.associated_branch)
    }
}

pub struct GenericInstr<P> {
    associated_branch: ExecutionBranch<P>,
    non_attr_exprs: Vec<CompiledNonAttributeExpr>,
    attr_exprs: Vec<CompiledAttributeExpr>,
}

impl<P> GenericInstr<P> {
    #[inline]
    pub fn new_boxed(
        associated_branch: ExecutionBranch<P>,
        non_attr_exprs: Vec<CompiledNonAttributeExpr>,
        attr_exprs: Vec<CompiledAttributeExpr>,
    ) -> Box<Self> {
        Box::new(GenericInstr {
            associated_branch,
            non_attr_exprs,
            attr_exprs,
        })
    }
}

impl<P> Instr<P> for GenericInstr<P> {
    #[inline]
    fn exec<'i>(
        &'i self,
        local_name: &LocalName<'_>,
        attr_matcher: Option<&AttributeMatcher<'_>>,
    ) -> ExecOutcome<'i, P> {
        if !self.non_attr_exprs.iter().all(|e| e(local_name)) {
            ExecOutcome::Result(None)
        } else {
            match attr_matcher {
                Some(m) => {
                    ExecOutcome::Result(do_match!(self.attr_exprs, m, self.associated_branch))
                }
                None => ExecOutcome::AttributesRequested,
            }
        }
    }

    fn exec_with_attrs<'i>(&'i self, attr_matcher: &AttributeMatcher<'_>) -> InstrResult<'i, P> {
        do_match!(self.attr_exprs, attr_matcher, self.associated_branch)
    }
}
