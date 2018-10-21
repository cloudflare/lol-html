extern crate lazycell;

pub mod base;
pub mod tokenizer;
pub mod transform_stream;

// TODO
// -- Functionality
// 2. Streaming
// 3. Eager tokenizer
// 4. Tokenizer driver
// 5. Adjustable limits
// 6. Get rid of token view as we don't need to store buffer anymore
//
// -- Performance
// 1. Implement benchmark
// 2. LTO
// 3. In-state loops
// 4. Don't emit character immidiately, extend existing
// 5. State embedding

// 1. Revive Chunk. Move last, pos, consume, lookahead, consume_few
