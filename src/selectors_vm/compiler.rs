use super::attribute_matcher::{is_attr_whitespace, AttributeMatcher};
use super::{
    AddressRange, Ast, AstNode, AttributeExpr, AttributeExprOperand, Instr, NonAttributeExpr,
    Predicate, Program, ThreadState,
};
use crate::base::Bytes;
use crate::html::LocalName;
use encoding_rs::Encoding;
use selectors::attr::ParsedCaseSensitivity;
use std::fmt::Debug;

type CompiledNonAttributeExpr = Box<dyn Fn(LocalName<'_>) -> bool>;
type CompiledAttributeExpr = Box<dyn Fn(&AttributeMatcher<'_>) -> bool>;

pub struct CompiledAttributeExprOperand {
    pub name: Bytes<'static>,
    pub value: Bytes<'static>,
    pub case_sensitivity: ParsedCaseSensitivity,
}

macro_rules! unmatchable {
    () => {
        Box::new(|_| false)
    };
}

trait MapOrUnmatchable<T> {
    fn map_or_unmatchable(self, map: impl Fn(T) -> CompiledAttributeExpr) -> CompiledAttributeExpr;
}

impl<T> MapOrUnmatchable<T> for Result<T, ()> {
    #[inline]
    fn map_or_unmatchable(self, map: impl Fn(T) -> CompiledAttributeExpr) -> CompiledAttributeExpr {
        match self {
            Ok(v) => map(v),
            Err(_) => unmatchable!(),
        }
    }
}

pub struct Compiler<P>
where
    P: PartialEq + Eq + Copy + Debug,
{
    encoding: &'static Encoding,
    instructions: Vec<Instr<P>>,
    free_space_ptr: usize,
}

impl<P> Compiler<P>
where
    P: PartialEq + Eq + Copy + Debug,
{
    pub fn new(encoding: &'static Encoding) -> Self {
        Compiler {
            encoding,
            instructions: Vec::default(),
            free_space_ptr: 0,
        }
    }

    fn compile_non_attr_expr(&self, expr: NonAttributeExpr) -> CompiledNonAttributeExpr {
        match expr {
            NonAttributeExpr::ExplicitAny => Box::new(|_| true),
            NonAttributeExpr::Unmatchable => unmatchable!(),
            NonAttributeExpr::LocalName(local_name) => {
                match LocalName::from_str_without_replacements(&local_name, self.encoding)
                    .map(|n| n.into_owned())
                {
                    Ok(local_name) => {
                        Box::new(move |actual_local_name| actual_local_name == local_name)
                    }
                    // NOTE: selector value can't be converted to the given encoding, so
                    // it won't ever match.
                    Err(_) => unmatchable!(),
                }
            }
        }
    }

    #[inline]
    fn compile_literal(&self, lit: &str) -> Result<Bytes<'static>, ()> {
        Bytes::from_str_without_replacements(lit, self.encoding).map(|b| b.into_owned())
    }

    #[inline]
    fn compile_literal_lowercase(&self, mut lit: String) -> Result<Bytes<'static>, ()> {
        lit.make_ascii_lowercase();

        self.compile_literal(&lit)
    }

    #[inline]
    fn compile_attr_expr_operand(
        &self,
        operand: AttributeExprOperand,
    ) -> Result<CompiledAttributeExprOperand, ()> {
        Ok(CompiledAttributeExprOperand {
            name: self.compile_literal_lowercase(operand.name)?,
            value: self.compile_literal(&operand.value)?,
            case_sensitivity: operand.case_sensitivity,
        })
    }

    fn compile_attr_expr(&self, expr: AttributeExpr) -> CompiledAttributeExpr {
        match expr {
            AttributeExpr::Id(id) => self
                .compile_literal(&id)
                .map_or_unmatchable(|id| Box::new(move |m| m.id_matches(&id))),

            AttributeExpr::Class(class) => self
                .compile_literal(&class)
                .map_or_unmatchable(|class| Box::new(move |m| m.has_class(&class))),

            AttributeExpr::AttributeExists(name) => self
                .compile_literal_lowercase(name)
                .map_or_unmatchable(|name| Box::new(move |m| m.has_attribute(&name))),

            AttributeExpr::AttributeEqual(operand) => self
                .compile_attr_expr_operand(operand)
                .map_or_unmatchable(|operand| Box::new(move |m| m.attr_eq(&operand))),

            AttributeExpr::AttributeIncludes(operand) => self
                .compile_attr_expr_operand(operand)
                .map_or_unmatchable(|operand| {
                    Box::new(move |m| m.matches_splitted_by(&operand, is_attr_whitespace))
                }),

            AttributeExpr::AttributeDashMatch(operand) => self
                .compile_attr_expr_operand(operand)
                .map_or_unmatchable(|operand| {
                    Box::new(move |m| m.matches_splitted_by(&operand, |b| b == b'-'))
                }),

            AttributeExpr::AttributePrefix(operand) => self
                .compile_attr_expr_operand(operand)
                .map_or_unmatchable(|operand| Box::new(move |m| m.has_attr_with_prefix(&operand))),

            AttributeExpr::AttributeSuffix(operand) => self
                .compile_attr_expr_operand(operand)
                .map_or_unmatchable(|operand| Box::new(move |m| m.has_attr_with_suffix(&operand))),

            AttributeExpr::AttributeSubstring(operand) => self
                .compile_attr_expr_operand(operand)
                .map_or_unmatchable(|operand| {
                    Box::new(move |m| m.has_attr_with_substring(&operand))
                }),
        }
    }

    fn compile_predicate(
        &self,
        predicate: Predicate,
        associated_thread_state: ThreadState<P>,
    ) -> Instr<P> {
        unimplemented!();
    }

    #[inline]
    fn reserve_space_for_nodes(&mut self, nodes: &[AstNode<P>]) -> AddressRange {
        let address_range = self.free_space_ptr..nodes.len();

        self.free_space_ptr = address_range.end;

        debug_assert!(self.free_space_ptr < self.instructions.len());

        address_range
    }

    fn compile_nodes(&mut self, nodes: Vec<AstNode<P>>) -> AddressRange {
        // NOTE: we need sibling nodes to be in a contiguous region, so
        // we can reference them by range instead of vector of addresses.
        let address_range = self.reserve_space_for_nodes(&nodes);

        for (i, node) in nodes.into_iter().enumerate() {
            let associated_thread_state = ThreadState {
                matched_payload: node.payload,
                jumps: node.children.map(|c| self.compile_nodes(c)),
                hereditary_jumps: node.descendants.map(|d| self.compile_nodes(d)),
            };

            self.instructions[address_range.start + i] =
                self.compile_predicate(node.predicate, associated_thread_state);
        }

        address_range
    }

    pub fn compile(mut self, ast: Ast<P>) -> Program<P> {
        self.instructions
            .resize_with(ast.cumulative_node_count, || {
                Box::new(|_| unreachable!("Instruction stub should never be executed"))
            });

        let entry_points = self.compile_nodes(ast.root);

        Program {
            instructions: self.instructions,
            entry_points,
        }
    }
}
