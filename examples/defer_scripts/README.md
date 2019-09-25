# Defer render-blocking scripts

Reads HTML from the stdin stream and defers render-blocking scripts, then streams the result
to the stdout.

## Usage example

```sh
curl -NL https://git.io/JeOSZ | cargo run --example=defer_scripts
```
