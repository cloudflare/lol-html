use super::parse::{SelectorImplDescriptor, SelectorsParser};
use super::SelectorError;
use selectors::attr::{AttrSelectorOperator, ParsedCaseSensitivity};
use selectors::parser::{Combinator, Component, Selector};
use std::fmt::Debug;

type AstNodeVec<P> = Vec<AstNode<P>>;

#[derive(Eq, PartialEq, Debug)]
pub struct AttributeExprOperand {
    pub name: String,
    pub value: String,
    pub case_sensitivity: ParsedCaseSensitivity,
}

#[derive(PartialEq, Eq, Debug)]
pub enum SimpleExpr {
    ExplicitAny,
    LocalName(String),
    Id(String),
    Class(String),
    AttributeExists(String),
    AttributeEqual(AttributeExprOperand),
    AttributeIncludes(AttributeExprOperand),
    AttributeDashMatch(AttributeExprOperand),
    AttributePrefix(AttributeExprOperand),
    AttributeSubstring(AttributeExprOperand),
    AttributeSuffix(AttributeExprOperand),
    Unmatchable,
}

impl SimpleExpr {
    #[inline]
    fn attr_expr_for_operator(
        operator: AttrSelectorOperator,
        name: &str,
        value: &str,
        case_sensitivity: ParsedCaseSensitivity,
    ) -> Self {
        use AttrSelectorOperator::*;

        let operand = AttributeExprOperand {
            name: name.to_owned(),
            value: value.to_owned(),
            case_sensitivity,
        };

        match operator {
            DashMatch => SimpleExpr::AttributeDashMatch(operand),
            Equal => SimpleExpr::AttributeEqual(operand),
            Includes => SimpleExpr::AttributeIncludes(operand),
            Prefix => SimpleExpr::AttributePrefix(operand),
            Substring => SimpleExpr::AttributeSubstring(operand),
            Suffix => SimpleExpr::AttributeSuffix(operand),
        }
    }
}

impl From<&Component<SelectorImplDescriptor>> for SimpleExpr {
    #[inline]
    fn from(component: &Component<SelectorImplDescriptor>) -> Self {
        match component {
            Component::LocalName(n) => SimpleExpr::LocalName(n.name.to_owned()),
            Component::ExplicitUniversalType | Component::ExplicitAnyNamespace => {
                SimpleExpr::ExplicitAny
            }
            Component::ExplicitNoNamespace => SimpleExpr::Unmatchable,
            Component::ID(id) => SimpleExpr::Id(id.to_owned()),
            Component::Class(c) => SimpleExpr::Class(c.to_owned()),
            Component::AttributeInNoNamespaceExists { local_name, .. } => {
                SimpleExpr::AttributeExists(local_name.to_owned())
            }
            &Component::AttributeInNoNamespace {
                ref local_name,
                ref value,
                operator,
                case_sensitivity,
                never_matches,
            } => {
                if never_matches {
                    SimpleExpr::Unmatchable
                } else {
                    Self::attr_expr_for_operator(operator, local_name, value, case_sensitivity)
                }
            }
            // NOTE: the rest of the components are explicit namespace or
            // pseudo class-related. Ideally none of them should appear in
            // the parsed selector as we should bail earlier in the parser.
            // Otherwise, we'll have AST in invalid state in case of error.
            _ => unreachable!(
                "Unsupported selector components should be filtered out by the parser."
            ),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Expr {
    Simple(SimpleExpr),
    Negation(Vec<SimpleExpr>),
}

impl From<&Component<SelectorImplDescriptor>> for Expr {
    #[inline]
    fn from(component: &Component<SelectorImplDescriptor>) -> Self {
        match component {
            Component::Negation(e) => Expr::Negation(e.iter().map(SimpleExpr::from).collect()),
            _ => Expr::Simple(component.into()),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct AstNode<P> {
    pub expressions: Vec<Expr>,
    pub children: Option<AstNodeVec<P>>,
    pub descendants: Option<AstNodeVec<P>>,
    pub payload: Option<Vec<P>>,
}

impl<P> AstNode<P> {
    fn new(expressions: Vec<Expr>) -> Self {
        AstNode {
            expressions,
            children: None,
            descendants: None,
            payload: None,
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
pub struct Ast<P>(pub AstNodeVec<P>)
where
    P: PartialEq + Eq + Copy + Debug;

impl<P> Ast<P>
where
    P: PartialEq + Eq + Copy + Debug,
{
    #[inline]
    pub fn add_selector(&mut self, selector: &str, payload: P) -> Result<(), SelectorError> {
        SelectorsParser::parse(selector)?
            .0
            .into_iter()
            .for_each(|s| self.add_parsed_selector(s, payload));

        Ok(())
    }

    #[inline]
    fn host_expressions(expressions: Vec<Expr>, branches: &mut AstNodeVec<P>) -> usize {
        match branches
            .iter()
            .enumerate()
            .find(|(_, n)| n.expressions == expressions)
        {
            Some((i, _)) => i,
            None => {
                branches.push(AstNode::new(expressions));

                branches.len() - 1
            }
        }
    }

    fn add_parsed_selector(&mut self, selector: Selector<SelectorImplDescriptor>, payload: P) {
        let mut expressions = Vec::default();
        let mut branches = &mut self.0;

        macro_rules! host_and_switch_branch_vec {
            ($branches:ident) => {{
                let node_idx = Self::host_expressions(expressions, branches);
                branches = branches[node_idx]
                    .$branches
                    .get_or_insert_with(Vec::default);
                expressions = Vec::default();
            }};
        }

        for component in selector.iter_raw_parse_order_from(0) {
            match component {
                Component::Combinator(c) => match c {
                    Combinator::Child => host_and_switch_branch_vec!(children),
                    Combinator::Descendant => host_and_switch_branch_vec!(descendants),
                    _ => unreachable!(
                        "Unsupported selector components should be filtered out by the parser."
                    ),
                },
                _ => expressions.push(component.into()),
            }
        }

        let node_idx = Self::host_expressions(expressions, branches);

        branches[node_idx]
            .payload
            .get_or_insert_with(Vec::default)
            .push(payload);
    }
}
