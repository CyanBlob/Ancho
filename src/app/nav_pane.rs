use crate::app::Message;
use crate::app::SimpleButton;

use iced::{Column, Element};

pub struct NavPane {
    pub new: SimpleButton,
    pub login: SimpleButton,
}

impl NavPane {
    pub fn new() -> Self {
        let login = SimpleButton::new("Login".into(), Message::LoginClicked);
        let new_simple = SimpleButton::new("New recipe".into(), Message::NewRecipeClicked);

        Self {
            new: new_simple,
            login: login,
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut column = Column::new();

        column = column.push(self.login.to_button());
        column = column.push(self.new.to_button());

        column.into()
    }
}
