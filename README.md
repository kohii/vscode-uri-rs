# vscode-uri-rs

A Rust implementation of [microsoft/vscode-uri](https://github.com/microsoft/vscode-uri), providing URI parsing, manipulation, and utility functions for handling URIs and file paths.

This crate contains the URI implementation that is used by VS Code and its extensions, ported to Rust. It has support for parsing a string into `scheme`, `authority`, `path`, `query`, and `fragment` URI components as defined in [RFC 3986](http://tools.ietf.org/html/rfc3986).

```text
foo://example.com:8042/over/there?name=ferret#nose
\_/   \______________/\_________/ \_________/ \__/
 |           |            |            |        |
scheme   authority       path        query   fragment
 |    _____________________|__
/ \ /                         \
urn:example:animal:ferret:nose
```

## Usage

```rust
use vscode_uri_rs::Uri;

// Parse a URI from string
let uri = Uri::parse("https://code.visualstudio.com/docs/extensions/overview#frag").unwrap();
assert_eq!(uri.scheme(), "https");
assert_eq!(uri.authority(), "code.visualstudio.com");
assert_eq!(uri.path(), "/docs/extensions/overview");
assert_eq!(uri.query(), "");
assert_eq!(uri.fragment(), "frag");
assert_eq!(uri.to_string(), "https://code.visualstudio.com/docs/extensions/overview#frag");

// Create a URI from a fs path
let uri = Uri::from_file("/users/me/rust-projects/");
assert_eq!(uri.scheme(), "file");
assert_eq!(uri.authority(), "");
assert_eq!(uri.path(), "/users/me/rust-projects/");
assert_eq!(uri.query(), "");
assert_eq!(uri.fragment(), "");
assert_eq!(uri.to_string(), "file:///users/me/rust-projects/");
```

## Utils

This crate also provides utility functions for path manipulation, similar to the original JavaScript implementation:

* `join_path(uri, paths): Uri` - Join a URI with path segments
* `resolve_path(uri, paths): Uri` - Resolve a URI with path segments
* `dirname(uri): String` - Get the directory name of a URI's path
* `basename(uri): String` - Get the base name of a URI's path
* `extname(uri): String` - Get the extension of a URI's path

All utility functions use POSIX path manipulation rules.

## License

MIT

## Acknowledgments

This is a Rust port of the [microsoft/vscode-uri](https://github.com/microsoft/vscode-uri) project. Thanks to the original authors and contributors.
