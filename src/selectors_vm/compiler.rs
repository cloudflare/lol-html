use super::attribute_matcher::AttributeMatcher;
use super::program::{
    AddressRange, AttrExprMatchingInstr, ExecutionBranch, GenericInstr, Instr, InstrStub,
    NonAttrExprMatchingInstr, Program,
};
use super::{Ast, AstNode, AttributeExpr, AttributeExprOperand, NonAttributeExpr, Predicate};
use crate::base::Bytes;
use crate::html::LocalName;
use encoding_rs::Encoding;
use selectors::attr::ParsedCaseSensitivity;
use std::fmt::Debug;

pub type CompiledNonAttributeExpr = Box<dyn Fn(&LocalName) -> bool>;
pub type CompiledAttributeExpr = Box<dyn Fn(&AttributeMatcher) -> bool>;

pub struct CompiledAttributeExprOperand {
    pub name: Bytes<'static>,
    pub value: Bytes<'static>,
    pub case_sensitivity: ParsedCaseSensitivity,
}

macro_rules! curry_compile_expr_macro {
    ($negation:ident) => {
        macro_rules! compile_expr {
            (|$arg:ident| $body:expr) => {
                if $negation {
                    Box::new(move |$arg| !($body))
                } else {
                    Box::new(move |$arg| $body)
                }
            };

            (@unmatchable) => {
                compile_expr!(|_arg| false);
            };

            (@match_all) => {
                compile_expr!(|_arg| true);
            };
        }
    };
}

trait CompileOr<T> {
    fn compile_or(
        self,
        negation: bool,
        compile: impl Fn(T) -> CompiledAttributeExpr,
    ) -> CompiledAttributeExpr;
}

impl<T> CompileOr<T> for Result<T, ()> {
    #[inline]
    fn compile_or(
        self,
        negation: bool,
        compile: impl Fn(T) -> CompiledAttributeExpr,
    ) -> CompiledAttributeExpr {
        curry_compile_expr_macro!(negation);

        match self {
            Ok(v) => compile(v),
            Err(_) => compile_expr!(@unmatchable),
        }
    }
}

pub struct Compiler<P>
where
    P: PartialEq + Eq + Copy + Debug,
{
    encoding: &'static Encoding,
    instructions: Vec<Box<dyn Instr<P>>>,
    free_space_ptr: usize,
}

