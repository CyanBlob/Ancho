mod account;
mod message;
mod nav_pane;
mod paprika;
mod recipe_button;
mod recipe_fetcher;
mod simple_button;
mod style;

use chrono::Utc;
use message::Message;
use nav_pane::NavPane;
use recipe_button::RecipeButton;
use recipe_fetcher::RecipeFetcher;
use simple_button::SimpleButton;
use std::sync::{Arc, Mutex};

use edit;

use iced::{
    executor,
    pane_grid::{self, Axis},
    scrollable, Align, Application, Clipboard, Column, Command, Container, Element, Length,
    PaneGrid, Scrollable, Subscription,
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

            Message::NewRecipeClicked => {
                println!("New recipe!");
                let mut recipe = paprika_api::api::Recipe::default();

                recipe.created = Utc::now().format("%Y-%m-%d %H:%M:%S".into()).to_string();

                let serialized = serde_json::to_string_pretty(&recipe).unwrap();

                let edited = edit::edit(serialized).unwrap();

                let mut edited_recipe: paprika_api::api::Recipe =
                    serde_json::from_str(&edited).unwrap();

                {
                    let _paprika = self.paprika.clone();
                    std::thread::spawn(move || {
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(_paprika.lock().unwrap().update_recipe(&mut edited_recipe));
                    });
                }
            }
            Message::RecipeClicked(recipe_uid) => {
                println!("Recipe clicked: {:?}", recipe_uid);
                let mut recipes = self.recipes.lock().unwrap();

                let found_recipe = recipes.iter_mut().find(|_recipe| _recipe.uid == recipe_uid);
                let serialized = serde_json::to_string_pretty(found_recipe.unwrap()).unwrap();

                let edited = edit::edit(serialized).unwrap();

                let mut edited_recipe: paprika_api::api::Recipe =
                    serde_json::from_str(&edited).unwrap();

                {
                    let _paprika = self.paprika.clone();
                    std::thread::spawn(move || {
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(_paprika.lock().unwrap().update_recipe(&mut edited_recipe));
                    });
                }
            }
            Message::RecipeFetched(recipe) => match recipe {
                Some(recipe) => {
                    let mut recipes = self.recipes.lock().unwrap();
                    let found_recipe = recipes.iter_mut().find(|_recipe| _recipe.uid == recipe.uid);
                    match found_recipe {
                        Some(_recipe) => *_recipe = recipe,
                        None => recipes.push(recipe),
                    }
                }
                None => {}
            },
            Message::LoginClicked => {
                let mut account = account::Account::new("".into(), "".into());

                let serialized = serde_json::to_string_pretty(&account).unwrap();

                let edited = edit::edit(serialized).unwrap();

                account = serde_json::from_str(&edited).unwrap();
                
                {
                    let mut _paprika = self.paprika.lock().unwrap();
                    _paprika.account = account;
                    _paprika.token = "".into();
                    _paprika.recipe_entries.clear();
                    _paprika.last_fetched = 0;
                }
                {
                    let mut _recipes = self.recipes.lock().unwrap();
                    _recipes.clear();
                }
            }
            Message::AccountChanged(_, _) => todo!(),
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
            nav_pane: NavPane::new(),
            recipes: recipes.clone(),
            recipe_buttons: Vec::new(),
        }
    }
    fn view(
        &mut self,
        #[allow(unused)] pane: pane_grid::Pane,
        #[allow(unused)] total_panes: usize,
        is_nav_bar: bool,
    ) -> Element<Message> {
        let Content { scroll, .. } = self;

        match is_nav_bar {
            true => self.nav_pane.view(),
            false => {
                let controls = Column::new().spacing(5).max_width(150);

                let mut content = Scrollable::new(scroll)
                    .width(Length::Fill)
                    .spacing(10)
                    .align_items(Align::Center)
                    .push(controls);

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
                                    recipe.uid.clone(),
                                    url.clone(),
                                )
                            }
                            None => {
                                recipe_button = recipe_button::RecipeButton::new(
                                    recipe.name.clone(),
                                    recipe.uid.clone(),
                                    "".into(),
                                )
                            }
                        }
                        // store the button in Content's owned Vec to allow it to live long enough
                        self.recipe_buttons.push(recipe_button);
                    }
                }
                for recipe_button in &mut self.recipe_buttons {
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
