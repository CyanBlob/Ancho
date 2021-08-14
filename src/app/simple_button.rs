use crate::app::Message;

use iced::{button, Button, Text};

pub struct SimpleButton {
    pub state: button::State,
    pub text: String,
    pub on_pressed: Message,
}

impl SimpleButton {
    pub fn new(text: String, on_pressed: Message) -> Self {
        Self {
            state: button::State::new(),
            text: text.into(),
            on_pressed: on_pressed,
        }
    }

    pub fn to_button(&mut self) -> button::Button<Message> {
        Button::new(&mut self.state, Text::new(&self.text))
            .on_press(self.on_pressed.clone())
            .into()
    }
}
