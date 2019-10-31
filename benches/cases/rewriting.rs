use cool_thing::html_content::*;
use cool_thing::*;

define_group!(
    "Rewriting",
    [
        (
            "Modification of tags of an element with lots of content",
            Settings {
                element_content_handlers: vec![element!("body", |el| {
                    el.set_tag_name("body1").unwrap();
                    el.after("test", ContentType::Text);

                    Ok(())
                })],
                ..Settings::default()
            }
        ),
        (
            "Remove content of an element",
            Settings {
                element_content_handlers: vec![element!("ul", |el| {
                    el.set_inner_content("", ContentType::Text);

                    Ok(())
                })],
                ..Settings::default()
            }
        ),
        (
            "Selector matching",
            Settings {
                element_content_handlers: vec![
                    element!("ul", noop_handler!()),
                    element!("ul > li", noop_handler!()),
                    element!("table > tbody td dfn", noop_handler!()),
                    element!("body table > tbody tr", noop_handler!()),
                    element!("body [href]", noop_handler!()),
                    element!("div img", noop_handler!()),
                    element!("div.note span", noop_handler!())
                ],
                ..Settings::default()
            }
        )
    ]
);
