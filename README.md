# This is the Cool Thing

~~An annual Clodflare excercise in HTML parser writing~~

_The backend for Cloudflare Workers' HTMLRewriter_

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
./scripts/bench.sh
```

To run benchmark for Cool Thing only and skip comparison with other parsers:

```
./scripts/bench.sh cool_thing
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
