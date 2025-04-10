/*
 * Rust implementation of vscode-uri
 * https://github.com/microsoft/vscode-uri
 */

#[cfg(target_os = "windows")]
pub const IS_WINDOWS: bool = true;

#[cfg(not(target_os = "windows"))]
pub const IS_WINDOWS: bool = false;

pub fn is_windows() -> bool {
    #[cfg(test)]
    {
        test_utils::is_windows()
    }
    #[cfg(not(test))]
    {
        IS_WINDOWS
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    use std::sync::atomic::{AtomicBool, Ordering};

    static IS_WINDOWS: AtomicBool = AtomicBool::new(false);

    pub fn is_windows() -> bool {
        IS_WINDOWS.load(Ordering::SeqCst)
    }

    pub fn set_is_windows(value: bool) {
        IS_WINDOWS.store(value, Ordering::SeqCst);
    }
}
