use crate::harness::ASCII_COMPATIBLE_ENCODINGS;
use cool_thing::selectors_vm::{Ast, AttributeMatcher, Compiler, Program};
use cool_thing::{LocalName, Namespace, StartTag};
use encoding_rs::{Encoding, UTF_8};
use std::collections::HashSet;

macro_rules! assert_instr_res {
    ($res:expr, $should_match:expr, $selector:expr, $input:expr, $encoding:expr) => {{
        let expected_payload = if *$should_match { Some(set![0]) } else { None };

        assert_eq!(
            $res.map(|b| b.matched_payload.to_owned()),
            expected_payload,
            "Instruction didn't produce expected matching result\n\
             selector: {:#?}\n\
             input: {:#?}\n\
             encoding: {:?}\n\
             ",
            $selector,
            $input,
            $encoding.name()
        );
    }};
}

fn compile(
    selectors: &[&str],
    encoding: &'static Encoding,
    expected_entry_point_count: usize,
) -> Program<usize> {
    let mut ast = Ast::default();

    for (idx, selector) in selectors.iter().enumerate() {
        ast.add_selector(selector, idx).unwrap();
    }

    let program = Compiler::new(encoding).compile(ast);

    assert_eq!(
        program.entry_points.end - program.entry_points.start,
        expected_entry_point_count
    );

    program
}

