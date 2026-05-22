use lol_html::{Settings, element};

define_group!(
    "Selector matching",
    [
        (
            "Match-all selector",
            Settings::new().append_element_content_handler(element!("*", noop_handler!()))
        ),
        (
            "Tag name selector",
            Settings::new().append_element_content_handler(element!("div", noop_handler!()))
        ),
        (
            "Class selector",
            Settings::new().append_element_content_handler(element!(".note", noop_handler!()))
        ),
        (
            "Attribute selector",
            Settings::new().append_element_content_handler(element!("[href]", noop_handler!()))
        ),
        (
            "Multiple selectors",
            Settings::new()
                .append_element_content_handler(element!("ul", noop_handler!()))
                .append_element_content_handler(element!("ul > li", noop_handler!()))
                .append_element_content_handler(element!("table > tbody td dfn", noop_handler!()))
                .append_element_content_handler(element!("body table > tbody tr", noop_handler!()))
                .append_element_content_handler(element!("body [href]", noop_handler!()))
                .append_element_content_handler(element!("div img", noop_handler!()))
                .append_element_content_handler(element!("div.note span", noop_handler!()))
        )
    ]
);
