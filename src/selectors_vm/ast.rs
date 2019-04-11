use super::parse::{parse_selector, SelectorImplDescriptor};
use super::SelectorError;
use crate::html::Namespace;
use selectors::attr::{AttrSelectorOperator, ParsedCaseSensitivity};
use selectors::parser::{Combinator, Component, Selector};

type AstNodeVec<P> = Vec<Box<AstNode<P>>>;

#[derive(Eq, PartialEq)]
struct AttributeExprOperand {
    name: String,
    value: String,
    case_sensitivity: ParsedCaseSensitivity,
}

#[derive(PartialEq, Eq)]
enum SimpleExpr {
    ExplicitAny,
    ExplicitNoNamespace,
    LocalName(String),
    Namespace(Namespace),
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
    fn try_from(component: &Component<SelectorImplDescriptor>) -> Result<Self, SelectorError> {
        Ok(match component {
            Component::LocalName(n) => SimpleExpr::LocalName(n.name.to_owned()),
            Component::ExplicitUniversalType | Component::ExplicitAnyNamespace => {
                SimpleExpr::ExplicitAny
            }
            Component::Namespace(_, n) | Component::DefaultNamespace(n) => {
                SimpleExpr::Namespace(*n)
            }
            Component::ExplicitNoNamespace => SimpleExpr::ExplicitNoNamespace,
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
            // the parsed selector as we should bail earlier in the parser. However,
            // we'll keep this branch as the guarding measure, because parsing
            // happens in the external code changes to which we don't control.
            _ => return Err(SelectorError::UnsupportedSyntax),
        })
    }

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

#[derive(PartialEq, Eq)]
enum Expr {
    Simple(SimpleExpr),
    Negation(Vec<SimpleExpr>),
}

impl Expr {
    #[inline]
    fn try_from(component: &Component<SelectorImplDescriptor>) -> Result<Self, SelectorError> {
        Ok(match component {
            Component::Negation(e) => Expr::Negation(
                e.iter()
                    .map(SimpleExpr::try_from)
                    .collect::<Result<_, _>>()?,
            ),
            _ => Expr::Simple(SimpleExpr::try_from(component)?),
        })
    }
}

#[derive(PartialEq, Eq)]
pub struct AstNode<P> {
    expressions: Vec<Expr>,
    child_combinator_branches: AstNodeVec<P>,
    descendant_combinator_branches: AstNodeVec<P>,
    payload: Vec<P>,
}

impl<P> AstNode<P> {
    fn new_boxed(expressions: Vec<Expr>) -> Box<Self> {
        Box::new(AstNode {
            expressions,
            child_combinator_branches: Vec::default(),
            descendant_combinator_branches: Vec::default(),
            payload: Vec::default(),
        })
    }
}

#[derive(Default, PartialEq, Eq)]
pub struct Ast<P>(AstNodeVec<P>)
where
    P: PartialEq + Eq + Copy;

impl<P> Ast<P>
where
    P: PartialEq + Eq + Copy,
{
    #[inline]
    pub fn add_selector(&mut self, selector: &str, payload: P) -> Result<(), SelectorError> {
        let selector_list = parse_selector(selector)?;

        for selector in selector_list.0.into_iter() {
            self.add_parsed_selector(selector, payload)?;
        }

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
                branches.push(AstNode::new_boxed(expressions));

                branches.len() - 1
            }
        }
    }

    fn add_parsed_selector(
        &mut self,
        selector: Selector<SelectorImplDescriptor>,
        payload: P,
    ) -> Result<(), SelectorError> {
        let mut expressions = Vec::default();
        let mut branches = &mut self.0;

        macro_rules! host_and_switch_branch_vec {
            ($branches:ident) => {{
                let node_idx = Self::host_expressions(expressions, branches);
                branches = &mut branches[node_idx].$branches;
                expressions = Vec::default();
            }};
        }

        for component in selector.iter_raw_parse_order_from(0) {
            match component {
                Component::Combinator(c) => match c {
                    Combinator::Child => host_and_switch_branch_vec!(child_combinator_branches),
                    Combinator::Descendant => {
                        host_and_switch_branch_vec!(descendant_combinator_branches)
                    }
                    Combinator::NextSibling => {
                        return Err(SelectorError::UnsupportedCombinator('+'));
                    }
                    Combinator::LaterSibling => {
                        return Err(SelectorError::UnsupportedCombinator('~'));
                    }
                    _ => (),
                },
                _ => expressions.push(Expr::try_from(component)?),
            }
        }

        let node_idx = Self::host_expressions(expressions, branches);
        branches[node_idx].payload.push(payload);

        Ok(())
    }
}
