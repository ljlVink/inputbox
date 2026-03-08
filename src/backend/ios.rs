use std::{
    io,
    ops::Deref,
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use block2::StackBlock;
use objc2::{MainThreadMarker, rc::Retained};
use objc2_core_foundation::{CGFloat, CGRect, CGSize};
use objc2_foundation::{NSArray, NSObjectNSKeyValueCoding, NSRange, NSString, ns_string};
use objc2_ui_kit::{
    NSLayoutConstraint, UIAlertAction, UIAlertActionStyle, UIAlertController,
    UIAlertControllerStyle, UIFont, UITextField, UITextInputTraits, UITextView, UIViewController,
};
use once_cell::sync::OnceCell;

use crate::{DEFAULT_CANCEL_LABEL, DEFAULT_OK_LABEL, DEFAULT_TITLE, InputMode, backend::Backend};

struct Global {
    vc: Retained<UIViewController>,
}

unsafe impl Send for Global {}
unsafe impl Sync for Global {}

static GLOBAL: OnceCell<Global> = OnceCell::new();

/// IOS backend for InputBox.
///
/// # Setup
///
/// To use this backend, you need do either of the following:
///
/// - Call [`IOS::set_view_controller`] to set a custom view controller before
///   creating backend instances. (This enables [`crate::default_backend`] to
///   work out of the box)
///
/// or
///
/// - Use [`IOS::custom`] to create backend instances with a custom view
///   controller.
///
/// # Warnings
///
/// - You can only run this backend on main thread.
/// - You may not call this backend in synchronous fashion (e.g. by calling
///   `execute` directly or using `show`), as it will block the main thread and
///   cause the app to freeze. Always use `execute_async` or `show_with_async`
///   when using this backend.
///
/// # Limitations
///
/// - `width` and `height` only affect the size of the text area when using
///   `InputMode::Multiline`.
/// - `auto_wrap` is ignored (iOS `UITextView` wraps by default).
///
/// # Defaults
///
/// - `title`: `DEFAULT_TITLE`
/// - `prompt`: empty
/// - `cancel_label`: `DEFAULT_CANCEL_LABEL`
/// - `ok_label`: `DEFAULT_OK_LABEL`
pub struct IOS<'a> {
    view_ctrl: &'a UIViewController,
}

impl Default for IOS<'_> {
    fn default() -> Self {
        let _mtm = MainThreadMarker::new().expect("IOS backend must be created on main thread");
        let global = GLOBAL.get().expect("IOS backend not initialized. Call IOS");
        Self {
            view_ctrl: &global.vc,
        }
    }
}

impl<'a> IOS<'a> {
    /// Creates a new IOS backend using the view controller set by `set_view_controller`.
    ///
    /// See the struct-level documentation for more details.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new IOS backend with the given view controller.
    pub fn custom(view_ctrl: &'a UIViewController) -> Self {
        Self { view_ctrl }
    }

    /// Sets the view controller to be used by the backend. This must be called before
    /// creating backend instances
    pub fn set_view_controller(view_ctrl: Retained<UIViewController>) {
        let _mtm =
            MainThreadMarker::new().expect("set_view_controller must be called on main thread");
        let _ = GLOBAL.set(Global { vc: view_ctrl });
    }
}

impl<'a> Backend for IOS<'a> {
    fn execute_async(
        &self,
        input: &crate::InputBox,
        callback: Box<dyn FnOnce(io::Result<Option<String>>) + Send>,
    ) -> io::Result<()> {
        let callback = Arc::new(Mutex::new(Some(callback)));

        let mtm = MainThreadMarker::new().expect("IOS backend can only be used on main thread");

        let title = input.title.as_deref().unwrap_or(DEFAULT_TITLE);
        let prompt_ns = input.prompt.as_deref().map(NSString::from_str);

        let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
            Some(&NSString::from_str(title)),
            prompt_ns.as_deref(),
            UIAlertControllerStyle::Alert,
            mtm,
        );

