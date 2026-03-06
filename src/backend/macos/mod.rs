use std::{borrow::Cow, path::Path, process::Command};

use crate::{
    DEFAULT_CANCEL_LABEL, DEFAULT_OK_LABEL, InputBox,
    backend::{Backend, run_command},
};

const JXA_SCRIPT: &str = include_str!("inputbox.jxa.js");

/// JXA (JavaScript for Automation) backend for macOS.
///
/// This backend uses the `osascript` command to execute a JavaScript for
/// Automation (JXA) script that displays an input dialog.
///
/// # Limitations
///
/// - Does not support [`InputMode::Multiline`] mode (falls back to single-line
///   input).
///
/// # Caveats
///
/// - If [`ok_label`](InputBox::ok_label) or
///   [`cancel_label`](InputBox::cancel_label) is not set, [`DEFAULT_OK_LABEL`]
///   and [`DEFAULT_CANCEL_LABEL`] will be used, which may not be localized.
#[derive(Clone, Debug)]
pub struct JXAScript {
    path: Cow<'static, Path>,
}

impl JXAScript {
    /// Creates a new backend using the default `osascript` command.
    pub fn new() -> Self {
        Self {
            path: Path::new("osascript").into(),
        }
    }

    /// Creates a new backend with a custom path to the osascript executable.
    pub fn custom(path: impl Into<Cow<'static, Path>>) -> Self {
        Self { path: path.into() }
    }
}

impl Default for JXAScript {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for JXAScript {
    fn execute(&self, input: &InputBox) -> Option<String> {
        let cancel_label = input
            .cancel_label
            .as_deref()
            .unwrap_or(DEFAULT_CANCEL_LABEL);
        let ok_label = input.ok_label.as_deref().unwrap_or(DEFAULT_OK_LABEL);
        let value = serde_json::json!({
            "title": input.title,
            "prompt": input.prompt,
            "default": input.default,
            "mode": input.mode.as_str(),
            "width": input.width,
            "height": input.height,
            "cancel_label": cancel_label,
            "ok_label": ok_label,
            "auto_wrap": input.auto_wrap,
            "scroll_to_end": input.scroll_to_end,
        });
        let stdin = Some(value.to_string());

        let mut cmd = Command::new(&*self.path);
        cmd.args(["-l", "JavaScript", "-e", JXA_SCRIPT]);

        run_command(&mut cmd, stdin.as_deref(), input.quiet)
    }
}
