use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature="integration_test")] {
        use cool_thing::selectors_vm::Ast;
        use std::env::args;

        fn main() {
            let arg = args()
                .nth(1)
                .expect("Tool should have at least one argument");

            let mut ast = Ast::default();

            serde_json::from_str::<Vec<String>>(&arg)
                .expect("Expected JSON-list of selector strings")
                .iter()
                .enumerate()
                .for_each(|(i, s)| {
                    let selector = s.parse().map_err(|e| format!("{}", e)).unwrap();

                    ast.add_selector(&selector, i)
                });

            println!("{:#?}", ast);
        }
    } else {
        fn main() {
            panic!(concat![
                "Selector's AST printer hasn't been compiled. ",
                "To compile it either run `./scripts/selectors_ast.sh` or `./scripts/test.sh`"
            ]);
        }
    }
}
