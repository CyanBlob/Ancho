use crate::app::Message;
use crate::app::SimpleButton;

use iced::{Column, Element};

pub struct NavPane {
    pub refresh: SimpleButton,
    pub new: SimpleButton,
}

impl NavPane {
    pub fn new() -> Self {
        let refresh_simple = SimpleButton::new("Refresh recipes".into(), Message::RefreshClicked);
        let new_simple = SimpleButton::new("New recipes".into(), Message::NewRecipeClicked);

        Self {
            refresh: refresh_simple,
            new: new_simple,
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut column = Column::new();

        column = column.push(self.refresh.to_button());
        column = column.push(self.new.to_button());

        column.into()
    }
}
