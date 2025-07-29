# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of [microsoft/vscode-uri](https://github.com/microsoft/vscode-uri), providing URI parsing, manipulation, and utility functions for handling URIs and file paths. The crate is used by VS Code and its extensions, ported to Rust.

## Development Commands

### Build
```bash
cargo build
```

### Test
```bash
# Run all tests with the test-utils feature enabled
cargo test --features test-utils

# Run a single test
cargo test test_name --features test-utils
```

### Lint and Format
```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Run clippy linter
cargo clippy
```

## Architecture

The crate is organized into the following modules:

- **uri.rs**: Core URI implementation with parsing, validation, and manipulation functionality
  - Implements RFC 3986 compliant URI parsing
  - Handles percent encoding/decoding
  - Provides platform-specific file path conversion

- **utils.rs**: Utility functions for path manipulation (join_path, resolve_path, dirname, basename, extname)
  - All utility functions use POSIX path manipulation rules

- **platform.rs**: Platform detection and test utilities
  - Contains `is_windows()` function for platform-specific behavior
  - `test_utils` module (behind `test-utils` feature) for mocking platform in tests

- **char_code.rs**: Character code constants used for URI encoding

## Important Implementation Notes

1. The `test-utils` feature flag must be enabled when running tests that need to mock platform behavior (Windows vs non-Windows).

2. The crate uses lazy_static for regex patterns and encoding tables to ensure they're compiled only once.

3. File URIs are handled differently on Windows vs other platforms:
   - Windows: Uses backslashes in file paths, handles UNC paths
   - Other platforms: Uses forward slashes

4. The original TypeScript implementation is located in the `vscode-uri/` directory for reference.