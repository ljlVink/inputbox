//! Backend implementations for different platforms.
//!
//! This module provides platform-specific backends for showing input dialogs:
//! - [`Zenity`] for Unix-like systems (Linux, BSD)
//! - `PSScript` for Windows (Windows only)
//! - `AppleScript` for macOS (macOS only)
//!
//! # Choosing a Backend
//!
//! The appropriate backend is automatically selected based on your target OS.
//! Use the `Backend` trait to run an [`InputBox`] with any backend:
//!
//! ```rust,ignore
//! use inputbox::{InputBox, InputMode};
//! use inputbox::backend::Backend;
//!
//! let input = InputBox::new("Title", "Enter something");
//! let result = input.run_with(&inputbox::backend::Zenity::default());
//! ```

use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::InputBox;

mod general;
use cfg_if::cfg_if;
pub use general::{Yad, Zenity};

#[cfg(target_os = "windows")]
mod windows;
use which::which;
#[cfg(target_os = "windows")]
pub use windows::PSScript;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::JXAScript;

/// Executes a command and returns its output.
///
/// Internal helper function that runs a command with optional stdin input
/// and returns stdout on success or None on failure.
fn run_command(cmd: &mut Command, stdin: Option<&str>, quiet: bool) -> Option<String> {
    if stdin.is_some() {
        cmd.stdin(Stdio::piped());
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(if quiet {
        Stdio::null()
    } else {
        Stdio::inherit()
    });
    let mut child = cmd.spawn().ok()?;
    if let Some(input) = stdin {
        child.stdin.take()?.write_all(input.as_bytes()).ok()?;
    }
    let output = child.wait_with_output().ok()?;

    if output.status.success() {
        Some(
            String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_string(),
        )
    } else {
        None
    }
}

/// Trait for platform-specific input box backends.
///
/// Implement this trait to add support for different dialog implementations.
/// See [`Zenity`] for an example (other backends are available on their
/// respective platforms).
pub trait Backend {
    /// Executes the input box with the given configuration.
    ///
    /// Returns `Some(input)` if the user confirmed the dialog,
    /// or `None` if the user cancelled or the dialog failed.
    fn execute(&self, input: &InputBox) -> Option<String>;
}

pub fn default_backend() -> Box<dyn Backend> {
    if which("yad").is_ok() {
        return Box::new(Yad::default());
    }

    cfg_if! {
        if #[cfg(target_os = "windows")] {
            Box::new(PSScript::default())
        } else if #[cfg(target_os = "macos")] {
            Box::new(JXAScript::default())
        } else {
            Box::new(Zenity::default())
        }
    }
}
