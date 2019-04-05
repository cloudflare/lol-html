#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Namespace {
    Html,
    Svg,
    MathML,
}

impl Default for Namespace {
    #[inline]
    fn default() -> Self {
        Namespace::Html
    }
}
