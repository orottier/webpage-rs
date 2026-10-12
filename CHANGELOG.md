# Version History

## Unreleased

- Reject non-HTTP(S) URLs in `HTTP::fetch` and `Webpage::from_url`
- Avoid a panic when parsing malformed OpenGraph media properties
- Raise the minimum supported Rust version to 1.88
- Update `html5ever` and `markup5ever_rcdom` dependencies

## Version 2.0.1

- Specified the MSRV rust-version (1.63)
- Updated dependencies

## Version 2.0.0

Breaking:
- Changed all structs to be `non_exhaustive`
- Moved all structs to the crate root (no re-exports)

New features:
- Added the ability to specify HTTP request headers
- Collect all links/anchors of the HTML document
