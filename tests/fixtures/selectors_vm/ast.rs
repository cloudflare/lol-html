use cool_thing::{AstNode, AttributeExprOperand, Expr, SelectorError, SelectorsAst, SimpleExpr};
use selectors::attr::ParsedCaseSensitivity;

fn assert_ast(selectors: &[&str], expected: SelectorsAst<usize>) {
    let mut ast = SelectorsAst::default();

    for (idx, selector) in selectors.iter().enumerate() {
        ast.add_selector(selector, idx).unwrap();
    }

    assert_eq!(ast, expected);
}

fn assert_err(selector: &str, expected_err: SelectorError) {
    assert_eq!(
        SelectorsAst::default()
            .add_selector(selector, 0)
            .unwrap_err(),
        expected_err
    );
}

test_fixture!("Selectors AST", {
    test("Basic expressions", {
        macro_rules! assert_simple_expr {
            ($([$selector:expr, $expr:expr]),+) => {
                $(
                     assert_ast(
                        &[$selector],
                        SelectorsAst(vec![AstNode {
                            expressions: vec![$expr],
                            children: None,
                            descendants: None,
                            payload: Some(vec![0])
                        }]),
                    );
                )+
            };
        }

        assert_simple_expr!(
            ["*", Expr::Simple(SimpleExpr::ExplicitAny)],
            ["div", Expr::Simple(SimpleExpr::LocalName("div".into()))],
            ["#foo", Expr::Simple(SimpleExpr::Id("foo".into()))],
            [".bar", Expr::Simple(SimpleExpr::Class("bar".into()))],
            [
                "[foo]",
                Expr::Simple(SimpleExpr::AttributeExists("foo".into()))
            ],
            [
                r#"[foo="bar"]"#,
                Expr::Simple(SimpleExpr::AttributeEqual(AttributeExprOperand {
                    name: "foo".into(),
                    value: "bar".into(),
                    case_sensitivity: ParsedCaseSensitivity::CaseSensitive
                }))
            ],
            [
                r#"[foo~="bar" i]"#,
                Expr::Simple(SimpleExpr::AttributeIncludes(AttributeExprOperand {
                    name: "foo".into(),
                    value: "bar".into(),
                    case_sensitivity: ParsedCaseSensitivity::AsciiCaseInsensitive
                }))
            ],
            [
                r#"[foo|="bar" s]"#,
                Expr::Simple(SimpleExpr::AttributeDashMatch(AttributeExprOperand {
                    name: "foo".into(),
                    value: "bar".into(),
                    case_sensitivity: ParsedCaseSensitivity::ExplicitCaseSensitive
                }))
            ],
            [
                r#"[foo^="bar"]"#,
                Expr::Simple(SimpleExpr::AttributePrefix(AttributeExprOperand {
                    name: "foo".into(),
                    value: "bar".into(),
                    case_sensitivity: ParsedCaseSensitivity::CaseSensitive
                }))
            ],
            [
                r#"[foo*="bar"]"#,
                Expr::Simple(SimpleExpr::AttributeSubstring(AttributeExprOperand {
                    name: "foo".into(),
                    value: "bar".into(),
                    case_sensitivity: ParsedCaseSensitivity::CaseSensitive
                }))
            ],
            [
                r#"[foo$="bar"]"#,
                Expr::Simple(SimpleExpr::AttributeSuffix(AttributeExprOperand {
                    name: "foo".into(),
                    value: "bar".into(),
                    case_sensitivity: ParsedCaseSensitivity::CaseSensitive
                }))
            ],
            [r#"[foo*=""]"#, Expr::Simple(SimpleExpr::Unmatchable)],
            [
                ":not(div)",
                Expr::Negation(vec![SimpleExpr::LocalName("div".into()),])
            ]
        );
    });

    test("Compound selectors", {
        assert_ast(
            &[".foo#bar[baz]"],
            SelectorsAst(vec![AstNode {
                expressions: vec![
                    Expr::Simple(SimpleExpr::AttributeExists("baz".into())),
                    Expr::Simple(SimpleExpr::Id("bar".into())),
                    Expr::Simple(SimpleExpr::Class("foo".into())),
                ],
                children: None,
                descendants: None,
                payload: Some(vec![0]),
            }]),
        );
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
            SelectorsAst(vec![
                AstNode {
                    expressions: vec![Expr::Simple(SimpleExpr::Class("c1".into()))],
                    children: Some(vec![
                        AstNode {
                            expressions: vec![Expr::Simple(SimpleExpr::Class("c2".into()))],
                            children: None,
                            descendants: Some(vec![
                                AstNode {
                                    expressions: vec![Expr::Simple(SimpleExpr::Class("c3".into()))],
                                    children: None,
                                    descendants: Some(vec![AstNode {
                                        expressions: vec![Expr::Simple(SimpleExpr::Id(
                                            "foo".into(),
                                        ))],
                                        children: None,
                                        descendants: None,
                                        payload: Some(vec![0]),
                                    }]),
                                    payload: None,
                                },
                                AstNode {
                                    expressions: vec![Expr::Simple(SimpleExpr::Id("bar".into()))],
                                    children: None,
                                    descendants: None,
                                    payload: Some(vec![1]),
                                },
                            ]),
                            payload: None,
                        },
                        AstNode {
                            expressions: vec![Expr::Simple(SimpleExpr::Id("qux".into()))],
                            children: None,
                            descendants: None,
                            payload: Some(vec![2]),
                        },
                    ]),
                    descendants: Some(vec![
                        AstNode {
                            expressions: vec![Expr::Simple(SimpleExpr::Id("baz".into()))],
                            children: None,
                            descendants: None,
                            payload: Some(vec![3]),
                        },
                        AstNode {
                            expressions: vec![Expr::Simple(SimpleExpr::AttributeExists(
                                "foo".into(),
                            ))],
                            children: None,
                            descendants: Some(vec![AstNode {
                                expressions: vec![Expr::Simple(SimpleExpr::AttributeExists(
                                    "bar".into(),
                                ))],
                                children: None,
                                descendants: None,
                                payload: Some(vec![4]),
                            }]),
                            payload: None,
                        },
                    ]),
                    payload: None,
                },
                AstNode {
                    expressions: vec![Expr::Simple(SimpleExpr::Id("quz".into()))],
                    children: None,
                    descendants: None,
                    payload: Some(vec![5]),
                },
            ]),
        );
    });

    test("Multiple payloads", {
        assert_ast(
            &["#foo", "#foo"],
            SelectorsAst(vec![AstNode {
                expressions: vec![Expr::Simple(SimpleExpr::Id("foo".into()))],
                children: None,
                descendants: None,
                payload: Some(vec![0, 1]),
            }]),
        );
    });

    test("Selector lists", {
        assert_ast(
            &["#foo > div, #foo > span", "#foo > .c1, #foo > .c2"],
            SelectorsAst(vec![AstNode {
                expressions: vec![Expr::Simple(SimpleExpr::Id("foo".into()))],
                children: Some(vec![
                    AstNode {
                        expressions: vec![Expr::Simple(SimpleExpr::LocalName("div".into()))],
                        children: None,
                        descendants: None,
                        payload: Some(vec![0]),
                    },
                    AstNode {
                        expressions: vec![Expr::Simple(SimpleExpr::LocalName("span".into()))],
                        children: None,
                        descendants: None,
                        payload: Some(vec![0]),
                    },
                    AstNode {
                        expressions: vec![Expr::Simple(SimpleExpr::Class("c1".into()))],
                        children: None,
                        descendants: None,
                        payload: Some(vec![1]),
                    },
                    AstNode {
                        expressions: vec![Expr::Simple(SimpleExpr::Class("c2".into()))],
                        children: None,
                        descendants: None,
                        payload: Some(vec![1]),
                    },
                ]),
                descendants: None,
                payload: None,
            }]),
        )
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
