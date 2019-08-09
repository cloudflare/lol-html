use super::parser::{Selector, SelectorImplDescriptor};
use selectors::attr::{AttrSelectorOperator, ParsedCaseSensitivity};
use selectors::parser::{Combinator, Component};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

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
    pub payload: HashSet<P>,
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
            payload: HashSet::default(),
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

            branches[node_idx].payload.insert(payload);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selectors_vm::SelectorError;

    macro_rules! set {
        ($($items:expr),*) => {
            vec![$($items),*].into_iter().collect::<HashSet<_>>()
        };
    }

    fn assert_ast(selectors: &[&str], expected: Ast<usize>) {
        let mut ast = Ast::default();

        for (idx, selector) in selectors.iter().enumerate() {
            ast.add_selector(&selector.parse().unwrap(), idx);
        }

        assert_eq!(ast, expected);
    }

    fn assert_err(selector: &str, expected_err: SelectorError) {
        assert_eq!(selector.parse::<Selector>().unwrap_err(), expected_err);
    }

    #[test]
    fn simple_non_attr_expression() {
        vec![
            (
                "*",
                Expr {
                    simple_expr: NonAttributeExpr::ExplicitAny,
                    negation: false,
                },
            ),
            (
                "div",
                Expr {
                    simple_expr: NonAttributeExpr::LocalName("div".into()),
                    negation: false,
                },
            ),
            (
                r#"[foo*=""]"#,
                Expr {
                    simple_expr: NonAttributeExpr::Unmatchable,
                    negation: false,
                },
            ),
            (
                ":not(div)",
                Expr {
                    simple_expr: NonAttributeExpr::LocalName("div".into()),
                    negation: true,
                },
            ),
        ]
        .into_iter()
        .for_each(|(selector, expected)| {
            assert_ast(
                &[selector],
                Ast {
                    root: vec![AstNode {
                        predicate: Predicate {
                            non_attr_exprs: Some(vec![expected]),
                            attr_exprs: None,
                        },
                        children: vec![],
                        descendants: vec![],
                        payload: set![0],
                    }],
                    cumulative_node_count: 1,
                },
            );
        });
    }

    #[test]
    fn simple_attr_expression() {
        vec![
            (
                "#foo",
                Expr {
                    simple_expr: AttributeExpr::Id("foo".into()),
                    negation: false,
                },
            ),
            (
                ".bar",
                Expr {
                    simple_expr: AttributeExpr::Class("bar".into()),
                    negation: false,
                },
            ),
            (
                "[foo]",
                Expr {
                    simple_expr: AttributeExpr::AttributeExists("foo".into()),
                    negation: false,
                },
            ),
            (
                r#"[foo="bar"]"#,
                Expr {
                    simple_expr: AttributeExpr::AttributeEqual(AttributeExprOperand {
                        name: "foo".into(),
                        value: "bar".into(),
                        case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
                    }),
                    negation: false,
                },
            ),
            (
                r#"[foo~="bar" i]"#,
                Expr {
                    simple_expr: AttributeExpr::AttributeIncludes(AttributeExprOperand {
                        name: "foo".into(),
                        value: "bar".into(),
                        case_sensitivity: ParsedCaseSensitivity::AsciiCaseInsensitive,
                    }),
                    negation: false,
                },
            ),
            (
                r#"[foo|="bar" s]"#,
                Expr {
                    simple_expr: AttributeExpr::AttributeDashMatch(AttributeExprOperand {
                        name: "foo".into(),
                        value: "bar".into(),
                        case_sensitivity: ParsedCaseSensitivity::ExplicitCaseSensitive,
                    }),
                    negation: false,
                },
            ),
            (
                r#"[foo^="bar"]"#,
                Expr {
                    simple_expr: AttributeExpr::AttributePrefix(AttributeExprOperand {
                        name: "foo".into(),
                        value: "bar".into(),
                        case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
                    }),
                    negation: false,
                },
            ),
            (
                r#"[foo*="bar"]"#,
                Expr {
                    simple_expr: AttributeExpr::AttributeSubstring(AttributeExprOperand {
                        name: "foo".into(),
                        value: "bar".into(),
                        case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
                    }),
                    negation: false,
                },
            ),
            (
                r#"[foo$="bar"]"#,
                Expr {
                    simple_expr: AttributeExpr::AttributeSuffix(AttributeExprOperand {
                        name: "foo".into(),
                        value: "bar".into(),
                        case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
                    }),
                    negation: false,
                },
            ),
            (
                r#":not([foo$="bar"])"#,
                Expr {
                    simple_expr: AttributeExpr::AttributeSuffix(AttributeExprOperand {
                        name: "foo".into(),
                        value: "bar".into(),
                        case_sensitivity: ParsedCaseSensitivity::CaseSensitive,
                    }),
                    negation: true,
                },
            ),
        ]
        .into_iter()
        .for_each(|(selector, expected)| {
            assert_ast(
                &[selector],
                Ast {
                    root: vec![AstNode {
                        predicate: Predicate {
                            non_attr_exprs: None,
                            attr_exprs: Some(vec![expected]),
                        },
                        children: vec![],
                        descendants: vec![],
                        payload: set![0],
                    }],
                    cumulative_node_count: 1,
                },
            );
        });
    }

    #[test]
    fn compound_selectors() {
        assert_ast(
            &["div.foo#bar:not([baz])"],
            Ast {
                root: vec![AstNode {
                    predicate: Predicate {
                        non_attr_exprs: Some(vec![Expr {
                            simple_expr: NonAttributeExpr::LocalName("div".into()),
                            negation: false,
                        }]),
                        attr_exprs: Some(vec![
                            Expr {
                                simple_expr: AttributeExpr::AttributeExists("baz".into()),
                                negation: true,
                            },
                            Expr {
                                simple_expr: AttributeExpr::Id("bar".into()),
                                negation: false,
                            },
                            Expr {
                                simple_expr: AttributeExpr::Class("foo".into()),
                                negation: false,
                            },
                        ]),
                    },
                    children: vec![],
                    descendants: vec![],
                    payload: set![0],
                }],
                cumulative_node_count: 1,
            },
        );
    }

    #[test]
    fn multiple_payloads() {
        assert_ast(
            &["#foo", "#foo"],
            Ast {
                root: vec![AstNode {
                    predicate: Predicate {
                        non_attr_exprs: None,
                        attr_exprs: Some(vec![Expr {
                            simple_expr: AttributeExpr::Id("foo".into()),
                            negation: false,
                        }]),
                    },
                    children: vec![],
                    descendants: vec![],
                    payload: set![0, 1],
                }],
                cumulative_node_count: 1,
            },
        );
    }

    #[test]
    fn selector_list() {
        assert_ast(
            &["#foo > div, #foo > span", "#foo > .c1, #foo > .c2"],
            Ast {
                root: vec![AstNode {
                    predicate: Predicate {
                        non_attr_exprs: None,
                        attr_exprs: Some(vec![Expr {
                            simple_expr: AttributeExpr::Id("foo".into()),
                            negation: false,
                        }]),
                    },
                    children: vec![
                        AstNode {
                            predicate: Predicate {
                                non_attr_exprs: Some(vec![Expr {
                                    simple_expr: NonAttributeExpr::LocalName("div".into()),
                                    negation: false,
                                }]),
                                attr_exprs: None,
                            },
                            children: vec![],
                            descendants: vec![],
                            payload: set![0],
                        },
                        AstNode {
                            predicate: Predicate {
                                non_attr_exprs: Some(vec![Expr {
                                    simple_expr: NonAttributeExpr::LocalName("span".into()),
                                    negation: false,
                                }]),
                                attr_exprs: None,
                            },
                            children: vec![],
                            descendants: vec![],
                            payload: set![0],
                        },
                        AstNode {
                            predicate: Predicate {
                                non_attr_exprs: None,
                                attr_exprs: Some(vec![Expr {
                                    simple_expr: AttributeExpr::Class("c1".into()),
                                    negation: false,
                                }]),
                            },
                            children: vec![],
                            descendants: vec![],
                            payload: set![1],
                        },
                        AstNode {
                            predicate: Predicate {
                                non_attr_exprs: None,
                                attr_exprs: Some(vec![Expr {
                                    simple_expr: AttributeExpr::Class("c2".into()),
                                    negation: false,
                                }]),
                            },
                            children: vec![],
                            descendants: vec![],
                            payload: set![1],
                        },
                    ],
                    descendants: vec![],
                    payload: set![],
                }],
                cumulative_node_count: 5,
            },
        )
    }

    #[test]
    fn combinators() {
        assert_ast(
            &[
                ".c1 > .c2 .c3 #foo",
                ".c1 > .c2 #bar",
                ".c1 > #qux",
                ".c1 #baz",
                ".c1 [foo] [bar]",
                "#quz",
            ],
            Ast {
                root: vec![
                    AstNode {
                        predicate: Predicate {
                            non_attr_exprs: None,
                            attr_exprs: Some(vec![Expr {
                                simple_expr: AttributeExpr::Class("c1".into()),
                                negation: false,
                            }]),
                        },
                        children: vec![
                            AstNode {
                                predicate: Predicate {
                                    non_attr_exprs: None,
                                    attr_exprs: Some(vec![Expr {
                                        simple_expr: AttributeExpr::Class("c2".into()),
                                        negation: false,
                                    }]),
                                },
                                children: vec![],
                                descendants: vec![
                                    AstNode {
                                        predicate: Predicate {
                                            non_attr_exprs: None,
                                            attr_exprs: Some(vec![Expr {
                                                simple_expr: AttributeExpr::Class("c3".into()),
                                                negation: false,
                                            }]),
                                        },
                                        children: vec![],
                                        descendants: vec![AstNode {
                                            predicate: Predicate {
                                                non_attr_exprs: None,
                                                attr_exprs: Some(vec![Expr {
                                                    simple_expr: AttributeExpr::Id("foo".into()),
                                                    negation: false,
                                                }]),
                                            },
                                            children: vec![],
                                            descendants: vec![],
                                            payload: set![0],
                                        }],
                                        payload: set![],
                                    },
                                    AstNode {
                                        predicate: Predicate {
                                            non_attr_exprs: None,
                                            attr_exprs: Some(vec![Expr {
                                                simple_expr: AttributeExpr::Id("bar".into()),
                                                negation: false,
                                            }]),
                                        },
                                        children: vec![],
                                        descendants: vec![],
                                        payload: set![1],
                                    },
                                ],
                                payload: set![],
                            },
                            AstNode {
                                predicate: Predicate {
                                    non_attr_exprs: None,
                                    attr_exprs: Some(vec![Expr {
                                        simple_expr: AttributeExpr::Id("qux".into()),
                                        negation: false,
                                    }]),
                                },
                                children: vec![],
                                descendants: vec![],
                                payload: set![2],
                            },
                        ],
                        descendants: vec![
                            AstNode {
                                predicate: Predicate {
                                    non_attr_exprs: None,
                                    attr_exprs: Some(vec![Expr {
                                        simple_expr: AttributeExpr::Id("baz".into()),
                                        negation: false,
                                    }]),
                                },
                                children: vec![],
                                descendants: vec![],
                                payload: set![3],
                            },
                            AstNode {
                                predicate: Predicate {
                                    non_attr_exprs: None,
                                    attr_exprs: Some(vec![Expr {
                                        simple_expr: AttributeExpr::AttributeExists("foo".into()),
                                        negation: false,
                                    }]),
                                },
                                children: vec![],
                                descendants: vec![AstNode {
                                    predicate: Predicate {
                                        non_attr_exprs: None,
                                        attr_exprs: Some(vec![Expr {
                                            simple_expr: AttributeExpr::AttributeExists(
                                                "bar".into(),
                                            ),
                                            negation: false,
                                        }]),
                                    },
                                    children: vec![],
                                    descendants: vec![],
                                    payload: set![4],
                                }],
                                payload: set![],
                            },
                        ],
                        payload: set![],
                    },
                    AstNode {
                        predicate: Predicate {
                            non_attr_exprs: None,
                            attr_exprs: Some(vec![Expr {
                                simple_expr: AttributeExpr::Id("quz".into()),
                                negation: false,
                            }]),
                        },
                        children: vec![],
                        descendants: vec![],
                        payload: set![5],
                    },
                ],
                cumulative_node_count: 10,
            },
        );
    }

    #[test]
    fn parse_errors() {
        assert_err("div@", SelectorError::UnexpectedToken);
        assert_err("div.", SelectorError::UnexpectedEnd);
        assert_err(r#"div[="foo"]"#, SelectorError::MissingAttributeName);
        assert_err("", SelectorError::EmptySelector);
        assert_err("div >", SelectorError::DanglingCombinator);
        assert_err(
            r#"div[foo~"bar"]"#,
            SelectorError::UnexpectedTokenInAttribute,
        );
        assert_err(":not(:not(p))", SelectorError::NestedNegation);
        assert_err("svg|img", SelectorError::NamespacedSelector);
        assert_err(".foo()", SelectorError::InvalidClassName);
        assert_err(":not()", SelectorError::EmptyNegation);
        assert_err("div + span", SelectorError::UnsupportedCombinator('+'));
        assert_err("div ~ span", SelectorError::UnsupportedCombinator('~'));
    }

    #[test]
    fn pseudo_class_parse_errors() {
        [
            ":active",
            ":any-link",
            ":blank",
            ":checked",
            ":current",
            ":default",
            ":defined",
            ":dir(rtl)",
            ":disabled",
            ":drop",
            ":empty",
            ":enabled",
            ":first",
            ":first-child",
            ":first-of-type",
            ":fullscreen",
            ":future",
            ":focus",
            ":focus-visible",
            ":focus-within",
            ":has(div)",
            ":host",
            ":host(h1)",
            ":host-context(h1)",
            ":hover",
            ":indeterminate",
            ":in-range",
            ":invalid",
            ":is(header)",
            ":lang(en)",
            ":last-child",
            ":last-of-type",
            ":left",
            ":link",
            ":local-link",
            ":nth-child(1)",
            ":nth-col(1)",
            ":nth-last-child(1)",
            ":nth-last-col(1)",
            ":nth-last-of-type(1)",
            ":nth-of-type(1)",
            ":only-child",
            ":only-of-type",
            ":optional",
            ":out-of-range",
            ":past",
            ":placeholder-shown",
            ":read-only",
            ":read-write",
            ":required",
            ":right",
            ":root",
            ":scope",
            ":target",
            ":target-within",
            ":user-invalid",
            ":valid",
            ":visited",
            ":where(p)",
        ]
        .iter()
        .for_each(|s| assert_err(s, SelectorError::UnsupportedPseudoClassOrElement));
    }

    #[test]
    fn pseudo_elements_parse_errors() {
        [
            "::after",
            "::backdrop",
            "::before",
            "::cue",
            "::first-letter",
            "::first-line",
            "::grammar-error",
            "::marker",
            "::placeholder",
            "::selection",
            "::slotted()",
            "::spelling-error",
        ]
        .iter()
        .for_each(|s| assert_err(s, SelectorError::UnsupportedPseudoClassOrElement));
    }

    #[test]
    fn negated_pseudo_class_parse_error() {
        assert_err(
            ":not(:nth-last-child(even))",
            SelectorError::UnsupportedPseudoClassOrElement,
        );
    }
}
