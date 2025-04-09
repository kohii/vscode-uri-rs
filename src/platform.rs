/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

#[cfg(target_os = "windows")]
pub const IS_WINDOWS: bool = true;

#[cfg(not(target_os = "windows"))]
pub const IS_WINDOWS: bool = false;

pub fn is_windows() -> bool {
    IS_WINDOWS
}
