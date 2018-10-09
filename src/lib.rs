#[macro_use]
pub mod tag_name;

pub mod lex_unit;
pub mod tokenizer;

// TODO
// -- Functionality
// 1. Feedback stuff description comments
// 2. Streaming
// 3. Eager tokenizer
// 4. Tokenizer driver
// 5. Adjustable limits
// 6. Try to get rid of lifetime for Tokenizer (it is required only for testing API)
//
// -- Performance
// 1. Implement benchmark
// 2. LTO
// 3. In-state loops
// 4. Don't emit character immidiately, extend existing
// 5. State embedding
