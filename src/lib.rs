/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

mod char_code;
pub mod platform;
mod uri;
mod utils;

pub use platform::is_windows;
pub use uri::{URIChange, URIComponents, UriError, URI};
pub use utils::Utils;
