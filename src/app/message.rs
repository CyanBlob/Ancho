use iced::pane_grid::{self};

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    Close(pane_grid::Pane),
    NewRecipeClicked,
    RecipeFetched(Option<paprika_api::api::Recipe>),
    RecipeClicked(String),
    LoginClicked,
    AccountChanged(String, String)
}
