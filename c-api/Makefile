OUT = target/debug
LIB := ../src/*.rs ../src/**/*.rs
C_API := src/*.rs
CFLAGS = -g -O0 -std=c99 -pthread -Wall -Wextra -Wcast-qual -Wwrite-strings -Wshadow -Winline -Wdisabled-optimization -Wuninitialized -Wcast-align -Wno-missing-field-initializers -Werror -Wno-address

test: $(OUT)/test-runner
	prove -v $(OUT)/test-runner

$(OUT)/test-runner: $(OUT)/test.o $(OUT)/picotest.o $(OUT)/libcoolthing.a
	$(CC) $(CFLAGS) -Wl,-no-as-needed -lpthread -ldl -lm -o $@ $(OUT)/libcoolthing.a $^

$(OUT)/libcoolthing.a: $(LIB) $(C_API)
	cargo clippy && cargo build

$(OUT)/test.o: tests/test.c include/cool_thing.h | $(OUT)
	$(CC) $(CFLAGS) -c $< -o $@

$(OUT)/picotest.o: tests/deps/picotest/picotest.c | $(OUT)
	$(CC) $(CFLAGS) -c $< -o $@

$(OUT):
	mkdir -p $(OUT)