mod message;
mod nav_pane;
mod paprika;
mod recipe_button;
mod recipe_fetcher;
mod simple_button;
mod style;

use message::Message;
use nav_pane::NavPane;
use recipe_button::RecipeButton;
use recipe_fetcher::RecipeFetcher;
use simple_button::SimpleButton;
use std::sync::{Arc, Mutex};

use iced::{
    button, executor,
    pane_grid::{self, Axis},
    scrollable, Align, Application, Button, Clipboard, Column, Command, Container, Element,
    HorizontalAlignment, Length, PaneGrid, Scrollable, Subscription, Text,
};

pub struct HomePage {
    panes: pane_grid::State<Pane>,
    paprika: Arc<Mutex<paprika::Paprika>>,
    recipes: Arc<Mutex<Vec<paprika_api::api::Recipe>>>,
}

struct Pane {
    pub content: Content,
    pub is_nav_pane: bool,
}

struct Content {
    scroll: scrollable::State,
    split_horizontally: button::State,
    split_vertically: button::State,
    close: button::State,
    nav_pane: NavPane,
    recipes: Arc<Mutex<Vec<paprika_api::api::Recipe>>>,
    recipe_buttons: Vec<RecipeButton>,
}

impl Application for HomePage {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let paprika = paprika::Paprika::new();
        let mutex = std::sync::Mutex::new(paprika);
        let arc = std::sync::Arc::new(mutex);

        let recipes =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::<paprika_api::api::Recipe>::new()));

        // create the State<Pane>, then split it
        let (mut panes, pane) = pane_grid::State::new(Pane::new(true, recipes.clone()));
        let (_split_panes, _split) = panes
            .split(Axis::Vertical, &pane, Pane::new(false, recipes.clone()))
            .expect("Failed to split panes");

        panes.resize(&_split, 0.15);

        (
            HomePage {
                panes: panes,
                paprika: arc.clone(),
                recipes: recipes.clone(),
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Split(axis, pane) => {
                let _result = self
                    .panes
                    .split(axis, &pane, Pane::new(false, self.recipes.clone()));
            }
            Message::Close(_) => todo!(),
            Message::RefreshClicked => {
                let mut mutex = self.paprika.lock().unwrap();
                {
                    mutex.recipe_entries.clear();
                    mutex.recipes.clear();
                    mutex.last_fetched = 0;
                }
                {
                    let mut recipes = self.recipes.lock().unwrap();
                    recipes.clear();
                }
            }
            Message::NewRecipeClicked => {
                println!("New recipe!")
            }
            Message::RecipeClicked(recipe) => {
                println!("Recipe clicked: {:?}", recipe);
            }
            Message::RecipeFetched(recipe) => {
                {
                    match recipe {
                        Some(recipe) => self.recipes.lock().unwrap().push(recipe),
                        None => {}
                    }
                }
                {
                    let count = self.recipes.lock().unwrap().len();
                    println!("Fetched recipe {}", count);
                }
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let pane_grid = PaneGrid::new(&mut self.panes, |id, pane| {
            pane_grid::Content::new(pane.content.view(id, 2, pane.is_nav_pane)).style(style::Pane {
                is_nav_pane: pane.is_nav_pane,
            })
        });
        pane_grid.into()
    }

    fn title(&self) -> String {
        String::from("Ancho Recipe Manager")
    }

    fn subscription(&self) -> Subscription<Message> {
        let paprika = self.paprika.clone();
        let test = iced::Subscription::from_recipe(RecipeFetcher {
            paprika: paprika,
            id: 0,
        });
        test.map(|recipe| Message::RecipeFetched(recipe))
    }
}

impl Pane {
    fn new(is_nav_pane: bool, recipes: Arc<Mutex<Vec<paprika_api::api::Recipe>>>) -> Self {
        Self {
            content: Content::new(recipes),
            is_nav_pane: is_nav_pane,
        }
    }
}

impl Content {
    fn new(recipes: Arc<Mutex<Vec<paprika_api::api::Recipe>>>) -> Self {
        Content {
            scroll: scrollable::State::new(),
            split_horizontally: button::State::new(),
            split_vertically: button::State::new(),
            close: button::State::new(),
            nav_pane: NavPane::new(),
            recipes: recipes.clone(),
            recipe_buttons: Vec::new(),
        }
    }
    fn view(
        &mut self,
        pane: pane_grid::Pane,
        total_panes: usize,
        is_nav_bar: bool,
    ) -> Element<Message> {
        let Content {
            scroll,
            split_horizontally,
            split_vertically,
            close,
            ..
        } = self;

        match is_nav_bar {
            true => self.nav_pane.view(),
            false => {
                let button = |state, label, message| {
                    Button::new(
                        state,
                        Text::new(label)
                            .width(Length::Fill)
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .size(16),
                    )
                    .width(Length::Fill)
                    .padding(8)
                    .on_press(message)
                };

                let mut controls = Column::new()
                    .spacing(5)
                    .max_width(150)
                    .push(button(
                        split_horizontally,
                        "Split horizontally",
                        Message::Split(pane_grid::Axis::Horizontal, pane),
                    ))
                    .push(button(
                        split_vertically,
                        "Split vertically",
                        Message::Split(pane_grid::Axis::Vertical, pane),
                    ));

                if total_panes > 1 {
                    controls = controls.push(button(close, "Close", Message::Close(pane)));
                }

                let mut content = Scrollable::new(scroll)
                    .width(Length::Fill)
                    .spacing(10)
                    .align_items(Align::Center)
                    .push(controls);

                //let _recipes = self.recipes.lock().unwrap();
                let _recipes_arc = self.recipes.clone();
                let _recipes = _recipes_arc.lock().unwrap();

                self.recipe_buttons.clear();
                for recipe in _recipes.iter() {
                    if !recipe.in_trash {
                        let recipe_button;
                        match recipe.image_url.clone() {
                            Some(url) => {
                                recipe_button = recipe_button::RecipeButton::new(
                                    recipe.name.clone(),
                                    url.clone(),
                                )
                            }
                            None => {
                                recipe_button =
                                    recipe_button::RecipeButton::new(recipe.name.clone(), "".into())
                            }
                        }
                        // store the button in Content's owned Vec to allow it to live long enough
                        self.recipe_buttons.push(recipe_button);
                    }
                }
                for recipe_button in &self.recipe_buttons {
                    content = content.push(recipe_button.view());
                }

                Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(5)
                    .center_y()
                    .into()
            }
        }
    }
}
