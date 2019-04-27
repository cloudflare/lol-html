use cool_thing::selectors_vm::{
    Ast, AstNode, AttributeExpr, AttributeExprOperand, Expr, NonAttributeExpr, Predicate,
    SelectorError,
};
use selectors::attr::ParsedCaseSensitivity;

fn assert_ast(selectors: &[&str], expected: Ast<usize>) {
    let mut ast = Ast::default();

    for (idx, selector) in selectors.iter().enumerate() {
        ast.add_selector(selector, idx).unwrap();
    }

    assert_eq!(ast, expected);
}

fn assert_err(selector: &str, expected_err: SelectorError) {
    assert_eq!(
        Ast::default().add_selector(selector, 0).unwrap_err(),
        expected_err
    );
}

test_fixture!("Selectors AST", {
    test("Simple non-attribute expressions", {
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
    });

    test("Simple attribute expressions", {
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
    });

    test("Compound selectors", {
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
    });

    test("Multiple payloads", {
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
    });

    test("Selector lists", {
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
    });

    test("Combinators", {
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
    });

    test("Parse errors", {
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
    });

    test("Parse errors - pseudo-classes", {
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
    });

    test("Parse errors - pseudo-elements", {
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
    });
});
