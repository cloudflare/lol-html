use cool_thing::*;
use encoding_rs::UTF_8;

use crate::harness::{Output, TestTransformController};

const BUFFER_SIZE: usize = 20;

test_fixture!("Stream", {
    test("Buffer capacity limit", {
        let mut output = Output::new(UTF_8);

        let transform_controller =
            TestTransformController::new(Box::new(|_| {}), TokenCaptureFlags::all());

        let mut transform_stream = TransformStream::new(
            transform_controller,
            |c: &[u8]| output.push(c),
            BUFFER_SIZE,
            UTF_8,
        );

        // Use two chunks for the stream to force the usage of the buffer and
        // make sure to overflow it.
        let chunk_1 = format!("<img alt=\"{}", "l".repeat(BUFFER_SIZE / 2));
        let chunk_2 = format!("{}\" />", "r".repeat(BUFFER_SIZE / 2));

        transform_stream.write(chunk_1.as_bytes()).unwrap();

        let write_err = transform_stream.write(chunk_2.as_bytes()).unwrap_err();

        let buffer_capacity_err = write_err
            .find_root_cause()
            .downcast_ref::<BufferCapacityExceededError>()
            .unwrap();

        assert_eq!(
            *buffer_capacity_err,
            BufferCapacityExceededError { capacity: 20 }
        );

        transform_stream.end().unwrap();
    });
});
