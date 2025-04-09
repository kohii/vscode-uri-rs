# vscode-uri-rs

A Rust implementation of [microsoft/vscode-uri](https://github.com/microsoft/vscode-uri), providing URI parsing, manipulation, and utility functions for handling URIs and file paths.

## Features

- URI parsing and manipulation following RFC 3986 standards
- File URI handling with cross-platform support
- Path manipulation utilities (join, resolve, dirname, basename, extname)
- Proper encoding/decoding of URI components
- Windows and Unix path handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vscode-uri-rs = "0.1.0"
```

## Usage

### Basic URI Parsing

```rust
use vscode_uri_rs::URI;

fn main() {
    // Parse a URI
    let uri = URI::parse("https://code.visualstudio.com/docs/extensions/overview#frag");
    
    // Access URI components
    println!("Scheme: {}", uri.scheme());
    println!("Authority: {}", uri.authority());
    println!("Path: {}", uri.path());
    println!("Query: {}", uri.query());
    println!("Fragment: {}", uri.fragment());
    
    // Convert back to string
    println!("URI: {}", uri.to_string(false));
}
```

### File URIs

```rust
use vscode_uri_rs::URI;
use std::path::Path;

fn main() {
    // Create a file URI from a path
    let uri = URI::file("/users/me/projects/");
    
    // Convert URI back to filesystem path
    let path = uri.fs_path();
    
    println!("URI: {}", uri);
    println!("Path: {}", path.display());
}
```

### URI Manipulation

```rust
use vscode_uri_rs::{URI, URIChange};

fn main() {
    let uri = URI::parse("https://example.com/path");
    
    // Create a new URI by changing components
    let new_uri = uri.with(URIChange {
        scheme: Some("http".to_string()),
        path: Some("/newpath".to_string()),
        ..Default::default()
    });
    
    println!("New URI: {}", new_uri);
}
```

### Path Utilities

```rust
use vscode_uri_rs::{URI, utils};

fn main() {
    let uri = URI::parse("https://example.com/path/to/file.txt");
    
    // Join paths
    let joined = utils::join_path(&uri, &["subdir", "file.js"]);
    println!("Joined: {}", joined);
    
    // Resolve paths (handles .. and .)
    let resolved = utils::resolve_path(&uri, &["../other", "./file.js"]);
    println!("Resolved: {}", resolved);
    
    // Get directory name
    let dir = utils::dirname(&uri);
    println!("Directory: {}", dir);
    
    // Get basename
    let base = utils::basename(&uri);
    println!("Basename: {}", base);
    
    // Get extension
    let ext = utils::extname(&uri);
    println!("Extension: {}", ext);
}
```

## License

MIT

## Credits

This is a Rust port of the [microsoft/vscode-uri](https://github.com/microsoft/vscode-uri) TypeScript library.
