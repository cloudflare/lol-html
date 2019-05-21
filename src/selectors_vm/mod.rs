mod ast;
mod attribute_matcher;
mod compiler;
mod error;
mod parser;
mod program;
mod stack;

use self::program::AddressRange;
use self::stack::StackDirective;
use crate::html::{LocalName, Namespace};
use crate::transform_stream::AuxStartTagInfo;
use encoding_rs::Encoding;

pub use self::ast::*;
pub use self::attribute_matcher::AttributeMatcher;
pub use self::compiler::Compiler;
pub use self::error::SelectorError;
pub use self::parser::SelectorsParser;
pub use self::program::{ExecutionBranch, Program};
pub use self::stack::{ElementData, Stack, StackItem};

pub struct MatchInfo<P> {
    pub payload: P,
    pub with_content: bool,
}

pub type AuxStartTagInfoRequest<E, P> =
    Box<dyn FnMut(&mut SelectorMatchingVm<E>, AuxStartTagInfo, &mut dyn FnMut(MatchInfo<P>))>;

type RecoveryPointHandler<T, E, P> = fn(
    &mut SelectorMatchingVm<E>,
    &mut ExecutionCtx<'static, E>,
    &AttributeMatcher,
    T,
    &mut dyn FnMut(MatchInfo<P>),
);

#[derive(Default)]
struct JumpPtr {
    instr_set_idx: usize,
    offset: usize,
}

#[derive(Default)]
struct HereditaryJumpPtr {
    stack_offset: usize,
    instr_set_idx: usize,
    offset: usize,
}

struct Bailout<T> {
    at_addr: usize,
    recovery_point: T,
}

struct ExecutionCtx<'i, E: ElementData> {
    stack_item: StackItem<'i, E>,
    with_content: bool,
    ns: Namespace,
}

impl<'i, E: ElementData> ExecutionCtx<'i, E> {
    #[inline]
    pub fn new(local_name: LocalName<'i>, ns: Namespace) -> Self {
        ExecutionCtx {
            stack_item: StackItem::new(local_name),
            with_content: true,
            ns,
        }
    }

