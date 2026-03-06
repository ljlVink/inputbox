use inputbox::{InputBox, InputMode};

fn main() {
    let input = InputBox::new()
        .title("Title")
        .prompt("Enter something")
        .default_text("Default value")
        .mode(InputMode::Text)
        .width(400)
        .height(200)
        .cancel_label("Nope")
        .ok_button("Fine");

    let result = input.run();
    println!("Result: {:?}", result);
}
