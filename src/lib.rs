/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

mod char_code;
mod platform;
mod uri;
mod utils;

pub use uri::{URIChange, URIComponents, URI};
pub use utils::Utils;
