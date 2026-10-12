# Version History

## Unreleased

Breaking:
- Reject non-HTTP(S) and scheme-less initial URLs in `HTTP::fetch` and `Webpage::from_url`. Use `HTTP::fetch_any_scheme` or `Webpage::from_url_any_scheme` to retain the previous behavior for trusted URLs. Redirects continue to follow libcurl's defaults.
- Raise the minimum supported Rust version to 1.88.

Dependencies:
- Update `html5ever` and `markup5ever_rcdom` to the matching 0.39 releases.

Fixes:
- Avoid a panic when parsing malformed OpenGraph media properties.

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
