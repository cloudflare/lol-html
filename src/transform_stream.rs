// After tokenize_chunk
//-------------------------------------
// Original chunk mode
// if chunk_len - consumed > 0 {
//    copy [consumed..chunk_len] to beginning of buff (check bounds),
//    set watermark to chunk_len - consumed
//    set mode to buffered chunk
// } else {
//   // do nothing
// }
//

// Buffered chunk
// if chunk_len - consumed > 0 {
//     copy [consumed..chunk_len] to beginning of buff (check bounds)
//     set watermark to chunk_len - consumed
// }
// else {
// set mode to original chunk
//}

// Before tokenize chunk
//-------------------------------------
// Original chunk mode
// pass as is

// Buffered chunk
// copy chunk to buff[watermark] (check bounds)
// set watermark to watermark + chunk.len
