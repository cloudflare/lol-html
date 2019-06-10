use super::parser::{Selector, SelectorImplDescriptor};
use selectors::attr::{AttrSelectorOperator, ParsedCaseSensitivity};
use selectors::parser::{Combinator, Component};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Eq, PartialEq, Debug)]
pub struct AttributeExprOperand {
    pub name: String,
    pub value: String,
    pub case_sensitivity: ParsedCaseSensitivity,
}

#[derive(PartialEq, Eq, Debug)]
pub enum AttributeExpr {
    Id(String),
    Class(String),
    AttributeExists(String),
    AttributeEqual(AttributeExprOperand),
    AttributeIncludes(AttributeExprOperand),
    AttributeDashMatch(AttributeExprOperand),
    AttributePrefix(AttributeExprOperand),
    AttributeSubstring(AttributeExprOperand),
    AttributeSuffix(AttributeExprOperand),
}

#[derive(PartialEq, Eq, Debug)]
pub enum NonAttributeExpr {
    ExplicitAny,
    Unmatchable,
    LocalName(String),
}

enum SimpleExpr {
    NonAttributeExpr(NonAttributeExpr),
    AttributeExpr(AttributeExpr),
}

impl SimpleExpr {
    #[inline]
    fn attr_expr_for_operator(
        operator: AttrSelectorOperator,
        name: &str,
        value: &str,
        case_sensitivity: ParsedCaseSensitivity,
    ) -> AttributeExpr {
        use AttrSelectorOperator::*;

        let operand = AttributeExprOperand {
            name: name.to_owned(),
            value: value.to_owned(),
            case_sensitivity,
        };

        match operator {
            DashMatch => AttributeExpr::AttributeDashMatch(operand),
            Equal => AttributeExpr::AttributeEqual(operand),
            Includes => AttributeExpr::AttributeIncludes(operand),
            Prefix => AttributeExpr::AttributePrefix(operand),
            Substring => AttributeExpr::AttributeSubstring(operand),
            Suffix => AttributeExpr::AttributeSuffix(operand),
        }
    }
}

impl From<&Component<SelectorImplDescriptor>> for SimpleExpr {
    #[inline]
    fn from(component: &Component<SelectorImplDescriptor>) -> Self {
        match component {
            Component::LocalName(n) => {
                SimpleExpr::NonAttributeExpr(NonAttributeExpr::LocalName(n.name.to_owned()))
            }
            Component::ExplicitUniversalType | Component::ExplicitAnyNamespace => {
                SimpleExpr::NonAttributeExpr(NonAttributeExpr::ExplicitAny)
            }
            Component::ExplicitNoNamespace => {
                SimpleExpr::NonAttributeExpr(NonAttributeExpr::Unmatchable)
            }
            Component::ID(id) => SimpleExpr::AttributeExpr(AttributeExpr::Id(id.to_owned())),
            Component::Class(c) => SimpleExpr::AttributeExpr(AttributeExpr::Class(c.to_owned())),
            Component::AttributeInNoNamespaceExists { local_name, .. } => {
                SimpleExpr::AttributeExpr(AttributeExpr::AttributeExists(local_name.to_owned()))
            }
            &Component::AttributeInNoNamespace {
                ref local_name,
                ref value,
                operator,
                case_sensitivity,
                never_matches,
            } => {
                if never_matches {
                    SimpleExpr::NonAttributeExpr(NonAttributeExpr::Unmatchable)
                } else {
                    SimpleExpr::AttributeExpr(Self::attr_expr_for_operator(
                        operator,
                        local_name,
                        value,
                        case_sensitivity,
                    ))
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
pub struct Expr<E>
where
    E: PartialEq + Eq + Debug,
{
    pub simple_expr: E,
    pub negation: bool,
}

impl<E> Expr<E>
where
    E: PartialEq + Eq + Debug,
{
    #[inline]
    fn new(simple_expr: E, negation: bool) -> Self {
        Expr {
            simple_expr,
            negation,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct Predicate {
    pub non_attr_exprs: Option<Vec<Expr<NonAttributeExpr>>>,
    pub attr_exprs: Option<Vec<Expr<AttributeExpr>>>,
}

#[inline]
fn add_expr_to_list<E>(list: &mut Option<Vec<Expr<E>>>, expr: E, negation: bool)
where
    E: PartialEq + Eq + Debug,
{
    list.get_or_insert_with(Vec::default)
        .push(Expr::new(expr, negation))
}

impl Predicate {
    #[inline]
    fn add_component(&mut self, component: &Component<SelectorImplDescriptor>, negation: bool) {
        match SimpleExpr::from(component) {
            SimpleExpr::AttributeExpr(e) => add_expr_to_list(&mut self.attr_exprs, e, negation),
            SimpleExpr::NonAttributeExpr(e) => {
                add_expr_to_list(&mut self.non_attr_exprs, e, negation)
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct AstNode<P>
where
    P: Hash + Eq,
{
    pub predicate: Predicate,
    pub children: Vec<AstNode<P>>,
    pub descendants: Vec<AstNode<P>>,
    pub payload: Rc<HashSet<P>>,
}

impl<P> AstNode<P>
where
    P: Hash + Eq,
{
    fn new(predicate: Predicate) -> Self {
        AstNode {
            predicate,
            children: Vec::default(),
            descendants: Vec::default(),
            payload: Rc::new(HashSet::default()),
        }
    }
}

#[derive(Default, PartialEq, Eq, Debug)]
pub struct Ast<P>
where
    P: PartialEq + Eq + Copy + Debug + Hash,
{
    pub root: Vec<AstNode<P>>,
    // NOTE: used to preallocate instruction vector during compilation.
    pub cumulative_node_count: usize,
}

impl<P> Ast<P>
where
    P: PartialEq + Eq + Copy + Debug + Hash,
{
    #[inline]
    fn host_expressions(
        predicate: Predicate,
        branches: &mut Vec<AstNode<P>>,
        cumulative_node_count: &mut usize,
    ) -> usize {
        match branches
            .iter()
            .enumerate()
            .find(|(_, n)| n.predicate == predicate)
        {
            Some((i, _)) => i,
            None => {
                branches.push(AstNode::new(predicate));
                *cumulative_node_count += 1;

                branches.len() - 1
            }
        }
    }

    pub fn add_selector(&mut self, selector: &Selector, payload: P) {
        for selector_item in &(selector.0).0 {
            let mut predicate = Predicate::default();
            let mut branches = &mut self.root;

            macro_rules! host_and_switch_branch_vec {
                ($branches:ident) => {{
                    let node_idx = Self::host_expressions(
                        predicate,
                        branches,
                        &mut self.cumulative_node_count,
                    );

                    branches = &mut branches[node_idx].$branches;
                    predicate = Predicate::default();
                }};
            }

            for component in selector_item.iter_raw_parse_order_from(0) {
                match component {
                    Component::Combinator(c) => match c {
                        Combinator::Child => host_and_switch_branch_vec!(children),
                        Combinator::Descendant => host_and_switch_branch_vec!(descendants),
                        _ => unreachable!(
                            "Unsupported selector components should be filtered out by the parser."
                        ),
                    },
                    Component::Negation(c) => {
                        c.iter().for_each(|c| predicate.add_component(c, true))
                    }
                    _ => predicate.add_component(component, false),
                }
            }

            let node_idx =
                Self::host_expressions(predicate, branches, &mut self.cumulative_node_count);

            Rc::make_mut(&mut branches[node_idx].payload).insert(payload);
        }
    }
}
