use cool_thing::selectors_vm::{Stack, StackItem};
use cool_thing::LocalName;
use encoding_rs::UTF_8;
use std::collections::HashMap;

fn local_name(name: &'static str) -> LocalName<'static> {
    LocalName::from_str_without_replacements(name, UTF_8).unwrap()
}

fn item(name: &'static str) -> StackItem<'static, usize> {
    StackItem::new(local_name(name))
}

test_fixture!("Selectors VM stack", {
    test("Hereditary jumps flag", {
        let mut stack = Stack::default();

        stack.push_item(item("item1"));

        let mut item2 = item("item2");
        item2.hereditary_jumps.push(0..0);
        stack.push_item(item2);

        let mut item3 = item("item3");
        item3.hereditary_jumps.push(0..0);
        stack.push_item(item3);

        stack.push_item(item("item4"));

        assert_eq!(
            stack
                .items()
                .iter()
                .map(|i| i.has_ancestor_with_hereditary_jumps)
                .collect::<Vec<_>>(),
            [false, false, true, true]
        );
    });

    test("Pop up to", {
        macro_rules! assert_pop_result {
            ($up_to:expr, $expected_payload:expr, $expected_items:expr) => {{
                let mut stack = Stack::default();

                stack.push_item(item("html"));

                let mut body = item("body");
                body.matched_payload.insert(1);
                body.matched_payload.insert(2);
                stack.push_item(body);

                let mut div1 = item("div");
                div1.matched_payload.insert(3);
                stack.push_item(div1);

                let mut div2 = item("div");
                div2.matched_payload.insert(2);
                stack.push_item(div2);

                let mut span = item("span");
                span.matched_payload.insert(4);
                span.matched_payload.insert(5);
                span.matched_payload.insert(6);
                stack.push_item(span);

                let mut unmatched_payload = HashMap::default();

                stack.pop_up_to(local_name($up_to), |p| {
                    unmatched_payload
                        .entry(p)
                        .and_modify(|c| *c += 1)
                        .or_insert(1);
                });

                assert_eq!(unmatched_payload, $expected_payload);

                assert_eq!(
                    stack
                        .items()
                        .iter()
                        .map(|i| i.local_name.clone())
                        .collect::<Vec<_>>(),
                    $expected_items
                        .iter()
                        .map(|&i| local_name(i))
                        .collect::<Vec<_>>()
                );
            }};
        }

        assert_pop_result!(
            "span",
            map![(4, 1), (5, 1), (6, 1)],
            ["html", "body", "div", "div"]
        );

        assert_pop_result!(
            "div",
            map![(2, 1), (4, 1), (5, 1), (6, 1)],
            ["html", "body", "div"]
        );

        assert_pop_result!(
            "body",
            map![(1, 1), (2, 2), (3, 1), (4, 1), (5, 1), (6, 1)],
            ["html"]
        );

        assert_pop_result!(
            "html",
            map![(1, 1), (2, 2), (3, 1), (4, 1), (5, 1), (6, 1)],
            []
        );

        assert_pop_result!("table", map![], ["html", "body", "div", "div", "span"]);
    });

    test("Pop up to - empty stack", {
        let mut stack = Stack::default();
        let mut handler_called = false;

        stack.pop_up_to(local_name("div"), |_: usize| {
            handler_called = true;
        });

        assert!(!handler_called);
        assert_eq!(stack.items().len(), 0);
    });
});
