#[macro_use]
extern crate getset;

#[macro_use]
extern crate failure;

#[macro_use]
mod debug_trace;

#[macro_use]
pub mod base;

pub mod token;
pub mod tokenizer;
pub mod transform_stream;
// TODO
// -- Functionality
// 5. Adjustable limits
//
// -- Performance
// 5. Don't emit character immidiately, extend existing
// 6. State embedding

// 7. We can use fast skip if:
// there is _ => () branch
// there are no consequent range or sequence branches
// If there is only one character branch except _, eof or eoc the use memchr
// Otherwise find the biggest char in the seq of skippable chars, use bit vector
// for skippable chars and compare that it less than 64.
// Try single loop

// 8.Lazily initialize buffer
// 9.Use smaller buffer for attributes (default?), it will grow proportional to
// to the buffer size, add the comment.
