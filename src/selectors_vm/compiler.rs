use super::attribute_matcher::{is_attr_whitespace, AttributeMatcher};
use super::{Ast, AttributeExpr, AttributeExprOperand, NonAttributeExpr, Program};
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
    program: Program<P>,
}

impl<P> Compiler<P>
where
    P: PartialEq + Eq + Copy + Debug,
{
    pub fn new(encoding: &'static Encoding) -> Self {
        Compiler {
            encoding,
            program: Program::default(),
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

    pub fn compile(self, ast: Ast<P>) -> Program<P> {
        self.program
    }
}