    pub fn add_execution_branch(
        &mut self,
        branch: &ExecutionBranch<E::MatchPayload>,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) {
        for &payload in branch.matched_payload.iter() {
            let element_payload = self.stack_item.element_data.matched_payload_mut();

            if !element_payload.contains(&payload) {
                match_handler(MatchInfo {
                    payload,
                    with_content: self.with_content,
                });

                element_payload.insert(payload);
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
    pub fn into_owned(self) -> ExecutionCtx<'static, E> {
        ExecutionCtx {
            stack_item: self.stack_item.into_owned(),
            with_content: self.with_content,
            ns: self.ns,
        }
    }
}

pub struct SelectorMatchingVm<E: ElementData> {
    program: Program<E::MatchPayload>,
    stack: Stack<E>,
}

impl<E: ElementData> SelectorMatchingVm<E> {
    #[inline]
    pub fn new(ast: &Ast<E::MatchPayload>, encoding: &'static Encoding) -> Self {
        let program = Compiler::new(encoding).compile(ast);

        SelectorMatchingVm {
            program,
            stack: Stack::default(),
        }
    }

    pub fn exec_for_start_tag(
        &mut self,
        local_name: LocalName,
        ns: Namespace,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) -> Result<(), AuxStartTagInfoRequest<E, E::MatchPayload>> {
        let mut ctx = ExecutionCtx::new(local_name, ns);
        let stack_directive = self.stack.get_stack_directive(&ctx.stack_item, ns);

        if let StackDirective::PopImmediately = stack_directive {
            ctx.with_content = false;
        } else if let StackDirective::PushIfNotSelfClosing = stack_directive {
            let mut ctx = ctx.into_owned();

            return Self::aux_info_request(move |this, aux_info, match_handler| {
                let attr_matcher = AttributeMatcher::new(aux_info.input, aux_info.attr_buffer, ns);

                ctx.with_content = !aux_info.self_closing;

                this.exec_instr_set_with_attrs(
                    &this.program.entry_points,
                    &attr_matcher,
                    &mut ctx,
                    0,
                    match_handler,
                );

                this.exec_jumps_with_attrs(
                    &attr_matcher,
                    &mut ctx,
                    JumpPtr::default(),
                    match_handler,
                );

                this.exec_hereditary_jumps_with_attrs(
                    &attr_matcher,
                    &mut ctx,
                    HereditaryJumpPtr::default(),
                    match_handler,
                );

                if ctx.with_content {
                    this.stack.push_item(ctx.stack_item);
                }
            });
        }

        self.exec_without_attrs(ctx, match_handler)
    }

    #[inline]
    pub fn exec_for_end_tag(
        &mut self,
        local_name: LocalName,
        unmatched_element_data_handler: impl FnMut(E),
    ) {
        self.stack
            .pop_up_to(local_name, unmatched_element_data_handler);
    }

    #[inline]
    pub fn current_element_data_mut(&mut self) -> Option<&mut E> {
        self.stack.current_element_data_mut()
    }

    #[inline]
    fn aux_info_request(
        req: impl FnOnce(
                &mut SelectorMatchingVm<E>,
                AuxStartTagInfo,
                &mut dyn FnMut(MatchInfo<E::MatchPayload>),
            ) + 'static,
    ) -> Result<(), AuxStartTagInfoRequest<E, E::MatchPayload>> {
        // TODO: remove this hack when Box<dyn FnOnce> become callable in Rust 1.35.
        let mut wrap = Some(req);

        Err(Box::new(move |this, aux_info, match_handler| {
            (wrap.take().expect("FnOnce called more than once"))(this, aux_info, match_handler)
        }))
    }

    fn bailout<T: 'static>(
        ctx: ExecutionCtx<E>,
        bailout: Bailout<T>,
        recovery_point_handler: RecoveryPointHandler<T, E, E::MatchPayload>,
    ) -> Result<(), AuxStartTagInfoRequest<E, E::MatchPayload>> {
        let mut ctx = ctx.into_owned();

        Self::aux_info_request(move |this, aux_info, match_handler| {
            let attr_matcher = AttributeMatcher::new(aux_info.input, aux_info.attr_buffer, ctx.ns);

            this.complete_instr_execution_with_attrs(
                bailout.at_addr,
                &attr_matcher,
                &mut ctx,
                match_handler,
            );

            recovery_point_handler(
                this,
                &mut ctx,
                &attr_matcher,
                bailout.recovery_point,
                match_handler,
            );

            if ctx.with_content {
                this.stack.push_item(ctx.stack_item);
            }
        })
    }

    fn recover_after_bailout_in_entry_points(
        &mut self,
        ctx: &mut ExecutionCtx<'static, E>,
        attr_matcher: &AttributeMatcher,
        recovery_point: usize,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) {
        self.exec_instr_set_with_attrs(
            &self.program.entry_points,
            attr_matcher,
            ctx,
            recovery_point,
            match_handler,
        );

        self.exec_jumps_with_attrs(attr_matcher, ctx, JumpPtr::default(), match_handler);

        self.exec_hereditary_jumps_with_attrs(
            attr_matcher,
            ctx,
            HereditaryJumpPtr::default(),
            match_handler,
        );
    }

    fn recover_after_bailout_in_jumps(
        &mut self,
        ctx: &mut ExecutionCtx<'static, E>,
        attr_matcher: &AttributeMatcher,
        recovery_point: JumpPtr,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) {
        self.exec_jumps_with_attrs(attr_matcher, ctx, recovery_point, match_handler);

        self.exec_hereditary_jumps_with_attrs(
            attr_matcher,
            ctx,
            HereditaryJumpPtr::default(),
            match_handler,
        );
    }

    fn recover_after_bailout_in_hereditary_jumps(
        &mut self,
        ctx: &mut ExecutionCtx<'static, E>,
        attr_matcher: &AttributeMatcher,
        recovery_point: HereditaryJumpPtr,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) {
        self.exec_hereditary_jumps_with_attrs(attr_matcher, ctx, recovery_point, match_handler);
    }

    fn exec_without_attrs(
        &mut self,
        mut ctx: ExecutionCtx<E>,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) -> Result<(), AuxStartTagInfoRequest<E, E::MatchPayload>> {
        if let Err(b) = self.try_exec_instr_set_without_attrs(
            self.program.entry_points.clone(),
            &mut ctx,
            match_handler,
        ) {
            return Self::bailout(ctx, b, Self::recover_after_bailout_in_entry_points);
        }

        if let Err(b) = self.try_exec_jumps_without_attrs(&mut ctx, match_handler) {
            return Self::bailout(ctx, b, Self::recover_after_bailout_in_jumps);
        }

        if let Err(b) = self.try_exec_hereditary_jumps_without_attrs(&mut ctx, match_handler) {
            return Self::bailout(ctx, b, Self::recover_after_bailout_in_hereditary_jumps);
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
        ctx: &mut ExecutionCtx<E>,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) {
        if let Some(branch) =
            self.program.instructions[addr].complete_execution_with_attrs(&attr_matcher)
        {
            ctx.add_execution_branch(branch, match_handler);
        }
    }

    #[inline]
    fn try_exec_instr_set_without_attrs(
        &self,
        addr_range: AddressRange,
        ctx: &mut ExecutionCtx<E>,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) -> Result<(), Bailout<usize>> {
        let start = addr_range.start;

        for addr in addr_range {
            let instr = &self.program.instructions[addr];
            let result = instr.try_exec_without_attrs(&ctx.stack_item.local_name);

            if let Ok(Some(branch)) = result {
                ctx.add_execution_branch(branch, match_handler)
            } else if result.is_err() {
                return Err(Bailout {
                    at_addr: addr,
                    recovery_point: addr - start + 1,
                });
            }
        }

        Ok(())
    }

    #[inline]
    fn exec_instr_set_with_attrs(
        &self,
        addr_range: &AddressRange,
        attr_matcher: &AttributeMatcher,
        ctx: &mut ExecutionCtx<E>,
        offset: usize,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) {
        for addr in addr_range.start + offset..addr_range.end {
            let instr = &self.program.instructions[addr];

            if let Some(branch) = instr.exec(&ctx.stack_item.local_name, attr_matcher) {
                ctx.add_execution_branch(branch, match_handler);
            }
        }
    }

    fn try_exec_jumps_without_attrs(
        &self,
        ctx: &mut ExecutionCtx<E>,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) -> Result<(), Bailout<JumpPtr>> {
        if let Some(parent) = self.stack.items().last() {
            for (i, jumps) in parent.jumps.iter().enumerate() {
                self.try_exec_instr_set_without_attrs(jumps.clone(), ctx, match_handler)
                    .map_err(|b| Bailout {
                        at_addr: b.at_addr,
                        recovery_point: JumpPtr {
                            instr_set_idx: i,
                            offset: b.recovery_point,
                        },
                    })?;
            }
        }

        Ok(())
    }

    fn exec_jumps_with_attrs(
        &self,
        attr_matcher: &AttributeMatcher,
        ctx: &mut ExecutionCtx<E>,
        ptr: JumpPtr,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) {
        // NOTE: find pointed jumps instruction set and execute it with the offset.
        if let Some(parent) = self.stack.items().last() {
            if let Some(ptr_jumps) = parent.jumps.get(ptr.instr_set_idx) {
                self.exec_instr_set_with_attrs(
                    ptr_jumps,
                    attr_matcher,
                    ctx,
                    ptr.offset,
                    match_handler,
                );

                // NOTE: execute remaining jumps instruction sets as usual.
                for jumps in parent.jumps.iter().skip(ptr.instr_set_idx + 1) {
                    self.exec_instr_set_with_attrs(jumps, attr_matcher, ctx, 0, match_handler);
                }
            }
        }
    }

    fn try_exec_hereditary_jumps_without_attrs(
        &self,
        ctx: &mut ExecutionCtx<E>,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) -> Result<(), Bailout<HereditaryJumpPtr>> {
        for (i, ancestor) in self.stack.items().iter().rev().enumerate() {
            for (j, jumps) in ancestor.hereditary_jumps.iter().cloned().enumerate() {
                self.try_exec_instr_set_without_attrs(jumps, ctx, match_handler)
                    .map_err(|b| Bailout {
                        at_addr: b.at_addr,
                        recovery_point: HereditaryJumpPtr {
                            stack_offset: i,
                            instr_set_idx: j,
                            offset: b.recovery_point,
                        },
                    })?;
            }

            if !ancestor.has_ancestor_with_hereditary_jumps {
                break;
            }
        }

        Ok(())
    }

    fn exec_hereditary_jumps_with_attrs(
        &self,
        attr_matcher: &AttributeMatcher,
        ctx: &mut ExecutionCtx<E>,
        ptr: HereditaryJumpPtr,
        match_handler: &mut dyn FnMut(MatchInfo<E::MatchPayload>),
    ) {
        let items = self.stack.items();

        if items.is_empty() {
            return;
        }

        let ptr_ancestor_idx = items.len() - 1 - ptr.stack_offset;

        // NOTE: first find pointed ancestor, then jump instruction
        // set and execute it with the offset.
        if let Some(ptr_ancestor) = items.get(ptr_ancestor_idx) {
            if let Some(ptr_jumps) = ptr_ancestor.hereditary_jumps.get(ptr.instr_set_idx) {
                self.exec_instr_set_with_attrs(
                    ptr_jumps,
                    attr_matcher,
                    ctx,
                    ptr.offset,
                    match_handler,
                );

                // NOTE: execute the rest of jump instruction sets in the pointed ancestor as usual.
                for jumps in ptr_ancestor
                    .hereditary_jumps
                    .iter()
                    .skip(ptr.instr_set_idx + 1)
                {
                    self.exec_instr_set_with_attrs(jumps, attr_matcher, ctx, 0, match_handler);
                }
            }

            // NOTE: execute hereditary jumps in remaining ancestors as usual.
            if ptr_ancestor.has_ancestor_with_hereditary_jumps {
                for ancestor in items.iter().rev().skip(ptr.stack_offset + 1) {
                    for jumps in &ancestor.hereditary_jumps {
                        self.exec_instr_set_with_attrs(jumps, attr_matcher, ctx, 0, match_handler);
                    }

                    if !ancestor.has_ancestor_with_hereditary_jumps {
                        break;
                    }
                }
            }
        }
    }
}
