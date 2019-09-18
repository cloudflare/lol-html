# Mixed content rewriter

Reads HTML from the stdin stream, rewrites [mixed content](https://developer.mozilla.org/en-US/docs/Web/Security/Mixed_content)
in it and streams the result to the stdout.

## Usage example

```sh
curl -NL https://git.io/JeOSZ | cargo run --example=mixed_content_rewriter
```
