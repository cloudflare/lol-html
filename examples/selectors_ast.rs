use cool_thing::SelectorsAst;
use std::env::args;

fn main() {
    let arg = args()
        .nth(1)
        .expect("Tool should have at least one argument");

    let mut ast = SelectorsAst::default();

    serde_json::from_str::<Vec<String>>(&arg)
        .expect("Expected JSON-list of selector strings")
        .iter()
        .enumerate()
        .for_each(|(i, s)| {
            ast.add_selector(s, i)
                .map_err(|e| format!("{}", e))
                .unwrap()
        });

    println!("{:#?}", ast);
}
