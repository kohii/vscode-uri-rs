/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

mod uri;
mod utils;
mod char_code;
mod platform;

pub use uri::{URI, URIChange, URIComponents};
pub use utils::Utils;
