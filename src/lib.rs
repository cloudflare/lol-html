extern crate lazycell;
extern crate safemem;

#[macro_use]
mod debug_trace;

pub mod base;
pub mod errors;
pub mod tokenizer;
pub mod transform_stream;

// TODO
// -- Functionality
// 2. TokenizerQuery
// 3. Eager tokenizer
// 4. Tokenizer driver
// 5. Adjustable limits
// 6. Get rid of token view as we don't need to store buffer anymore
//
// -- Performance
// 1. Implement benchmark
// 2. Get rid of dynamic dispatch for input (chunk from buffer)
// 3. LTO
// 4. In-state loops
// 5. Don't emit character immidiately, extend existing
// 6. State embedding

// We can use fast skip if:
// there is _ => () branch
// there are no consequent range or sequence branches

// If there is only one character branch except _, eof or eoc the use memchr
// Otherwise find the biggest char in the seq of skippable chars, use bit vector
// for skippable chars and compare that it less than 64.

// Try single loop
