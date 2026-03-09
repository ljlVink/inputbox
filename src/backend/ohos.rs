//! OHOS (OpenHarmony) backend for InputBox.
//!
//! This backend uses NAPI to communicate with ArkTS layer for showing native dialogs.

use std::io;
use std::sync::OnceLock;

use napi_derive_ohos::napi;
use napi_ohos::{
    bindgen_prelude::*,
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};

use super::Backend;
use crate::{DEFAULT_CANCEL_LABEL, DEFAULT_OK_LABEL, DEFAULT_TITLE, InputBox, InputMode};

static REQUEST_CALLBACK: OnceLock<
    ThreadsafeFunction<InputBoxRequest, (), InputBoxRequest, napi_ohos::Status, false, false, 16>,
> = OnceLock::new();

#[napi(object)]
#[derive(Clone)]
pub struct InputBoxRequest {
    /// Callback pointer as i64 (leaked Box<dyn FnOnce>)
    pub callback_ptr: i64,
    pub title: String,
    pub prompt: Option<String>,
    pub default_value: String,
    pub mode: String,
    pub ok_label: String,
    pub cancel_label: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub auto_wrap: bool,
    pub scroll_to_end: bool,
}

#[allow(dead_code)]
#[napi(object)]
pub struct InputBoxResponse {
    /// Callback pointer as i64 (to recover the leaked Box)
    pub callback_ptr: i64,
    pub text: Option<String>,
    pub error: Option<String>,
}

/// OHOS backend for InputBox.
///
/// This backend uses NAPI to call into ArkTS layer for showing native dialogs.
///
/// # Setup
///
/// To use this backend, you need to:
///
/// 1. Import this native library in your ArkTS code.
/// 2. Call [`register_inputbox_callback`] to register the request handler.
/// 3. Implement the dialog display logic in ArkTS.
///
/// # ArkTS Integration Example
///
/// ```typescript
/// import inputbox from 'libinputbox.so';
///
/// // Register the callback handler
/// inputbox.registerInputboxCallback((request: InputBoxRequest) => {
///   // Show your custom dialog using request.title, request.prompt, etc.
///   // When user confirms or cancels, call:
///   inputbox.onInputboxResponse({
///     callbackPtr: request.callbackPtr,
///     text: userInput,  // or null if cancelled
///     error: null
///   });
/// });
/// ```
///
/// # Limitations
///
/// - `width` and `height` are hints only and may be ignored.
///
/// # Defaults
///
/// - `title`: `DEFAULT_TITLE`
/// - `prompt`: empty
/// - `cancel_label`: `DEFAULT_CANCEL_LABEL`
/// - `ok_label`: `DEFAULT_OK_LABEL`
#[derive(Default, Debug, Clone)]
pub struct OHOS {
    _priv: (),
}

impl OHOS {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Backend for OHOS {
    fn execute_async(
        &self,
        input: &InputBox,
        callback: Box<dyn FnOnce(io::Result<Option<String>>) + Send>,
    ) -> io::Result<()> {
        let tsfn = REQUEST_CALLBACK.get().ok_or_else(|| {
            io::Error::other(
                "OHOS callback not registered. Call registerInputboxCallback from ArkTS first.",
            )
        })?;

        // Leak the callback and pass its pointer to ArkTS
        // ArkTS will call on_inputbox_response with this pointer to invoke the callback
        let callback_ptr = Box::into_raw(Box::new(callback)) as i64;

        let request = InputBoxRequest {
            callback_ptr,
            title: input.title.as_deref().unwrap_or(DEFAULT_TITLE).to_string(),
            prompt: input.prompt.as_deref().map(|s| s.to_string()),
            default_value: input.default.to_string(),
            mode: match input.mode {
                InputMode::Text => "text",
                InputMode::Password => "password",
                InputMode::Multiline => "multiline",
            }
            .to_string(),
            ok_label: input
                .ok_label
                .as_deref()
                .unwrap_or(DEFAULT_OK_LABEL)
                .to_string(),
            cancel_label: input
                .cancel_label
                .as_deref()
                .unwrap_or(DEFAULT_CANCEL_LABEL)
                .to_string(),
            width: input.width,
            height: input.height,
            auto_wrap: input.auto_wrap,
            scroll_to_end: input.scroll_to_end,
        };

        // Send request to ArkTS layer
        let status = tsfn.call(request, ThreadsafeFunctionCallMode::NonBlocking);
        if status != napi_ohos::Status::Ok {
            // Recover and call the callback with error if send failed
            let callback = unsafe {
                Box::from_raw(
                    callback_ptr as *mut Box<dyn FnOnce(io::Result<Option<String>>) + Send>,
                )
            };
            callback(Err(io::Error::other(format!(
                "Failed to send request to ArkTS: {:?}",
                status
            ))));
        }

        Ok(())
    }
}

/// Register the ArkTS callback handler for input box requests.
///
/// This function must be called from ArkTS before using the InputBox API. The
/// callback will receive [`InputBoxRequest`] objects when `show()` is called.
///
/// # Example
///
/// ```typescript
/// import inputbox from 'libinputbox.so';
///
/// inputbox.registerInputboxCallback((request) => {
///   // Display dialog and handle user input
/// });
/// ```
#[allow(dead_code)]
#[napi]
pub fn register_inputbox_callback(
    callback: Function<InputBoxRequest, ()>,
) -> napi_ohos::Result<()> {
    let tsfn = callback
        .build_threadsafe_function()
        .max_queue_size::<16>()
        .build()?;

    REQUEST_CALLBACK
        .set(tsfn)
        .map_err(|_| napi_ohos::Error::from_reason("Callback already registered"))?;

    Ok(())
}

/// Handle response from ArkTS layer.
///
/// This function should be called from ArkTS when the user completes or cancels
/// the input dialog.
///
/// # Example
///
/// ```typescript
/// import inputbox from 'libinputbox.so';
///
/// // When user clicks OK:
/// inputbox.onInputboxResponse({
///   callbackPtr: request.callbackPtr,
///   text: userInputText,
///   error: null
/// });
///
/// // When user clicks Cancel:
/// inputbox.onInputboxResponse({
///   callbackPtr: request.callbackPtr,
///   text: null,
///   error: null
/// });
/// ```
#[allow(dead_code)]
#[napi]
pub fn on_inputbox_response(response: InputBoxResponse) {
    // Recover the leaked callback from the pointer passed by ArkTS
    let callback = unsafe {
        Box::from_raw(
            response.callback_ptr as *mut Box<dyn FnOnce(io::Result<Option<String>>) + Send>,
        )
    };

    if let Some(error) = response.error {
        callback(Err(io::Error::other(error)));
    } else {
        callback(Ok(response.text));
    }
}
