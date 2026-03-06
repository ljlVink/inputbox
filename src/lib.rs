//! Cross-platform input box library.
//!
//! The entry point is the [`InputBox`] struct, which you can configure using
//! the builder pattern and then call `run()` to display the input dialog and
//! get the user's input.
//!
//! # Usage
//!
//! ```rust,ignore
//! use inputbox::InputBox;
//!
//! let input = InputBox::new().title("Title").prompt("Prompt").default_text("Default");
//! let result: Option<String> = input.run();
//! // Or use a specific backend:
//! // let result = input.run_with(&inputbox::backend::Zenity::default());
//! println!("Result: {:?}", result);
//! ```
//!
//! See [`crate::backend`] for details on the available backends and their
//! individual features and limitations.

pub mod backend;

use std::borrow::Cow;

use crate::backend::{Backend, default_backend};

/// Default title for the input box dialog.
pub const DEFAULT_TITLE: &str = "Input";

/// Default prompt message for the input box.
pub const DEFAULT_PROMPT: &str = "Enter text";

/// Default label for the OK/confirm button.
pub const DEFAULT_OK_LABEL: &str = "OK";

/// Default label for the cancel button.
pub const DEFAULT_CANCEL_LABEL: &str = "Cancel";

/// Input mode for the input box.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Standard single-line text input.
    #[default]
    Text,
    /// Password input where characters are hidden.
    Password,
    /// Multi-line text input with a textarea.
    Multiline,
}

impl InputMode {
    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            InputMode::Text => "text",
            InputMode::Password => "password",
            InputMode::Multiline => "multiline",
        }
    }
}

/// An input box configuration.
///
/// # Builder Pattern
///
/// Use the builder pattern to configure the input box:
///
/// ```rust
/// use inputbox::{InputBox, InputMode};
///
/// let input = InputBox::new("Title", "Prompt")
///     .default("default value")
///     .mode(InputMode::Text)
///     .ok_button("Submit")
///     .cancel_label("Quit");
/// ```
#[derive(Clone, Debug)]
pub struct InputBox<'a> {
    /// The title of the dialog window.
    pub title: Option<Cow<'a, str>>,
    /// The prompt text shown to the user.
    pub prompt: Option<Cow<'a, str>>,
    /// Default value pre-filled in the input field.
    pub default: Cow<'a, str>,
    /// Input mode (text, password, or multiline).
    pub mode: InputMode,

    /// The width of the input box.
    pub width: Option<u32>,
    /// The height of the input box.
    pub height: Option<u32>,

    /// Custom label for the cancel button.
    pub cancel_label: Option<Cow<'a, str>>,
    /// Custom label for the OK button.
    pub ok_label: Option<Cow<'a, str>>,

    /// (Multiline mode) Whether to automatically wrap long lines in multiline mode.
    pub auto_wrap: bool,
    /// (Multiline mode) Whether to scroll to the end of the text on open.
    pub scroll_to_end: bool,

    /// Whether to suppress stderr output.
    pub quiet: bool,
}

impl Default for InputBox<'_> {
    fn default() -> Self {
        Self {
            title: None,
            prompt: None,
            default: "".into(),
            mode: InputMode::default(),

            width: None,
            height: None,

            cancel_label: None,
            ok_label: None,

            auto_wrap: true,
            scroll_to_end: false,

            quiet: false,
        }
    }
}

impl<'a> InputBox<'a> {
    /// Creates a new input box.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title of the dialog window.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the prompt text shown to the user.
    pub fn prompt(mut self, prompt: impl Into<Cow<'a, str>>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Sets the default value pre-filled in the input field.
    pub fn default_text(mut self, default: impl Into<Cow<'a, str>>) -> Self {
        self.default = default.into();
        self
    }

    /// Sets the input mode (text, password, or multiline).
    pub fn mode(mut self, mode: InputMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the width of the input box.
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the height of the input box.
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the custom label for the cancel button.
    pub fn cancel_label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.cancel_label = Some(label.into());
        self
    }

    /// Sets the custom label for the OK button.
    pub fn ok_button(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.ok_label = Some(label.into());
        self
    }

    /// Sets whether to automatically wrap long lines in multiline mode.
    pub fn auto_wrap(mut self, auto_wrap: bool) -> Self {
        self.auto_wrap = auto_wrap;
        self
    }

    /// Sets whether to scroll to the end of the text on open.
    pub fn scroll_to_end(mut self, scroll_to_end: bool) -> Self {
        self.scroll_to_end = scroll_to_end;
        self
    }

    /// Sets whether to suppress stderr output.
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Runs the input box with [`default_backend`] for the current platform.
    ///
    /// Returns `Some(input)` if the user clicked OK and entered text, or `None`
    /// if the user clicked Cancel or closed the dialog.
    pub fn run(&self) -> Option<String> {
        default_backend().execute(self)
    }

    /// Runs the input box with the specified backend.
    ///
    /// Returns `Some(input)` if the user clicked OK and entered text, or `None`
    /// if the user clicked Cancel or closed the dialog.
    pub fn run_with<B: Backend>(&self, backend: &B) -> Option<String> {
        backend.execute(self)
    }
}
