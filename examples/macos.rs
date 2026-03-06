use inputbox::{InputBox, InputMode, backend::JXAScript};

fn main() {
    let input = InputBox::new()
        .title("Title")
        .prompt("Enter something")
        .default_text("Default value")
        .mode(InputMode::Text)
        .width(400)
        .height(200)
        .cancel_label("Cancel")
        .ok_button("OK");

    let result = input.run_with(&JXAScript::default());
    println!("Result: {:?}", result);
}
