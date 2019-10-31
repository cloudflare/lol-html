# Developing

## Testing

To run unit tests only (tests defined in `/src`):

```
cargo test
```

To run all the available tests including integration tests, C API tests and linting:

```
./scripts/test.sh
```

To run only those tests that contain a `{substring}` in their name:

```
./scripts/test.sh {substring}
```

## Running benchmarks

```
cargo bench
```

To run only those benchmarks that contain a `{substring}` in their name:

```
cargo bench {substring}
```

## Useful debugging tools

### HTML parser tracer

The tool provides comprehensive trace information about parsing process of the given HTML input. For usage information run:

```
./scripts/parser_trace.sh -- -h
```

### CSS selector VM's AST printer

The tool prints selector VM's program AST for the given list of CSS selectors. CSS selector list should be specified in the JSON format:

```
./scripts/selectors_ast.sh '["selector1", "selector2", ...]'
```

## Fuzzing

### Fuzzing with cargo-fuzz
https://rust-fuzz.github.io/book/cargo-fuzz.html
cargo-fuzz requires Rust nightly.

Run fuzzing of `c-api`

```
./scripts/fuzz_c_api_with_libfuzzer.sh
```

Run fuzzing of rust library

```
./scripts/fuzz_with_libfuzzer.sh
```

### Fuzzing with afl
https://rust-fuzz.github.io/book/afl.html


```
./scripts/fuzz_with_afl.sh
```

### Fuzzing with honggfuzz
https://github.com/rust-fuzz/honggfuzz-rs

```
./scripts/fuzz_with_hongg.sh
```