impl<P: 'static> Compiler<P>
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

    fn compile_non_attr_expr(
        &self,
        expr: NonAttributeExpr,
        negation: bool,
    ) -> CompiledNonAttributeExpr {
        curry_compile_expr_macro!(negation);

        match expr {
            NonAttributeExpr::ExplicitAny => compile_expr!(@match_all),
            NonAttributeExpr::Unmatchable => compile_expr!(@unmatchable),
            NonAttributeExpr::LocalName(local_name) => {
                match LocalName::from_str_without_replacements(&local_name, self.encoding)
                    .map(|n| n.into_owned())
                {
                    Ok(local_name) => {
                        compile_expr!(|actual_local_name| *actual_local_name == local_name)
                    }
                    // NOTE: selector value can't be converted to the given encoding, so
                    // it won't ever match.
                    Err(_) => compile_expr!(@unmatchable),
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

    fn compile_attr_expr(&self, expr: AttributeExpr, negation: bool) -> CompiledAttributeExpr {
        curry_compile_expr_macro!(negation);

        match expr {
            AttributeExpr::Id(id) => self
                .compile_literal(&id)
                .compile_or(negation, |id| compile_expr!(|m| m.id_matches(&id))),

            AttributeExpr::Class(class) => self
                .compile_literal(&class)
                .compile_or(negation, |class| compile_expr!(|m| m.has_class(&class))),

            AttributeExpr::AttributeExists(name) => self
                .compile_literal_lowercase(name)
                .compile_or(negation, |name| compile_expr!(|m| m.has_attribute(&name))),

            AttributeExpr::AttributeEqual(operand) => self
                .compile_attr_expr_operand(operand)
                .compile_or(negation, |operand| compile_expr!(|m| m.attr_eq(&operand))),

            AttributeExpr::AttributeIncludes(operand) => self
                .compile_attr_expr_operand(operand)
                .compile_or(negation, |operand| {
                    compile_expr!(|m| m.matches_splitted_by_whitespace(&operand))
                }),

            AttributeExpr::AttributeDashMatch(operand) => self
                .compile_attr_expr_operand(operand)
                .compile_or(negation, |operand| {
                    compile_expr!(|m| m.has_dash_matching_attr(&operand))
                }),

            AttributeExpr::AttributePrefix(operand) => self
                .compile_attr_expr_operand(operand)
                .compile_or(negation, |operand| {
                    compile_expr!(|m| m.has_attr_with_prefix(&operand))
                }),

            AttributeExpr::AttributeSuffix(operand) => self
                .compile_attr_expr_operand(operand)
                .compile_or(negation, |operand| {
                    compile_expr!(|m| m.has_attr_with_suffix(&operand))
                }),

            AttributeExpr::AttributeSubstring(operand) => self
                .compile_attr_expr_operand(operand)
                .compile_or(negation, |operand| {
                    compile_expr!(|m| m.has_attr_with_substring(&operand))
                }),
        }
    }

    fn compile_predicate(
        &self,
        predicate: Predicate,
        branch: ExecutionBranch<P>,
    ) -> Box<dyn Instr<P>> {
        let non_attr_exprs = predicate.non_attr_exprs.map(|e| {
            e.into_iter()
                .map(|expr| self.compile_non_attr_expr(expr.simple_expr, expr.negation))
                .collect::<Vec<_>>()
        });

        let attr_exprs = predicate.attr_exprs.map(|e| {
            e.into_iter()
                .map(|expr| self.compile_attr_expr(expr.simple_expr, expr.negation))
                .collect::<Vec<_>>()
        });

        match (non_attr_exprs, attr_exprs) {
            (Some(non_attr_exprs), None) => {
                NonAttrExprMatchingInstr::new_boxed(branch, non_attr_exprs)
            }
            (None, Some(attr_exprs)) => AttrExprMatchingInstr::new_boxed(branch, attr_exprs),
            (Some(non_attr_exprs), Some(attr_exprs)) => {
                GenericInstr::new_boxed(branch, non_attr_exprs, attr_exprs)
            }
            _ => unreachable!("Predicate should contain expressions"),
        }
    }

    #[inline]
    fn reserve_space_for_nodes(&mut self, nodes: &[AstNode<P>]) -> AddressRange {
        let address_range = self.free_space_ptr..self.free_space_ptr + nodes.len();

        self.free_space_ptr = address_range.end;

        debug_assert!(self.free_space_ptr <= self.instructions.len());

        address_range
    }

    fn compile_nodes(&mut self, nodes: Vec<AstNode<P>>) -> AddressRange {
        // NOTE: we need sibling nodes to be in a contiguous region, so
        // we can reference them by range instead of vector of addresses.
        let address_range = self.reserve_space_for_nodes(&nodes);

        for (i, node) in nodes.into_iter().enumerate() {
            let branch = ExecutionBranch {
                matched_payload: node.payload,
                jumps: node.children.map(|c| self.compile_nodes(c)),
                hereditary_jumps: node.descendants.map(|d| self.compile_nodes(d)),
            };

            self.instructions[address_range.start + i] =
                self.compile_predicate(node.predicate, branch);
        }

        address_range
    }

    pub fn compile(mut self, ast: Ast<P>) -> Program<P> {
        self.instructions
            .resize_with(ast.cumulative_node_count, || InstrStub::new_boxed());

        let entry_points = self.compile_nodes(ast.root);

        Program {
            instructions: self.instructions,
            entry_points,
        }
    }
}
