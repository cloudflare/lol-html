use lol_html::html_content::ContentType;
use lol_html::{Settings, element};

define_group!(
    "Rewriting",
    [
        (
            "Modification of tags of an element with lots of content",
            Settings::new().append_element_content_handler(element!("body", |el| {
                el.set_tag_name("body1").unwrap();
                el.after("test", ContentType::Text);

                Ok(())
            }))
        ),
        (
            "Remove content of an element",
            Settings::new().append_element_content_handler(element!("ul", |el| {
                el.set_inner_content("", ContentType::Text);

                Ok(())
            }))
        )
    ]
);