        let mode = input.mode.clone();
        let default = input.default.to_string();
        let text_view = if mode == InputMode::Multiline {
            let vc = UIViewController::new(mtm);
            let w = input.width.map(|v| v as CGFloat).unwrap_or(270.0);
            let h = input.height.map(|v| v as CGFloat).unwrap_or(150.0);
            let size = CGSize {
                width: w,
                height: h,
            };
            vc.setPreferredContentSize(size);

            let root_view = vc.view().unwrap();

            let text_view = UITextView::new(mtm);
            text_view.setFrame(CGRect {
                origin: Default::default(),
                size,
            });
            text_view.setFont(Some(&UIFont::systemFontOfSize(16.0)));
            text_view.layer().setBorderWidth(0.5);
            text_view.layer().setCornerRadius(5.);

            let text_ns = NSString::from_str(&default);
            text_view.setText(Some(&text_ns));

            if input.scroll_to_end {
                let length = text_ns.length();
                text_view.scrollRangeToVisible(NSRange {
                    location: length,
                    length: 0,
                });
            }

            root_view.addSubview(&text_view);

            text_view.setTranslatesAutoresizingMaskIntoConstraints(false);
            NSLayoutConstraint::activateConstraints(
                &NSArray::from_slice(&[
                    text_view
                        .leadingAnchor()
                        .constraintEqualToAnchor_constant(&root_view.leadingAnchor(), 10.0)
                        .deref(),
                    text_view
                        .trailingAnchor()
                        .constraintEqualToAnchor_constant(&root_view.trailingAnchor(), -10.0)
                        .deref(),
                    text_view
                        .topAnchor()
                        .constraintEqualToAnchor_constant(&root_view.topAnchor(), 10.0)
                        .deref(),
                    text_view
                        .bottomAnchor()
                        .constraintEqualToAnchor_constant(&root_view.bottomAnchor(), -10.0)
                        .deref(),
                ]),
                mtm,
            );

            unsafe {
                alert.setValue_forKey(Some(&vc), ns_string!("contentViewController"));
            }

            Some(text_view)
        } else {
            let mode_clone = mode.clone();
            let default_clone = default.clone();
            alert.addTextFieldWithConfigurationHandler(Some(&StackBlock::new(
                move |field: NonNull<UITextField>| {
                    let field = unsafe { field.as_ref() };
                    field.setText(Some(&NSString::from_str(&default_clone)));
                    if mode_clone == InputMode::Password {
                        field.setSecureTextEntry(true);
                    }
                },
            )));

            None
        };

        let cancel_label = input
            .cancel_label
            .as_deref()
            .unwrap_or(DEFAULT_CANCEL_LABEL);
        let cancel_action = UIAlertAction::actionWithTitle_style_handler(
            Some(&NSString::from_str(cancel_label)),
            UIAlertActionStyle::Cancel,
            Some(&StackBlock::new({
                let callback = callback.clone();
                move |_| {
                    if let Some(cb) = { callback.lock().unwrap().take() } {
                        cb(Ok(None));
                    }
                }
            })),
            mtm,
        );
        alert.addAction(&cancel_action);

        let ok_label = input.ok_label.as_deref().unwrap_or(DEFAULT_OK_LABEL);
        let ok_action = UIAlertAction::actionWithTitle_style_handler(
            Some(&NSString::from_str(ok_label)),
            UIAlertActionStyle::Default,
            Some(&StackBlock::new({
                let alert = alert.clone();
                let callback = callback.clone();
                move |_| {
                    if let Some(cb) = { callback.lock().unwrap().take() } {
                        let text = if let Some(tv) = &text_view {
                            tv.text().to_string()
                        } else {
                            let fields = alert.textFields().unwrap().firstObject().unwrap();
                            fields.text().unwrap().to_string()
                        };
                        cb(Ok(Some(text)));
                    }
                }
            })),
            mtm,
        );
        alert.addAction(&ok_action);

        self.view_ctrl
            .presentViewController_animated_completion(&alert, true, None);

        Ok(())
    }
}
