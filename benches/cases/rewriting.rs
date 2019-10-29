use cool_thing::html_content::*;
use cool_thing::*;

macro_rules! create_runner {
    ($settings:expr) => {
        move |b, i: &Vec<Vec<u8>>| {
            b.iter(|| {
                let mut rewriter = HtmlRewriter::try_new($settings, |c: &[u8]| {
                    black_box(c);
                })
                .unwrap();

                for chunk in i {
                    rewriter.write(chunk).unwrap();
                }

                rewriter.end().unwrap();
            })
        }
    };
}

macro_rules! noop_handler {
    () => {
        |arg| {
            black_box(arg);
            Ok(())
        }
    };
}

define_group!(
    "Rewriting",
    [
        (
            "Modification of tags of an element with lots of content",
            create_runner!(Settings {
                element_content_handlers: vec![element!("body", |el| {
                    el.set_tag_name("body1").unwrap();
                    el.after("test", ContentType::Text);

                    Ok(())
                })],
                ..Settings::default()
            })
        ),
        (
            "Remove content of an element",
            create_runner!(Settings {
                element_content_handlers: vec![element!("ul", |el| {
                    el.set_inner_content("", ContentType::Text);

                    Ok(())
                })],
                ..Settings::default()
            })
        ),
        (
            "Selector matching",
            create_runner!(Settings {
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
            })
        )
    ]
);
