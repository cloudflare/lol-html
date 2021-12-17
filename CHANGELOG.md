# Changelog

## Unreleased
- Change string allocators in the C API to return `lol_html_str_t`, not `lol_html_str_t*`. This was necessary to fix a memory leak in `lol_html_str_free`. (#115)

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
