mod input_chunk;

pub use self::input_chunk::InputChunk;

// 4. Don't store InputChunk in tokenizer
// 5. Get rid of token view, since we don't have referential
// structure problem anymore.

// TransformStream contains tokenizer and input
// Peek is implemented on input, tokenizer methods
// receive input as argument.

// Data type:
// 1. Original chunk
// 2. Buffered

// Write:
//

// After write:
//

// Peek:
//
