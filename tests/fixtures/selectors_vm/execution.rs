use cool_thing::selectors_vm::{Ast, ElementData, MatchInfo, SelectorMatchingVm};
use cool_thing::{AuxStartTagInfo, EndTag, LocalName, Namespace, StartTag};
use encoding_rs::UTF_8;
use std::collections::HashMap;
use std::collections::HashSet;

struct Expectation {
    should_bailout: bool,
    should_match_with_content: bool,
    matched_payload: HashSet<usize>,
}

#[derive(Default)]
struct TestElementData(HashSet<usize>);

impl ElementData for TestElementData {
    type MatchPayload = usize;

    fn matched_payload_mut(&mut self) -> &mut HashSet<usize> {
        &mut self.0
    }
}

macro_rules! local_name {
    ($t:expr) => {
        LocalName::from_str_without_replacements(&$t.name(), UTF_8).unwrap()
    };
}

// NOTE: these are macroses to preserve callsites on fails.
macro_rules! create_vm {
    ($selectors:expr) => {{
        let mut ast = Ast::default();

        for (i, selector) in $selectors.iter().enumerate() {
            ast.add_selector(selector, i).unwrap();
        }

        let vm: SelectorMatchingVm<TestElementData> = SelectorMatchingVm::new(&ast, UTF_8);

        vm
    }};
}

macro_rules! exec_for_start_tag_and_assert {
    ($vm:expr, $tag_html:expr, $ns:expr, $expectation:expr) => {
        parse_token!($tag_html, UTF_8, StartTag, |t: &mut StartTag| {
            let mut matched_payload = HashSet::default();

            {
                let mut match_handler = |m: MatchInfo<_>| {
                    assert_eq!(m.with_content, $expectation.should_match_with_content);
                    matched_payload.insert(m.payload);
                };

                let result = $vm.exec_for_start_tag(local_name!(t), $ns, &mut match_handler);

                if $expectation.should_bailout {
                    let mut aux_info_req = result.expect_err("Bailout expected");
                    let (input, attr_buffer) = t.raw_attributes();

                    aux_info_req(
                        &mut $vm,
                        AuxStartTagInfo {
                            input,
                            attr_buffer,
                            self_closing: t.self_closing(),
                        },
                        &mut match_handler,
                    );
                } else {
                    // NOTE: can't use unwrap() or expect() here, because
                    // Debug is not implemented for the closure in the error type.
                    #[allow(clippy::match_wild_err_arm)]
                    match result {
                        Ok(_) => (),
                        Err(_) => panic!("Should match without bailout"),
                    }
                }
            }

            assert_eq!(matched_payload, $expectation.matched_payload);
        });
    };
}

macro_rules! exec_for_end_tag_and_assert {
    ( $vm:expr, $tag_html:expr, $expected_unmatched_payload:expr) => {
        parse_token!($tag_html, UTF_8, EndTag, |t: &mut EndTag| {
            let mut unmatched_payload = HashMap::default();

            $vm.exec_for_end_tag(local_name!(t), |elem_data: TestElementData| {
                for payload in elem_data.0 {
                    unmatched_payload
                        .entry(payload)
                        .and_modify(|c| *c += 1)
                        .or_insert(1);
                }
            });

            assert_eq!(unmatched_payload, $expected_unmatched_payload);
        });
    };
}

