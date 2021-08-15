use crate::app::Message;

#[allow(unused)]
use iced::{button, Button, Element, Text};

pub struct RecipeButton {
    name: String,
    recipe_uid: String,
    #[allow(unused)]
    image_url: String,
    pub state: button::State,
}

impl RecipeButton {
    pub fn new(name: String, recipe_uid: String, image_url: String) -> Self {
        RecipeButton {
            name: name,
            recipe_uid: recipe_uid,
            image_url: image_url,
            state: button::State::new()
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        Button::new(&mut self.state, Text::new(&self.name)).on_press(Message::RecipeClicked(self.recipe_uid.clone())).into()
        //Text::new(format!("{}", &self.name)).into()
    }
}
