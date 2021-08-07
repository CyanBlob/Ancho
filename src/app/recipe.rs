use crate::app::Message;
use paprika_api;

use iced::{button, Button, Column, Element, Svg, Text};

pub struct RecipeView {
    pub recipe: paprika_api::api::Recipe,
    pub state: button::State,
    recipe_interior: RecipeInterior,
}

struct RecipeInterior {
    text: Text,
    image: Svg,
}

impl RecipeInterior {
    fn new(text: String, image_path: &str) -> Self {
        Self {
            text: Text::new(text),
            image: Svg::from_path(image_path),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut column = Column::new();
        // TODO: Why do I have to clone these?
        column = column.push(self.image.clone());
        column = column.push(self.text.clone());
        column.into()
    }
}

impl RecipeView {
    pub fn new(recipe: paprika_api::api::Recipe) -> Self {

        let name = recipe.name.clone();
        Self {
            recipe: recipe,
            state: button::State::new(),
            recipe_interior: RecipeInterior::new(name, "resources/recipe.svg"),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut column = Column::new();

        let test_button = Button::new(&mut self.state, self.recipe_interior.view())
            .on_press(Message::RecipeClicked(self.recipe.clone()));

        column = column.push(test_button);

        column.into()
    }
}
