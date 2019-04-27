mod ast;
mod attribute_matcher;
mod compiler;
mod error;
mod parser;
mod program;
mod stack;

use self::program::AddressRange;
use self::stack::{Stack, StackDirective, StackItem};
use crate::html::{LocalName, Namespace};
use crate::transform_stream::AuxStartTagInfo;
use encoding_rs::Encoding;
use std::fmt::Debug;
use std::hash::Hash;

pub use self::ast::*;
pub use self::attribute_matcher::AttributeMatcher;
pub use self::compiler::Compiler;
pub use self::error::SelectorError;
pub use self::program::{ExecutionBranch, Program};

pub type AuxStartTagInfoRequest<'v> = Box<dyn FnOnce(AuxStartTagInfo) + 'v>;

pub struct MatchInfo<P> {
    pub payload: P,
    pub with_content: bool,
}

#[derive(Default)]
struct JumpsPtr {
    instr_set_idx: usize,
    offset: usize,
}

#[derive(Default)]
struct HereditaryJumpsPtr {
    stack_offset: usize,
    instr_set_idx: usize,
    offset: usize,
}

struct Bailout<T> {
    at_addr: usize,
    restore_point: T,
}

macro_rules! aux_info_request {
    ($handler:expr) => {
        Err(Box::new($handler))
    };
}

struct ExecutionCtx<'i, P>
where
    P: PartialEq + Eq + Copy + Debug + Hash + 'static,
{
    stack_item: StackItem<'i, P>,
    match_handler: Box<dyn FnMut(MatchInfo<P>)>,
    with_content: bool,
    ns: Namespace,
}

impl<'i, P> ExecutionCtx<'i, P>
where
    P: PartialEq + Eq + Copy + Debug + Hash + 'static,
{
    #[inline]
    pub fn new(
        local_name: LocalName<'i>,
        ns: Namespace,
        match_handler: impl FnMut(MatchInfo<P>) + 'static,
    ) -> Self {
        ExecutionCtx {
            stack_item: StackItem::new(local_name),
            match_handler: Box::new(match_handler),
            with_content: true,
            ns,
        }
    }

    pub fn add_execution_branch(&mut self, branch: &ExecutionBranch<P>) {
        for &payload in &branch.matched_payload {
            if !self.stack_item.matched_payload.contains(&payload) {
                self.report_match(payload);
                self.stack_item.matched_payload.insert(payload);
            }
        }

        if self.with_content {
            if let Some(ref jumps) = branch.jumps {
                self.stack_item.jumps.push(jumps.to_owned());
            }

            if let Some(ref hereditary_jumps) = branch.hereditary_jumps {
                self.stack_item
                    .hereditary_jumps
                    .push(hereditary_jumps.to_owned());
            }
        }
    }

    #[inline]
    pub fn into_owned(self) -> ExecutionCtx<'static, P> {
        ExecutionCtx {
            stack_item: self.stack_item.into_owned(),
            match_handler: self.match_handler,
            with_content: self.with_content,
            ns: self.ns,
        }
    }

    #[inline]
    fn report_match(&mut self, payload: P) {
        (self.match_handler)(MatchInfo {
            payload,
            with_content: self.with_content,
        });
    }
}

pub struct SelectorMatchingVm<P>
where
    P: PartialEq + Eq + Copy + Debug + Hash + 'static,
{
    program: Program<P>,
    stack: Stack<P>,
}