fn with_negated<'i>(
    selector: &str,
    test_cases: &[(&'i str, bool)],
) -> Vec<(String, Vec<(&'i str, bool)>)> {
    vec![
        (selector.to_string(), test_cases.to_owned()),
        (
            format!(":not({})", selector),
            test_cases
                .iter()
                .map(|(input, should_match)| (*input, !should_match))
                .collect(),
        ),
    ]
}

fn with_start_tag(
    html: &str,
    encoding: &'static Encoding,
    mut action: impl FnMut(LocalName, AttributeMatcher),
) {
    parse_token!(html, encoding, StartTag, |t: &mut StartTag| {
        let (input_bytes, attrs) = t.raw_attributes();
        let tag_name = t.name();
        let attr_matcher = AttributeMatcher::new(input_bytes, attrs, Namespace::Html);
        let local_name = LocalName::from_str_without_replacements(&tag_name, encoding).unwrap();

        action(local_name, attr_matcher);
    });
}

fn for_each_test_case<T>(
    test_cases: &[(&str, T)],
    encoding: &'static Encoding,
    action: impl Fn(&str, &T, LocalName, AttributeMatcher),
) {
    for (input, matching_data) in test_cases.iter() {
        with_start_tag(input, encoding, |local_name, attr_matcher| {
            action(input, matching_data, local_name, attr_matcher);
        });
    }
}

fn assert_attr_expr_matches(
    selector: &str,
    encoding: &'static Encoding,
    test_cases: &[(&str, bool)],
) {
    let program = compile(&[selector], encoding, 1);
    let instr = &*program.instructions[program.entry_points.start];

    for_each_test_case(
        test_cases,
        encoding,
        |input, should_match, local_name, attr_matcher| {
            instr
                .try_exec_without_attrs(&local_name)
                .expect_err("Instruction should not execute without attributes");

            let multi_step_res = instr.complete_execution_with_attrs(&attr_matcher);
            let res = instr.exec(&local_name, &attr_matcher);

            assert_eq!(multi_step_res, res);
            assert_instr_res!(res, should_match, selector, input, encoding);
        },
    );
}

fn assert_non_attr_expr_matches_and_negation_reverses_match(
    selector: &str,
    encoding: &'static Encoding,
    test_cases: &[(&str, bool)],
) {
    for (selector, test_cases) in with_negated(selector, test_cases) {
        let program = compile(&[&selector], encoding, 1);
        let instr = &*program.instructions[program.entry_points.start];

        for_each_test_case(
            &test_cases,
            encoding,
            |input, should_match, local_name, attr_matcher| {
                // NOTE: can't use unwrap() or expect() here, because
                // Debug is not implemented for the closure in the error type.
                #[allow(clippy::match_wild_err_arm)]
                let multi_step_res = match instr.try_exec_without_attrs(&local_name) {
                    Ok(res) => res,
                    Err(_) => panic!("Should match without attribute request"),
                };

                let res = instr.exec(&local_name, &attr_matcher);

                assert_eq!(multi_step_res, res);

                assert_instr_res!(res, should_match, selector, input, encoding);
            },
        );
    }
}

fn assert_attr_expr_matches_and_negation_reverses_match(
    selector: &str,
    encoding: &'static Encoding,
    test_cases: &[(&str, bool)],
) {
    for (selector, test_cases) in with_negated(selector, test_cases).iter() {
        assert_attr_expr_matches(selector, encoding, test_cases);
    }
}

macro_rules! exec_generic_instr {
    ($instr:expr, $local_name:expr, $attr_matcher:expr) => {{
        let res = $instr.exec(&$local_name, &$attr_matcher);

        let multi_step_res = match $instr.try_exec_without_attrs(&$local_name) {
            Ok(res) => res,
            Err(_) => $instr.complete_execution_with_attrs(&$attr_matcher),
        };

        assert_eq!(res, multi_step_res);

        res
    }};
}

fn assert_generic_expr_matches(
    selector: &str,
    encoding: &'static Encoding,
    test_cases: &[(&str, bool)],
) {
    let program = compile(&[selector], encoding, 1);
    let instr = &*program.instructions[program.entry_points.start];

    for_each_test_case(
        &test_cases,
        encoding,
        |input, should_match, local_name, attr_matcher| {
            let res = exec_generic_instr!(instr, local_name, attr_matcher);

            assert_instr_res!(res, should_match, selector, input, encoding);
        },
    );
}

macro_rules! exec_instr_range {
    ($range:expr, $program:expr, $local_name:expr, $attr_matcher:expr) => {{
        let mut matched_payload = HashSet::default();
        let mut jumps = Vec::default();
        let mut hereditary_jumps = Vec::default();

        for addr in $range.clone() {
            let res = exec_generic_instr!($program.instructions[addr], $local_name, $attr_matcher);

            if let Some(res) = res {
                for &p in &res.matched_payload {
                    matched_payload.insert(p);
                }

                if let Some(ref j) = res.jumps {
                    jumps.push(j.to_owned());
                }

                if let Some(ref j) = res.hereditary_jumps {
                    hereditary_jumps.push(j.to_owned());
                }
            }
        }

        (matched_payload, jumps, hereditary_jumps)
    }};
}

macro_rules! assert_payload {
    ($actual:expr, $expected:expr, $selectors:expr, $input:expr) => {
        assert_eq!(
            $actual,
            $expected.iter().cloned().collect::<HashSet<_>>(),
            "Instructions didn't produce expected payload\n\
             selectors: {:#?}\n\
             input: {:#?}\n\
             ",
            $selectors,
            $input
        );
    };
}

fn assert_entry_points_match(
    selectors: &[&str],
    expected_entry_point_count: usize,
    test_cases: &[(&str, Vec<usize>)],
) {
    let program = compile(selectors, UTF_8, expected_entry_point_count);

    // NOTE: encoding of the individual components is tested by other tests,
    // so we use only UTF-8 here.
    for_each_test_case(
        &test_cases,
        UTF_8,
        |input, expected_payload, local_name, attr_matcher| {
            let (matched_payload, _, _) =
                exec_instr_range!(program.entry_points, program, local_name, attr_matcher);

            assert_payload!(matched_payload, expected_payload, selectors, input);
        },
    );
}

test_fixture!("Selectors compiler", {
    test("Compiled non-attribute expression", {
        for encoding in ASCII_COMPATIBLE_ENCODINGS.iter() {
            assert_non_attr_expr_matches_and_negation_reverses_match(
                "*",
                encoding,
                &[("<div>", true), ("<span>", true), ("<anything-else>", true)],
            );

            assert_non_attr_expr_matches_and_negation_reverses_match(
                r#"[foo*=""]"#,
                encoding,
                &[
                    ("<div>", false),
                    ("<span>", false),
                    ("<anything-else>", false),
                ],
            );

            assert_non_attr_expr_matches_and_negation_reverses_match(
                "div",
                encoding,
                &[
                    ("<div>", true),
                    ("<divnotdiv>", false),
                    ("<span>", false),
                    ("<anything-else>", false),
                ],
            );

            assert_non_attr_expr_matches_and_negation_reverses_match(
                "span",
                encoding,
                &[
                    ("<div>", false),
                    ("<span>", true),
                    ("<anything-else>", false),
                ],
            );

            assert_non_attr_expr_matches_and_negation_reverses_match(
                "anything-else",
                encoding,
                &[
                    ("<div>", false),
                    ("<span>", false),
                    ("<anything-else>", true),
                ],
            );
        }
    });

    test("Compiled attribute expression", {
        for encoding in ASCII_COMPATIBLE_ENCODINGS.iter() {
            assert_attr_expr_matches_and_negation_reverses_match(
                "#foo",
                encoding,
                &[
                    ("<div bar=baz qux id='foo'>", true),
                    ("<div iD='foo'>", true),
                    ("<div bar=baz qux id='foo1'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                ".c2",
                encoding,
                &[
                    ("<div bar=baz class='c1 c2 c3 c4' qux>", true),
                    ("<div CLASS='c1 c2 c3 c4'>", true),
                    ("<div class='c1 c23 c4'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                "[foo]",
                encoding,
                &[
                    ("<div foo1 foo2 foo>", true),
                    ("<div FOo=123>", true),
                    ("<div id='baz'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo="bar"]"#,
                encoding,
                &[
                    ("<div fOo='bar'>", true),
                    ("<div foo=bar>", true),
                    ("<div foo='BaR'>", false),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo="bar" i]"#,
                encoding,
                &[
                    ("<div fOo='bar'>", true),
                    ("<div foo=bar>", true),
                    ("<div foo='BaR'>", true),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo~="bar3"]"#,
                encoding,
                &[
                    ("<div fOo='bar1\nbar2 bar3\tbar4'>", true),
                    ("<div foo='bar3'>", true),
                    ("<div foo='bar1 bar2 BAR3'>", false),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo~="bar3" i]"#,
                encoding,
                &[
                    ("<div fOo='bar1\nbar2 bar3\tbar4'>", true),
                    ("<div foo='bar3'>", true),
                    ("<div foo='bar1 bar2 BAR3'>", true),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            // NOTE: "lang" attribute always case-insesitive for HTML elements.
            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[lang|="en" s]"#,
                encoding,
                &[
                    ("<div lang='en-GB'>", true),
                    ("<div lang='en-US'>", true),
                    ("<div lang='en'>", true),
                    ("<div lang='En'>", false),
                    ("<div lang='En-GB'>", false),
                    ("<div lang='fr'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[lang|="en"]"#,
                encoding,
                &[
                    ("<div lang='en-GB'>", true),
                    ("<div lang='en-US'>", true),
                    ("<div lang='en'>", true),
                    ("<div lang='En'>", true),
                    ("<div lang='En-GB'>", true),
                    ("<div lang='fr'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo^="bar"]"#,
                encoding,
                &[
                    ("<div fOo='bar1\nbar2 bar3\tbar4'>", true),
                    ("<div foo='bar'>", true),
                    ("<div foo='BaR'>", false),
                    ("<div foo='bazbar'>", false),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo^="bar" i]"#,
                encoding,
                &[
                    ("<div fOo='bar1\nbar2 bar3\tbar4'>", true),
                    ("<div foo='bar'>", true),
                    ("<div foo='BaR'>", true),
                    ("<div foo='bazbar'>", false),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo*="bar"]"#,
                encoding,
                &[
                    ("<div fOo='bar1\nbar2 bar3\tbar4'>", true),
                    ("<div foo='bar'>", true),
                    ("<div foo='42BaR42'>", false),
                    ("<div foo='bazbatbar'>", true),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo*="bar" i]"#,
                encoding,
                &[
                    ("<div fOo='bar1\nbar2 bar3\tbar4'>", true),
                    ("<div foo='bar'>", true),
                    ("<div foo='42BaR42'>", true),
                    ("<div foo='bazbatbar'>", true),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo$="bar"]"#,
                encoding,
                &[
                    ("<div fOo='bar1\nbar2 bar3\tbar'>", true),
                    ("<div foo='bar'>", true),
                    ("<div foo='bazbar'>", true),
                    ("<div foo='BaR'>", false),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches_and_negation_reverses_match(
                r#"[foo$="bar" i]"#,
                encoding,
                &[
                    ("<div fOo='bar1\nbar2 bar3\tbar'>", true),
                    ("<div foo='bar'>", true),
                    ("<div foo='bazbar'>", true),
                    ("<div foo='BaR'>", true),
                    ("<div foo='42'>", false),
                    ("<div bar=baz qux>", false),
                ],
            );

            assert_attr_expr_matches(
                r#"#foo1.c1.c2[foo3][foo2$="bar"]"#,
                encoding,
                &[
                    ("<div id='foo1' class='c4 c2 c3 c1' foo3 foo2=heybar>", true),
                    (
                        "<div ID='foo1' class='c4 c2 c3 c1' foo3=test foo2=bar>",
                        true,
                    ),
                    ("<div id='foo1' class='c4 c2 c3 c1' foo3>", false),
                    (
                        "<div id='foo1' class='c4 c2 c3 c5' foo3 foo2=heybar>",
                        false,
                    ),
                    (
                        "<div id='foo12' class='c4 c2 c3 c5' foo3 foo2=heybar>",
                        false,
                    ),
                ],
            );
        }
    });

    test("Generic expressions", {
        for encoding in ASCII_COMPATIBLE_ENCODINGS.iter() {
            assert_generic_expr_matches(
                r#"div#foo1.c1.c2[foo3][foo2$="bar"]"#,
                encoding,
                &[
                    ("<div id='foo1' class='c4 c2 c3 c1' foo3 foo2=heybar>", true),
                    (
                        "<span id='foo1' class='c4 c2 c3 c1' foo3 foo2=heybar>",
                        false,
                    ),
                    (
                        "<div ID='foo1' class='c4 c2 c3 c1' foo3=test foo2=bar>",
                        true,
                    ),
                    ("<div id='foo1' class='c4 c2 c3 c1' foo3>", false),
                    (
                        "<div id='foo1' class='c4 c2 c3 c5' foo3 foo2=heybar>",
                        false,
                    ),
                    (
                        "<div id='foo12' class='c4 c2 c3 c5' foo3 foo2=heybar>",
                        false,
                    ),
                ],
            );

            assert_generic_expr_matches(
                r#"some-thing[lang|=en]"#,
                encoding,
                &[
                    ("<some-thing lang='en-GB'", true),
                    ("<some-thing lang='en-US'", true),
                    ("<some-thing lang='fr'>", false),
                    ("<some-thing lang>", false),
                    ("<span lang='en-GB'", false),
                ],
            );
        }
    });

    test("Multiple entry points", {
        assert_entry_points_match(
            &["div", "div.c1.c2", "#foo", ".c1#foo"],
            4,
            &[
                ("<div>", vec![0]),
                ("<div class='c3 c2  c1'>", vec![0, 1]),
                ("<div class='c1 c2' id=foo>", vec![0, 1, 2, 3]),
                ("<div class='c1' id=foo>", vec![0, 2, 3]),
                ("<span class='c1 c2'>", vec![]),
            ],
        );

        assert_entry_points_match(
            &["span, [foo$=bar]"],
            2,
            &[
                ("<span>", vec![0]),
                ("<div fOo=testbar>", vec![0]),
                ("<span foo=bar>", vec![0]),
            ],
        );
    });

    test("Jumps", {
        let selectors = [
            "div > .c1",
            "div > .c2",
            "div #d1",
            "div #d2",
            "[foo=bar] #id1 > #id2",
        ];

        let program = compile(&selectors, UTF_8, 2);

        macro_rules! exec {
            ($html:expr, $add_range:expr, $expected_payload:expr) => {{
                let mut jumps = Vec::default();
                let mut hereditary_jumps = Vec::default();

                with_start_tag($html, UTF_8, |local_name, attr_matcher| {
                    let res = exec_instr_range!($add_range, program, local_name, attr_matcher);

                    assert_payload!(res.0, $expected_payload, selectors, $html);

                    jumps = res.1;
                    hereditary_jumps = res.2;
                });

                (jumps, hereditary_jumps)
            }};
        }

        {
            let (jumps, hereditary_jumps) = exec!("<div>", program.entry_points, vec![]);

            assert_eq!(jumps.len(), 1);
            assert_eq!(hereditary_jumps.len(), 1);

            {
                let (jumps, hereditary_jumps) = exec!("<span class='c1 c2'>", jumps[0], vec![0, 1]);

                assert_eq!(jumps.len(), 0);
                assert_eq!(hereditary_jumps.len(), 0);
            }

            {
                let (jumps, hereditary_jumps) = exec!("<span class='c2'>", jumps[0], vec![1]);

                assert_eq!(jumps.len(), 0);
                assert_eq!(hereditary_jumps.len(), 0);
            }

            {
                let (jumps, hereditary_jumps) = exec!("<h1 id=d2>", hereditary_jumps[0], vec![3]);

                assert_eq!(jumps.len(), 0);
                assert_eq!(hereditary_jumps.len(), 0);
            }
        }

        {
            let (jumps, hereditary_jumps) = exec!("<div foo=bar>", program.entry_points, vec![]);

            assert_eq!(jumps.len(), 1);
            assert_eq!(hereditary_jumps.len(), 2);
        }

        {
            let (jumps, hereditary_jumps) = exec!("<span foo=bar>", program.entry_points, vec![]);

            assert_eq!(jumps.len(), 0);
            assert_eq!(hereditary_jumps.len(), 1);

            {
                let (jumps, hereditary_jumps) =
                    exec!("<table id=id1>", hereditary_jumps[0], vec![]);

                assert_eq!(jumps.len(), 1);
                assert_eq!(hereditary_jumps.len(), 0);

                {
                    let (jumps, hereditary_jumps) = exec!("<span id=id2>", jumps[0], vec![4]);

                    assert_eq!(jumps.len(), 0);
                    assert_eq!(hereditary_jumps.len(), 0);
                }
            }
        }
    });
});
