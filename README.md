# inputbox

[![crates.io](https://img.shields.io/crates/v/inputbox)](https://crates.io/crates/inputbox)
[![docs.rs](https://img.shields.io/docsrs/inputbox)](https://docs.rs/inputbox)
[![CI](https://github.com/Mivik/inputbox/actions/workflows/check.yml/badge.svg)](https://github.com/Mivik/inputbox/actions)

> A cross-platform, native GUI input box for Rust. Yes, finally.

## The Story

Picture this: you're writing a Rust CLI tool and you just want to pop up a little
dialog that says _"hey, what's your name?"_ and take whatever the user types.
Simple, right?

So you look at the ecosystem.

- **`rfd`**: Rusty File Dialogs! Cross-platform! Async! Beautiful! ...opens files. Not text. Files.
- **`native-dialog`**: Again, no input box, just message boxes and file pickers.
- **`tinyfiledialogs`**: `input_box` looks promising! ...but why does my input box turn into a password input when `default` is empty (`Some("")`)? No multiline, no custom labels, no control over backends... Oh and it's a C binding.
- **`dialog`**: Finally, an input box! ...but not for Windows or macOS. It fully depends on tools like `zenity`, `kdialog` or `dialog`.

You stare into the void. The void stares back. You write the dialog in HTML/JS
because at least Electron works on all platforms.

_Not anymore._

## What `inputbox` Does

`inputbox` is a minimal, cross-platform Rust library that shows a native GUI
input dialog and returns what the user typed. It uses whatever is available on
the system. Should work™ most of the time.

## Quick Start

```toml
[dependencies]
inputbox = "0.0.1"
```

```rust
use inputbox::InputBox;

fn main() {
    let result = InputBox::new()
        .title("Greetings")
        .prompt("What's your name?")
        .show()
        .unwrap();

    match result {
        Some(name) => println!("Hello, {name}!"),
        None => println!("Fine, be that way."),
    }
}
```

## Examples

### Password Input

```rust
use inputbox::{InputBox, InputMode};

let result = InputBox::new()
    .title("Login")
    .prompt("Enter your password:")
    .mode(InputMode::Password)
    .show()
    .unwrap();
```

### Custom Buttons and Dimensions

```rust
use inputbox::InputBox;

let result = InputBox::new()
    .title("Confirm")
    .prompt("Are you sure?")
    .ok_label("Yes, do it")
    .cancel_label("No, abort")
    .width(400)
    .height(200)
    .show()
    .unwrap();
```

### Asynchronous Usage

```rust
use inputbox::InputBox;

InputBox::new()
    .title("Async Input")
    .prompt("Enter something:")
    .show_async(|result| {
        println!("Result: {:?}", result);
    });
```

> **Note:** On iOS, you must use async methods. See [the
> documentation](https://docs.rs/inputbox/latest/inputbox/struct.InputBox.html#sync-vs-async)
> for details.

## Features

- **Multiple input modes** — text, password, or multiline
- **Highly customizable** — title, prompt, button labels, dimensions, and more
- **Works on most platforms** — Windows, macOS, Linux, Android, and iOS
- **Pluggable backends** — use a specific backend or let the library pick
- **Synchronous and asynchronous** — safe sync on most platforms, async required on iOS

## Backends

| Backend     | Platform | How it works                                    |
| ----------- | -------- | ----------------------------------------------- |
| `PSScript`  | Windows  | PowerShell + WinForms, no extra install needed  |
| `JXAScript` | macOS    | `osascript` JXA, built into the OS              |
| `Android`   | Android  | AAR + JNI to show an Android AlertDialog        |
| `IOS`       | iOS      | UIKit alert                                     |
| `Yad`       | Linux    | [`yad`](https://github.com/v1cont/yad)          |
| `Zenity`    | Linux    | `zenity` — fallback on GNOME systems            |

### Linux Installation

```bash
# Arch Linux
sudo pacman -S yad

# Debian/Ubuntu
sudo apt install yad

# Fedora
sudo dnf install yad

# or use zenity (usually pre-installed on GNOME)
sudo apt install zenity
```

The `show()` method automatically picks the best backend for the current platform.
You can also specify one explicitly via `show_with()`.

## License

MIT