test_fixture!("Selectors VM execution", {
    test("HTML elements", {
        let mut vm = create_vm!(&["a", "img.c1", ":not(a).c2"]);

        // Stack after:
        // - <a> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<a>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );

        // Void element.
        // Stack after:
        // - <a> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<img>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: false,
                matched_payload: set![],
            }
        );

        // Void element.
        // Stack after:
        // - <a> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<img class='c2 c1'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: false,
                matched_payload: set![1, 2],
            }
        );

        // Stack after:
        // - <a> (0)
        // - <a class='c2 c1'> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<a class='c1 c2'>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );

        // Stack after:
        // - <a> (0)
        // - <a class='c2 c1'> (0)
        // - <div class=c2> (2)
        exec_for_start_tag_and_assert!(
            vm,
            "<div class='c2'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![2],
            }
        );

        // Stack after:
        // - <a> (0)
        // - <a class='c2 c1'> (0)
        // - <div class=c2> (2)
        // - <h1 class='c1 c2 c3'> (2)
        exec_for_start_tag_and_assert!(
            vm,
            "<h1 class='c1 c2 c3'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![2],
            }
        );

        // Stack after:
        // Stack after:
        // - <a> (0)
        // - <a class='c2 c1'> (0)
        // - <div class=c2> (2)
        // - <h1 class='c1 c2 c3'> (2)
        exec_for_end_tag_and_assert!(vm, "</span>", map![]);

        // Stack after:
        // - <a> (0)
        exec_for_end_tag_and_assert!(vm, "</a>", map![(0, 1), (2, 2)]);

        // Stack after:
        // - <a> (0)
        exec_for_end_tag_and_assert!(vm, "</div>", map![]);

        // Stack after: empty
        exec_for_end_tag_and_assert!(vm, "</a>", map![(0, 1)]);
    });

    test("Foreign elements", {
        let mut vm = create_vm!(&["circle", "#foo"]);

        // Stack after:
        // - <svg>
        exec_for_start_tag_and_assert!(
            vm,
            "<svg>",
            Namespace::Svg,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Self-closing.
        // Stack after:
        // - <svg>
        exec_for_start_tag_and_assert!(
            vm,
            "<circle id=foo />",
            Namespace::Svg,
            Expectation {
                should_bailout: true,
                should_match_with_content: false,
                matched_payload: set![0, 1],
            }
        );

        // Self-closing.
        // Stack after:
        // - <svg>
        // - <circle> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<circle>",
            Namespace::Svg,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );

        // Stack after: empty
        exec_for_end_tag_and_assert!(vm, "</svg>", map![(0, 1)]);
    });

    test("Entry points", {
        let mut vm = create_vm!(&["div", "span[foo=bar]", "span#test"]);

        exec_for_start_tag_and_assert!(
            vm,
            "<div>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );

        exec_for_start_tag_and_assert!(
            vm,
            "<span>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        exec_for_start_tag_and_assert!(
            vm,
            "<span foo=bar>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![1],
            }
        );

        exec_for_start_tag_and_assert!(
            vm,
            "<span foo=bar id=test>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![1, 2],
            }
        );

        exec_for_end_tag_and_assert!(vm, "</div>", map![(0, 1), (1, 2), (2, 1)]);
    });

    test("Entry points bailout - last addr in set", {
        let mut vm = create_vm!(&["*", "span", "span[foo=bar]"]);

        exec_for_start_tag_and_assert!(
            vm,
            "<span>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 1],
            }
        );

        exec_for_start_tag_and_assert!(
            vm,
            "<span foo=bar>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 1, 2],
            }
        );
    });

    test("Jumps", {
        let mut vm = create_vm!(&["div > span", "div > #foo", ":not(span) > .c2 > .c3"]);

        // Stack after:
        // - <div>
        exec_for_start_tag_and_assert!(
            vm,
            "<div>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <div>
        // - <span> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<span>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );

        // Stack after:
        // - <div>
        // - <span> (0)
        // - <span>
        exec_for_start_tag_and_assert!(
            vm,
            "<span>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <div>
        // - <span> (0)
        exec_for_end_tag_and_assert!(vm, "</span>", map![]);

        // Stack after:
        // - <div>
        exec_for_end_tag_and_assert!(vm, "</span>", map![(0, 1)]);

        // Stack after:
        // - <div>
        // - <div id=foo class=c2> (1)
        exec_for_start_tag_and_assert!(
            vm,
            "<div id=foo class=c2>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![1],
            }
        );

        // Stack after:
        // - <div>
        // - <div id=foo class=c2> (1)
        // - <span class=c3> (0, 2)
        exec_for_start_tag_and_assert!(
            vm,
            "<span class=c3>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 2],
            }
        );

        // Stack after is empty
        exec_for_end_tag_and_assert!(vm, "</div>", map![(0, 1), (1, 1), (2, 1)]);
    });

    test("Jumps bailout - last addr in last set", {
        let mut vm = create_vm!(&["div > span", "div > *", "#foo > span", "#foo > ul.c1"]);

        // Stack after:
        // - <div id=foo>
        exec_for_start_tag_and_assert!(
            vm,
            "<div id=foo>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <div id=foo>
        // - <span> (0, 1, 2)
        exec_for_start_tag_and_assert!(
            vm,
            "<span>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 1, 2],
            }
        );

        // Stack after:
        // - <div id=foo>
        exec_for_end_tag_and_assert!(vm, "</span>", map![(0, 1), (1, 1), (2, 1)]);

        // Stack after:
        // - <div id=foo>
        // - <ul> (1)
        exec_for_start_tag_and_assert!(
            vm,
            "<ul>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![1],
            }
        );

        // Stack after:
        // - <div id=foo>
        exec_for_end_tag_and_assert!(vm, "</ul>", map![(1, 1)]);

        // Stack after:
        // - <div id=foo>
        // - <ul class=c1> (1, 3)
        exec_for_start_tag_and_assert!(
            vm,
            "<ul class=c1>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![1, 3],
            }
        );
    });

    test("Hereditary jumps", {
        let mut vm = create_vm!(&["div .c1", "#foo .c2 .c3"]);

        // Stack after:
        // - <div id=foo>
        exec_for_start_tag_and_assert!(
            vm,
            "<div id=foo>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <div id=foo>
        // - <div class='c1 c2'> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<div class='c1 c2'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );

        // Stack after:
        // - <div id=foo>
        // - <div class='c1 c2'> (0)
        // - <div class='c1 c2 c3'> (0, 1)
        exec_for_start_tag_and_assert!(
            vm,
            "<div class='c1 c2 c3'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 1],
            }
        );

        // Stack after:
        // - <div id=foo>
        // - <div class='c1 c2'> (0)
        // - <div class='c1 c2 c3'> (0, 1)
        // - <span class='c3'> (1)
        exec_for_start_tag_and_assert!(
            vm,
            "<span class='c3'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![1],
            }
        );

        // Stack after:
        // - <div id=foo>
        // - <div class='c1 c2'> (0)
        // - <div class='c1 c2 c3'> (0, 1)
        // - <span class='c3'> (1)
        // - <span class='c1 c3'> (0, 1)
        exec_for_start_tag_and_assert!(
            vm,
            "<span class='c1 c3'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 1],
            }
        );

        // Stack after:
        // - <div id=foo>
        // - <div class='c1 c2'> (0)
        exec_for_end_tag_and_assert!(vm, "</div>", map![(0, 2), (1, 3)]);

        // Stack after:
        // - <div id=foo>
        // - <div class='c1'> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<span class='c1'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );
    });

    test("Hereditary jumps bailout - first ancestor", {
        let mut vm = create_vm!(&[
            "body div *",
            "body div span#foo",
            "body div span",
            "body * #foo"
        ]);

        // Stack after:
        // - <body>
        exec_for_start_tag_and_assert!(
            vm,
            "<body>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        // - <div>
        exec_for_start_tag_and_assert!(
            vm,
            "<div>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        // - <div>
        exec_for_start_tag_and_assert!(
            vm,
            "<img>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: false,
                matched_payload: set![0],
            }
        );

        // Stack after:
        // - <body>
        // - <div>
        // - <span id=foo> (0, 1, 2, 3)
        exec_for_start_tag_and_assert!(
            vm,
            "<span id=foo>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 1, 2, 3],
            }
        );

        // Stack after:
        // - <body>
        // - <div>
        // - <span id=foo> (0, 1, 2, 3)
        // - <span> (0, 2)
        exec_for_start_tag_and_assert!(
            vm,
            "<span>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 2],
            }
        );

        // Stack after is empty
        exec_for_end_tag_and_assert!(vm, "</body>", map![(0, 2), (1, 1), (2, 2), (3, 1)]);
    });

    test("Hereditary jumps bailout - last ancestor, addr, set", {
        let mut vm = create_vm!(&["body *", "body span#foo", "div *"]);

        // Stack after:
        // - <body>
        exec_for_start_tag_and_assert!(
            vm,
            "<body>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        // - <div> (0)
        exec_for_start_tag_and_assert!(
            vm,
            "<div>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );

        // Stack after:
        // - <body>
        // - <div> (0)
        // - <div> (0, 1, 2)
        exec_for_start_tag_and_assert!(
            vm,
            "<span id=foo>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 1, 2],
            }
        );

        // Stack after:
        // - <body>
        // - <div> (0)
        // - <span> (0, 1, 2)
        // - <span> (0, 2)
        exec_for_start_tag_and_assert!(
            vm,
            "<span>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0, 2],
            }
        );

        // Stack after is empty
        exec_for_end_tag_and_assert!(vm, "</body>", map![(0, 3), (1, 1), (2, 2)]);
    });

    test("Compound selector", {
        let mut vm = create_vm!(&["body > span#foo .c1 .c2"]);

        // Stack after:
        // - <body>
        exec_for_start_tag_and_assert!(
            vm,
            "<body>",
            Namespace::Html,
            Expectation {
                should_bailout: false,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        // - <span>
        exec_for_start_tag_and_assert!(
            vm,
            "<span>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        exec_for_end_tag_and_assert!(vm, "</span>", map![]);

        // Stack after:
        // - <body>
        // - <span id=foo>
        exec_for_start_tag_and_assert!(
            vm,
            "<span id=foo>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        // - <span id=foo>
        // - <div>
        exec_for_start_tag_and_assert!(
            vm,
            "<div>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        // - <span id=foo>
        // - <div>
        // - <ul class='bar c1'>
        exec_for_start_tag_and_assert!(
            vm,
            "<ul class='bar c1'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        // - <span id=foo>
        // - <div>
        // - <ul class='bar c1'>
        // - <li class='c3'>
        exec_for_start_tag_and_assert!(
            vm,
            "<li class='c3'>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![],
            }
        );

        // Stack after:
        // - <body>
        // - <span id=foo>
        // - <div>
        // - <ul class='bar c1'>
        // - <li class='c3'>
        // - <span class=c2>
        exec_for_start_tag_and_assert!(
            vm,
            "<span class=c2>",
            Namespace::Html,
            Expectation {
                should_bailout: true,
                should_match_with_content: true,
                matched_payload: set![0],
            }
        );
    });
});
