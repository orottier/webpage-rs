# Version History

## Unreleased

### Security patch / behaviorally breaking

Restricted `HTTP::fetch` and `Webpage::from_url` to public HTTP(S) destinations

- URLs must have an explicit HTTP or HTTPS scheme; other schemes and scheme-less URLs are rejected.
- URLs are rejected if any resolved address is non-public, including private, loopback, and link-local addresses.
- Redirects are checked for these constraints too

Use `HTTP::fetch_unrestricted` or `Webpage::from_url_unrestricted` to avoid the checks for trusted URLs.

### Other changes

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
