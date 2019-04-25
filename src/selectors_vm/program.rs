use super::attribute_matcher::AttributeMatcher;
use super::compiler::{CompiledAttributeExpr, CompiledNonAttributeExpr};
use crate::html::LocalName;
use std::marker::PhantomData;
use std::ops::Range;

pub type AddressRange = Range<usize>;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutionBranch<P> {
    pub matched_payload: Option<Vec<P>>,
    pub jumps: Option<AddressRange>,
    pub hereditary_jumps: Option<AddressRange>,
}

type AttributesRequest<'i, P> =
    Box<dyn Fn(&AttributeMatcher) -> Option<&'i ExecutionBranch<P>> + 'i>;

type ExecWithoutAttrsResult<'i, P> =
    Result<Option<&'i ExecutionBranch<P>>, AttributesRequest<'i, P>>;

pub trait Instr<P> {
    fn try_exec_without_attrs<'i>(
        &'i self,
        local_name: &LocalName,
    ) -> ExecWithoutAttrsResult<'i, P>;

    fn exec<'i>(
        &'i self,
        local_name: &LocalName,
        attr_matcher: &AttributeMatcher,
    ) -> Option<&'i ExecutionBranch<P>>;
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
    fn try_exec_without_attrs<'i>(&'i self, _: &LocalName) -> ExecWithoutAttrsResult<'i, P> {
        unreachable!("Instruction stub should never be executed");
    }

    fn exec<'i>(&'i self, _: &LocalName, _: &AttributeMatcher) -> Option<&'i ExecutionBranch<P>> {
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
    fn try_exec_without_attrs<'i>(
        &'i self,
        local_name: &LocalName,
    ) -> ExecWithoutAttrsResult<'i, P> {
        Ok(do_match!(self.exprs, local_name, self.associated_branch))
    }

    #[inline]
    fn exec<'i>(
        &'i self,
        local_name: &LocalName,
        _: &AttributeMatcher,
    ) -> Option<&'i ExecutionBranch<P>> {
        do_match!(self.exprs, local_name, self.associated_branch)
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
    fn try_exec_without_attrs<'i>(&'i self, _: &LocalName) -> ExecWithoutAttrsResult<'i, P> {
        Err(Box::new(move |attr_matcher| {
            do_match!(self.exprs, attr_matcher, self.associated_branch)
        }))
    }

    #[inline]
    fn exec<'i>(
        &'i self,
        _: &LocalName,
        attr_matcher: &AttributeMatcher,
    ) -> Option<&'i ExecutionBranch<P>> {
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
    fn try_exec_without_attrs<'i>(
        &'i self,
        local_name: &LocalName,
    ) -> ExecWithoutAttrsResult<'i, P> {
        if !self.non_attr_exprs.iter().all(|e| e(local_name)) {
            Ok(None)
        } else {
            Err(Box::new(move |attr_matcher| {
                do_match!(self.attr_exprs, attr_matcher, self.associated_branch)
            }))
        }
    }

    #[inline]
    fn exec<'i>(
        &'i self,
        local_name: &LocalName,
        attr_matcher: &AttributeMatcher,
    ) -> Option<&'i ExecutionBranch<P>> {
        if self.non_attr_exprs.iter().all(|e| e(local_name)) {
            do_match!(self.attr_exprs, attr_matcher, self.associated_branch)
        } else {
            None
        }
    }
}
