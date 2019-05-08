use cool_thing::selectors_vm::{ElementData, Stack, StackItem};
use cool_thing::LocalName;
use encoding_rs::UTF_8;
use std::collections::HashSet;

#[derive(Default)]
struct TestElementData(usize);

impl ElementData for TestElementData {
    type MatchPayload = ();

    fn matched_payload_mut(&mut self) -> &mut HashSet<()> {
        unreachable!();
    }
}

fn local_name(name: &'static str) -> LocalName<'static> {
    LocalName::from_str_without_replacements(name, UTF_8).unwrap()
}

fn item(name: &'static str, data: usize) -> StackItem<'static, TestElementData> {
    let mut item = StackItem::new(local_name(name));

    item.element_data = TestElementData(data);

    item
}

test_fixture!("Selectors VM stack", {
    test("Hereditary jumps flag", {
        let mut stack = Stack::default();

        stack.push_item(item("item1", 0));

        let mut item2 = item("item2", 1);
        item2.hereditary_jumps.push(0..0);
        stack.push_item(item2);

        let mut item3 = item("item3", 2);
        item3.hereditary_jumps.push(0..0);
        stack.push_item(item3);

        stack.push_item(item("item4", 3));

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
            ($up_to:expr, $expected_unmatched:expr, $expected_items:expr) => {{
                let mut stack = Stack::default();

                stack.push_item(item("html", 0));
                stack.push_item(item("body", 1));
                stack.push_item(item("div", 2));
                stack.push_item(item("div", 3));
                stack.push_item(item("span", 4));

                let mut unmatched = Vec::default();

                stack.pop_up_to(local_name($up_to), |d| {
                    unmatched.push(d.0);
                });

                assert_eq!(unmatched, $expected_unmatched);

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

        assert_pop_result!("span", vec![4], ["html", "body", "div", "div"]);
        assert_pop_result!("div", vec![3, 4], ["html", "body", "div"]);
        assert_pop_result!("body", vec![1, 2, 3, 4], ["html"]);
        assert_pop_result!("html", vec![0, 1, 2, 3, 4], []);

        let empty: Vec<usize> = Vec::default();

        assert_pop_result!("table", empty, ["html", "body", "div", "div", "span"]);
    });

    test("Pop up to - empty stack", {
        let mut stack = Stack::default();
        let mut handler_called = false;

        stack.pop_up_to(local_name("div"), |_: TestElementData| {
            handler_called = true;
        });

        assert!(!handler_called);
        assert_eq!(stack.items().len(), 0);
    });
});
