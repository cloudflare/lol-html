OUT = target/release
LIB := ../src/*.rs ../src/**/*.rs
C_API := src/*.rs
CFLAGS = -g -O0 -std=c99 -Wall -Wextra -Wcast-qual -Wwrite-strings -Wshadow -Winline -Wdisabled-optimization -Wuninitialized -Wcast-align -Wno-missing-field-initializers -Werror

test: $(OUT)/test-runner
	prove -v $(OUT)/test-runner

$(OUT)/test-runner: $(OUT)/test.o $(OUT)/picotest.o $(OUT)/libcoolthing.a
	$(CC) -lm -L$(OUT) -lcoolthing -o $@ $^

$(OUT)/libcoolthing.a: $(LIB) $(C_API)
	cargo clippy && cargo build --release

$(OUT)/test.o: tests/test.c include/cool_thing.h | $(OUT)
	$(CC) $(CFLAGS) -c $< -o $@

$(OUT)/picotest.o: tests/deps/picotest/picotest.c | $(OUT)
	$(CC) $(CFLAGS) -c $< -o $@

$(OUT):
	mkdir -p $(OUT)