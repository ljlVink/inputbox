use std::{ops::Deref, ptr::NonNull, sync::mpsc};

use block2::StackBlock;
use objc2::MainThreadMarker;
use objc2_core_foundation::{CGFloat, CGRect, CGSize};
use objc2_foundation::{
    NSArray, NSDate, NSObjectNSKeyValueCoding, NSRange, NSRunLoop, NSString, ns_string,
};
use objc2_ui_kit::{
    NSLayoutConstraint, UIAlertAction, UIAlertActionStyle, UIAlertController,
    UIAlertControllerStyle, UIFont, UITextField, UITextInputTraits, UITextView, UIViewController,
};

use crate::{DEFAULT_CANCEL_LABEL, DEFAULT_OK_LABEL, DEFAULT_TITLE, InputMode, backend::Backend};

/// IOS backend for InputBox.
///
/// # Limitations
///
/// - `width` option is mostly constrained by iOS alert limits (typically maxes out around 270pt on iPhones),
///   but `height` is respected in Multiline mode.
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

impl<'a> IOS<'a> {
    /// Creates a new IOS backend with the given view controller.
    pub fn new(view_ctrl: &'a UIViewController) -> Self {
        Self { view_ctrl }
    }
}

impl<'a> Backend for IOS<'a> {
    fn execute(&self, input: &crate::InputBox) -> Option<String> {
        let mtm = MainThreadMarker::new().unwrap();

        let title = input.title.as_deref().unwrap_or(DEFAULT_TITLE);
        let prompt_ns = input.prompt.as_deref().map(NSString::from_str);

        let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
            Some(&NSString::from_str(title)),
            prompt_ns.as_deref(),
            UIAlertControllerStyle::Alert,
            mtm,
        );

        let (tx, rx) = mpsc::sync_channel::<Option<String>>(1);

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
        let tx_cancel = tx.clone();
        let cancel_action = UIAlertAction::actionWithTitle_style_handler(
            Some(&NSString::from_str(cancel_label)),
            UIAlertActionStyle::Cancel,
            Some(&StackBlock::new(move |_| {
                let _ = tx_cancel.send(None);
            })),
            mtm,
        );
        alert.addAction(&cancel_action);

        let ok_label = input.ok_label.as_deref().unwrap_or(DEFAULT_OK_LABEL);
        let tx_ok = tx.clone();
        let ok_action = UIAlertAction::actionWithTitle_style_handler(
            Some(&NSString::from_str(ok_label)),
            UIAlertActionStyle::Default,
            Some(&StackBlock::new({
                let alert = alert.clone();
                move |_| {
                    let text = if let Some(tv) = &text_view {
                        tv.text().to_string()
                    } else {
                        let fields = alert.textFields().unwrap().firstObject().unwrap();
                        fields.text().unwrap().to_string()
                    };
                    let _ = tx_ok.send(Some(text));
                }
            })),
            mtm,
        );
        alert.addAction(&ok_action);

        self.view_ctrl
            .presentViewController_animated_completion(&alert, true, None);

        let run_loop = NSRunLoop::currentRunLoop();
        loop {
            match rx.try_recv() {
                Ok(res) => return res,
                Err(mpsc::TryRecvError::Disconnected) => return None,
                Err(mpsc::TryRecvError::Empty) => {
                    let date = NSDate::dateWithTimeIntervalSinceNow(0.05);
                    run_loop.runUntilDate(&date);
                }
            }
        }
    }
}
