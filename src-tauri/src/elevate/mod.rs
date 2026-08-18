//! Shared privileged operations. These are the only entry points the
//! elevated helper executes; the main process either runs them directly (when
//! already elevated) or forwards them over a named pipe.

pub mod ops;
pub mod pipe;
pub mod runner;

pub fn is_elevated() -> bool {
    unsafe { windows::Win32::UI::Shell::IsUserAnAdmin().as_bool() }
}
