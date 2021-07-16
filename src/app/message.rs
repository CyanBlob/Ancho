use iced::{
    pane_grid::{self}
};

#[derive(Debug, Clone)]
pub enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    Close(pane_grid::Pane),
    RefreshClicked,
    NewRecipeClicked,
    RecipeFetched(Option<paprika_api::api::Recipe>),
}