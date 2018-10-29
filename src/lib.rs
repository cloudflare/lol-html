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
// 1. Extract methods for actions
// 2. TokenizerQuery
// 3. Eager tokenizer
// 4. Tokenizer driver
// 5. Adjustable limits
//
// -- Performance
// 5. Don't emit character immidiately, extend existing
// 6. State embedding

// We can use fast skip if:
// there is _ => () branch
// there are no consequent range or sequence branches

// If there is only one character branch except _, eof or eoc the use memchr
// Otherwise find the biggest char in the seq of skippable chars, use bit vector
// for skippable chars and compare that it less than 64.

// Try single loop
