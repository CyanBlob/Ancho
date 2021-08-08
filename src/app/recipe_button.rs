use crate::app::Message;

#[allow(unused)]
use iced::{
    button, Button, Text, Element
};

pub struct RecipeButton {
    name: String,
    #[allow(unused)]
    image_url: String
}

impl RecipeButton {
    pub fn new(name: String, image_url: String) -> Self {
        RecipeButton {
            name: name,
            image_url: image_url
        }
    }

    pub fn view(&self) -> Element<Message> {
        Text::new(format!("{}", &self.name)).into()
    }
}