use shared::{DownMsg, Message, UpMsg};
use zoon::{eprintln, *};

// ------ ------
//    States
// ------ ------

#[static_ref]
fn username() -> &'static Mutable<String> {
    Mutable::new("John".to_owned())
}

#[static_ref]
fn messages() -> &'static MutableVec<Message> {
    MutableVec::new()
}

#[static_ref]
fn new_message_text() -> &'static Mutable<String> {
    Mutable::new(String::new())
}

#[static_ref]
fn connection() -> &'static Connection<UpMsg, DownMsg> {
    Connection::new(|DownMsg::MessageReceived(message), _| {
        messages().lock_mut().push_cloned(message);
    })
}

// ------ ------
//   Commands
// ------ ------

fn set_username(name: String) {
    username().set(name);
}

fn set_new_message_text(text: String) {
    new_message_text().set(text);
}

fn send_message() {
    Task::start(async {
        connection()
            .send_up_msg(UpMsg::SendMessage(Message {
                username: username().get_cloned(),
                text: new_message_text().take(),
            }))
            .await
            .unwrap_or_else(|error| eprintln!("Failed to send message: {:?}", error))
    });
}

// ------ ------
//     View
// ------ ------

fn root() -> impl Element {
    Column::new()
        .s(Padding::new().all(30))
        .s(Spacing::new(20))
        .item(received_messages())
        .item(new_message_panel())
        .item(username_panel())
}

// ------ received_messages ------

fn received_messages() -> impl Element {
    Column::new().items_signal_vec(messages().signal_vec_cloned().map(received_message))
}

fn received_message(message: Message) -> impl Element {
    Column::new()
        .s(Padding::new().all(10))
        .s(Spacing::new(6))
        .item(
            El::new()
                .s(Font::new().bold().color(NamedColor::Gray10).size(17))
                .child(message.username),
        )
        .item(
            El::new()
                .s(Font::new().color(NamedColor::Gray8).size(17))
                .child(message.text),
        )
}

// ------ new_message_panel ------

fn new_message_panel() -> impl Element {
    Row::new().item(new_message_input()).item(send_button())
}

fn new_message_input() -> impl Element {
    TextInput::new()
        .s(Padding::new().x(10))
        .s(Font::new().size(17))
        .focused()
        .on_change(set_new_message_text)
        .label_hidden("New message text")
        .placeholder(Placeholder::new("Message"))
        .on_key_down(|event| event.if_key(Key::Enter, send_message))
        .text_signal(new_message_text().signal_cloned())
}

fn send_button() -> impl Element {
    let (hovered, hovered_signal) = Mutable::new_and_signal(false);
    Button::new()
        .s(Padding::new().all(10))
        .s(Background::new()
            .color_signal(hovered_signal.map_bool(|| NamedColor::Green5, || NamedColor::Green2)))
        .s(Font::new().color(NamedColor::Gray10).size(20))
        .on_hovered_change(move |is_hovered| hovered.set(is_hovered))
        .on_press(send_message)
        .label("Send")
}

// ------ username_panel ------

fn username_panel() -> impl Element {
    let id = "username_input";
    Row::new()
        .item(username_input_label(id))
        .item(username_input(id))
}

fn username_input_label(id: &str) -> impl Element {
    Label::new()
        .s(Font::new().color(NamedColor::Gray10))
        .s(Padding::new().all(10))
        .for_input(id)
        .label("Username:")
}

fn username_input(id: &str) -> impl Element {
    TextInput::new()
        .s(Padding::new().x(10))
        .id(id)
        .on_change(set_username)
        .placeholder(Placeholder::new("Joe"))
        .text_signal(username().signal_cloned())
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn start() {
    start_app("app", root);
    connection();
}
