# Changelog

## v2.7.2

- Replaced several panicking assertions with gracefully reported errors, especially in the C API

## v2.7.1

- Performance improvements.
- Updated dependencies.

## v2.7.0

- Improve type generation in js-api.
- Updated dependencies.

## v2.6.0

- Added source code locations to the C and JS APIs
- Significant performance improvements and code size reductions

## v2.5.0

- Source code locations for tags and other tokens.
- Document text chunks and escaping of attributes.
- Selector validation improvements.

## v2.4.0

 - Upgraded `selectors` and `cssparser`.

## v2.3.0

 - Added `element.onEndTag` to JS bindings.
 - Refactored TextDecoder and token construction to avoid heap allocations.
 - Added fast paths for UTF-8 rewrites.

## v2.2.0

 - Updated cssparser and selectors dependencies
 - Adopted `cargo-c` for building the C API
 - Added WASM/JS API
 - An invalid `/>` syntax will be removed when content is added to an HTML element

## v2.1.0

- Added streaming handlers.
- Only allow changing the charset once with the `<meta>` tag, in accordance with the HTML spec.
- Fixed parsing of invalid elements in `<svg>` and `<math>`.

## v2.0.0

- Added the ability for the rewriter to be [`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html).
  The `send` module contains the utilities for that.

## v1.2.1

- Remove unmaintained `safemem` dependency.

## v1.2.0

- Expose `is_self_closing` and `can_have_content` in C api.
- Make `ElementContentHandlers` and `DocumentContentHandlers` fields public.
- Add missing docs to public API.

## v1.1.1

### Fixed

- Ensure that `TagScanner::is_in_end_tag` resets when changing parsers.

## v1.1.0

### Added

- Added ability to get the tag and attribute names with the original casing.

## v1.0.1

### Fixed

- The C API's new `lol_html_element_add_end_tag_handler()` function now sets the last error retrievable by `lol_html_take_last_error()` if it is called on an element that can have no end tag.

## v1.0.0

Yes, you got that right: this is the first 1.x release!  From now on you should expect this project to adhere to
the semantic versioning spec (we have been somewhat relaxed about that in the past).

### Added

* Added `Element::end_tag_handlers()` which allows better control over the end tag handlers.

### Changed

* Removed `Element::on_end_tag()` and `Element::add_on_end_tag()` in favor of the newly added
  `Element::end_tag_handlers()`.

## v0.4.0

### Added

- Added method `TextChunk::as_mut_str()` and `TextChunk::set_str()` for in-place modifications to the text in a
  `TextChunk`. (#175)

### Changed

- Modified method `Element::on_end_tag()` to support multiple handlers. This is a breaking change since the old
  semantics of the method was to overwrite any previously set handler. (#177)

## v0.3.3

### Added

- Support dynamic charset change on meta tags in HtmlRewriter. (#162)
- Add `Element::can_have_content()`. (#163)

## v0.3.2

### Added

- Add `Doctype::remove`. (#129)
- Add `Element::start_tag()` and `Element::is_self_closing()`. (#148)
- Add mutation methods to `StartTag` and `EndTag`. (#148)
- Implement `Eq` for all types that implement `PartialEq`. (#146)

### Fixed

- Changed the HTML parser to more closely match the spec. This only affects rewriters which modify HTML comments. (#128)

## v0.3.1

### Added

- Add `Element::on_end_tag` (#97, #107, #124)

### Changed

- Change string allocators in the C API to return `lol_html_str_t`, not `lol_html_str_t*`. This was necessary to fix a memory leak in `lol_html_str_free`. (#115)
- Update dependencies (#98, #103)

### Fixed

- Fix memory leaks in C API (#113, #115)

## v0.3.0
- Add unofficial Go bindings to the README (#77)
- Update dependencies (#73)
- Take `self` in HtmlRewriter::end (#68)
- Refactor HTMLRewriter Settings to make `HTMLRewriter::new` infallible (#70)
- Allow using `element!` in a separate expression from `rewrite_str` (#69)
- Update to hashbrown 0.9 (#64)
- Add Send+Sync constraint for ContentHandler Error
- feat: Allow using either Settings or RewriteStrSettings (#57)
- Fix unhappy clippy (#60)
- Compile literal attribute name lowercase instead of value (#51)
- Use more memory efficient nth-of-type tracking. (#49)
- Minor cleanup from :nth-child (#48)
- Add support for :nth-child selectors (#47)

## v0.2.0
- Added: `DocumentContentHandlers::end`.

## v0.1.0
- Initial release