impl<P> SelectorMatchingVm<P>
where
    P: PartialEq + Eq + Copy + Debug + Hash + 'static,
{
    #[inline]
    pub fn new(ast: Ast<P>, encoding: &'static Encoding) -> Self {
        let program = Compiler::new(encoding).compile(ast);

        SelectorMatchingVm {
            program,
            stack: Stack::default(),
        }
    }

    pub fn exec_for_start_tag<'v>(
        &'v mut self,
        local_name: LocalName,
        ns: Namespace,
        match_handler: impl FnMut(MatchInfo<P>) + 'static,
    ) -> Result<(), AuxStartTagInfoRequest<'v>> {
        let mut ctx = ExecutionCtx::new(local_name, ns, match_handler);
        let stack_directive = self.stack.get_stack_directive(&ctx.stack_item, ns);

        if let StackDirective::PopImmediately = stack_directive {
            ctx.with_content = false;
        } else if let StackDirective::PushIfNotSelfClosing = stack_directive {
            let mut ctx = ctx.into_owned();

            return aux_info_request!(move |aux_info| {
                let attr_matcher = AttributeMatcher::new(aux_info.input, aux_info.attr_buffer, ns);

                ctx.with_content = !aux_info.self_closing;

                self.exec_instr_set_with_attrs(
                    &self.program.entry_points,
                    &attr_matcher,
                    &mut ctx,
                    0,
                );

                self.exec_jumps_with_attrs(&attr_matcher, &mut ctx, JumpsPtr::default());

                self.exec_hereditary_jumps_with_attrs(
                    &attr_matcher,
                    &mut ctx,
                    HereditaryJumpsPtr::default(),
                );

                if ctx.with_content {
                    self.stack.push_item(ctx.stack_item);
                }
            });
        }

        self.exec_without_attrs(ctx)
    }

    #[inline]
    pub fn exec_for_end_tag(&mut self, local_name: LocalName, unmatch_handler: impl FnMut(P)) {
        self.stack.pop_up_to(local_name, unmatch_handler);
    }

    #[inline]
    fn exec_jumps_with_attrs(
        &self,
        attr_matcher: &AttributeMatcher,
        ctx: &mut ExecutionCtx<P>,
        from_ptr: JumpsPtr,
    ) {
        if let Some(parent) = self.stack.items().last() {
            for i in from_ptr.instr_set_idx..parent.jumps.len() {
                self.exec_instr_set_with_attrs(
                    &parent.jumps[i],
                    attr_matcher,
                    ctx,
                    from_ptr.offset,
                );
            }
        }
    }

    #[inline]
    fn exec_hereditary_jumps_with_attrs(
        &self,
        attr_matcher: &AttributeMatcher,
        ctx: &mut ExecutionCtx<P>,
        from_ptr: HereditaryJumpsPtr,
    ) {
        for ancestor in self.stack.items().iter().rev().skip(from_ptr.stack_offset) {
            for i in from_ptr.instr_set_idx..ancestor.hereditary_jumps.len() {
                self.exec_instr_set_with_attrs(
                    &ancestor.hereditary_jumps[i],
                    attr_matcher,
                    ctx,
                    from_ptr.offset,
                );
            }

            if !ancestor.has_ancestor_with_hereditary_jumps {
                break;
            }
        }
    }

    #[inline]
    fn exec_instr_set_with_attrs(
        &self,
        addr_range: &AddressRange,
        attr_matcher: &AttributeMatcher,
        ctx: &mut ExecutionCtx<P>,
        offset: usize,
    ) {
        for addr in addr_range.start + offset..addr_range.end {
            let instr = &self.program.instructions[addr];

            if let Some(branch) = instr.exec(&ctx.stack_item.local_name, attr_matcher) {
                ctx.add_execution_branch(branch);
            }
        }
    }

    fn exec_without_attrs<'v>(
        &'v mut self,
        mut ctx: ExecutionCtx<P>,
    ) -> Result<(), AuxStartTagInfoRequest<'v>> {
        macro_rules! bailout {
            ($at_addr:expr, $restore_point_handler:expr) => {{
                let mut ctx = ctx.into_owned();

                return aux_info_request!(move |aux_info| {
                    let attr_matcher =
                        AttributeMatcher::new(aux_info.input, aux_info.attr_buffer, ctx.ns);

                    self.complete_instr_execution_with_attrs($at_addr, &attr_matcher, &mut ctx);

                    $restore_point_handler(&mut ctx, &attr_matcher);

                    if ctx.with_content {
                        self.stack.push_item(ctx.stack_item);
                    }
                });
            }};
        }

        if let Err(b) =
            self.try_exec_instr_set_without_attrs(self.program.entry_points.clone(), &mut ctx)
        {
            bailout!(b.at_addr, |ctx: &mut _, attr_matcher| {
                self.exec_instr_set_with_attrs(
                    &self.program.entry_points,
                    attr_matcher,
                    ctx,
                    b.restore_point,
                );

                self.exec_jumps_with_attrs(attr_matcher, ctx, JumpsPtr::default());

                self.exec_hereditary_jumps_with_attrs(
                    attr_matcher,
                    ctx,
                    HereditaryJumpsPtr::default(),
                );
            });
        }

        if let Err(b) = self.try_exec_jumps_without_attrs(&mut ctx) {
            bailout!(b.at_addr, |ctx: &mut _, attr_matcher| {
                self.exec_jumps_with_attrs(attr_matcher, ctx, b.restore_point);

                self.exec_hereditary_jumps_with_attrs(
                    attr_matcher,
                    ctx,
                    HereditaryJumpsPtr::default(),
                );
            });
        }

        if let Err(b) = self.try_exec_hereditary_jumps_without_attrs(&mut ctx) {
            bailout!(b.at_addr, |ctx: &mut _, attr_matcher| {
                self.exec_hereditary_jumps_with_attrs(attr_matcher, ctx, b.restore_point);
            });
        }

        if ctx.with_content {
            self.stack.push_item(ctx.stack_item.into_owned());
        }

        Ok(())
    }

    #[inline]
    fn complete_instr_execution_with_attrs(
        &self,
        addr: usize,
        attr_matcher: &AttributeMatcher,
        ctx: &mut ExecutionCtx<P>,
    ) {
        if let Some(branch) =
            self.program.instructions[addr].complete_execution_with_attrs(&attr_matcher)
        {
            ctx.add_execution_branch(branch);
        }
    }

    #[inline]
    fn try_exec_jumps_without_attrs(
        &self,
        ctx: &mut ExecutionCtx<P>,
    ) -> Result<(), Bailout<JumpsPtr>> {
        if let Some(parent) = self.stack.items().last() {
            for (i, jumps) in parent.jumps.iter().enumerate() {
                self.try_exec_instr_set_without_attrs(jumps.clone(), ctx)
                    .map_err(|b| Bailout {
                        at_addr: b.at_addr,
                        restore_point: JumpsPtr {
                            instr_set_idx: i,
                            offset: b.restore_point,
                        },
                    })?;
            }
        }

        Ok(())
    }

    #[inline]
    fn try_exec_hereditary_jumps_without_attrs(
        &self,
        ctx: &mut ExecutionCtx<P>,
    ) -> Result<(), Bailout<HereditaryJumpsPtr>> {
        for (i, ancestor) in self.stack.items().iter().rev().enumerate() {
            for (j, jumps) in ancestor.hereditary_jumps.iter().enumerate() {
                self.try_exec_instr_set_without_attrs(jumps.clone(), ctx)
                    .map_err(|b| Bailout {
                        at_addr: b.at_addr,
                        restore_point: HereditaryJumpsPtr {
                            stack_offset: i,
                            instr_set_idx: j,
                            offset: b.restore_point,
                        },
                    })?;
            }

            if !ancestor.has_ancestor_with_hereditary_jumps {
                break;
            }
        }

        Ok(())
    }

    #[inline]
    fn try_exec_instr_set_without_attrs(
        &self,
        addr_range: AddressRange,
        ctx: &mut ExecutionCtx<P>,
    ) -> Result<(), Bailout<usize>> {
        for addr in addr_range {
            let instr = &self.program.instructions[addr];
            let result = instr.try_exec_without_attrs(&ctx.stack_item.local_name);

            if let Ok(Some(branch)) = result {
                ctx.add_execution_branch(branch)
            } else if result.is_err() {
                return Err(Bailout {
                    at_addr: addr,
                    restore_point: addr + 1,
                });
            }
        }

        Ok(())
    }
}
