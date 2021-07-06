mod paprika;

use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::thread;

use iced::{
    button, executor, keyboard,
    pane_grid::{self, Axis},
    scrollable, Align, Application, Button, Clipboard, Color, Column, Command, Container, Element,
    HorizontalAlignment, Length, PaneGrid, Row, Scrollable, Settings, Subscription, Text,
};

use iced_futures::futures;
use iced_native::{event, subscription, Event};

pub struct HomePage {
    panes: pane_grid::State<Pane>,
    recipes: Vec<paprika_api::api::Recipe>,
    paprika: Arc<Mutex<paprika::Paprika>>,
}

struct NavPane {
    refresh: SimpleButton,
    new: SimpleButton,
}

struct SimpleButton {
    state: button::State,
    text: String,
    on_pressed: Message<i32>,
}

struct Pane {
    pub content: Content,
    pub is_nav_pane: bool, //pub controls: Controls,
}

struct Content {
    id: usize,
    scroll: scrollable::State,
    split_horizontally: button::State,
    split_vertically: button::State,
    close: button::State,
    nav_pane: NavPane,
}

struct Controls {
    close: button::State,
}

#[derive(Debug, Clone, Copy)]
pub enum Message<T> {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Close(pane_grid::Pane),
    CloseFocused,
    RefreshClicked,
    NewRecipeClicked,
    RecipeFetched(T),
}

struct RecipeFetcher<T> {
    id: T,
    counter: u32,
    paprika: Arc<Mutex<paprika::Paprika>>,
}

impl<H, I, T> iced_native::subscription::Recipe<H, I> for RecipeFetcher<T>
where
    T: 'static + Hash + Copy + Send,
    H: Hasher,
{
    //type Output = Message<T>;
    type Output = T;

    fn hash(&self, state: &mut H) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.id.hash(state);
    }

    fn stream(
        self: Box<Self>,
        input: futures::stream::BoxStream<I>,
    ) -> futures::stream::BoxStream<Self::Output> {
        let _id = self.id;

        //let paprika_locked = self.paprika.clone().lock().unwrap();
        //let paprika = self.paprika;
        //let cloned = self.paprika.clone();
        //let uid = cloned.lock().unwrap().recipe_entries[self.counter]
        //.uid
        //.to_owned();
        //let paprika = self.paprika.clone();

        Box::pin(futures::stream::unfold(
            self.paprika.clone(),
            move |paprika| async move {
                //paprika.lock().unwrap().list_recipes().await;
                let mut uid: String = "".into();
                {
                    let mut _paprika = paprika.lock().unwrap();

                    if _paprika.recipe_entries.len() == 0 {
                        tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(_paprika.fetch_recipe_list());
                    }

                    if _paprika.recipe_entries.len() != 0 {
                        uid = _paprika.recipe_entries[_paprika.last_fetched]
                            .uid
                            .to_owned();
                        _paprika.last_fetched += 1;
                        //_paprika.get_recipe_by_id(&uid).await;
                        let recipe = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(_paprika.fetch_recipe_by_id(&uid));
                    }
                }
                //self.paprika.lock().unwrap().list_recipes().await;
                //paprika_locked.get_recipe_by_id(&uid).await;
                //self.counter += 1;
                //println!("Sub?");
                //Some((Message::RecipeFetched(_id), counter))
                Some((_id, paprika))
            },
        ))
    }
}

/*impl RecipeFetcher<T> {
    // must be called from a secondary thread
    pub fn fetch(paprika: Arc<Mutex<paprika::Paprika>>) {
        let recipe_entries = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(paprika.lock().unwrap().fetch_recipe_list());

        for entry in recipe_entries {
            // only fetch if we don't already have it stored
            if !paprika
                .lock()
                .unwrap()
                .recipes
                .iter()
                .any(|recipe| return entry.uid == recipe.uid)
            {
                let recipe = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(paprika.lock().unwrap().get_recipe_by_id(&entry.uid));
                paprika.lock().unwrap().recipes.push(recipe);
            }
        }
    }
}*/

impl Application for HomePage {
    type Message = Message<i32>;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message<i32>>) {
        let paprika = paprika::Paprika::new();
        let mutex = std::sync::Mutex::new(paprika);
        let arc = std::sync::Arc::new(mutex);

        {
            let arc = arc.clone();
            thread::spawn(move || {
                //RecipeFetcher::fetch(arc);
            });
        }

        // create the State<Pane>, then split it
        let (mut panes, pane) = pane_grid::State::new(Pane::new(100, true));
        let (_split_panes, _split) = panes
            .split(Axis::Vertical, &pane, Pane::new(2, false))
            .expect("Failed to split panes");

        panes.resize(&_split, 0.15);

        (
            HomePage {
                panes: panes,
                recipes: Vec::<paprika_api::api::Recipe>::new(),
                paprika: arc.clone(),
            },
            Command::none(),
        )
    }

    fn update(
        &mut self,
        message: Message<i32>,
        _clipboard: &mut Clipboard,
    ) -> Command<Message<i32>> {
        match message {
            Message::Split(axis, pane) => {
                let _result = self.panes.split(axis, &pane, Pane::new(2, false));

                //if let Some((pane, _)) = result {
                //self.focus = Some(pane);
                //}

                //self.panes_created += 1;
            }
            Message::SplitFocused(_) => todo!(),
            Message::FocusAdjacent(_) => todo!(),
            Message::Clicked(_) => todo!(),
            Message::Dragged(_) => todo!(),
            Message::Resized(_) => todo!(),
            Message::TogglePin(_) => todo!(),
            Message::Close(_) => todo!(),
            Message::CloseFocused => todo!(),
            Message::RefreshClicked => {
                self.paprika.lock().unwrap().recipes.clear();
                {
                    let paprika = self.paprika.clone();
                    thread::spawn(move || {
                        //RecipeFetcher::fetch(paprika);
                    });
                }
            }
            Message::NewRecipeClicked => {
                println!("New recipe!")
            }
            Message::RecipeFetched(_id) => {
                println!(
                    "Fetched! {} total. ID: {}",
                    self.paprika.lock().unwrap().recipes.len(),
                    _id
                );
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message<i32>> {
        let pane_grid = PaneGrid::new(&mut self.panes, |id, pane| {
            pane_grid::Content::new(pane.content.view(id, 2, pane.is_nav_pane)).style(style::Pane {
                is_nav_pane: pane.is_nav_pane,
            })
        });
        println!("Recipes: {}", self.paprika.lock().unwrap().recipes.len());
        pane_grid.into()
    }

    fn title(&self) -> String {
        String::from("Ancho Recipe Manager")
    }

    fn subscription(&self) -> Subscription<Message<i32>> {
        let paprika = self.paprika.clone();
        let test = iced::Subscription::from_recipe(RecipeFetcher {
            paprika: paprika,
            id: 0,
            counter: 0,
        });
        println!("Sub");
        //Message::RecipeFetched(test)
        test.map(|id| Message::RecipeFetched(id))
        //.map(Message::RecipeFetched)
    }
    /*fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| {
            if let event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    modifiers,
                    key_code,
                }) if modifiers.is_command_pressed() => Some(Message::RecipeFetched()),
                _ => None,
            }
        })
    }*/
}

impl SimpleButton {
    fn new(text: String, on_pressed: Message<i32>) -> Self {
        Self {
            state: button::State::new(),
            text: text.into(),
            on_pressed: on_pressed,
        }
    }

    fn to_button(&mut self) -> button::Button<Message<i32>> {
        Button::new(&mut self.state, Text::new(&self.text))
            .on_press(self.on_pressed)
            .into()
    }
}

impl NavPane {
    fn new() -> Self {
        let refresh_simple = SimpleButton::new("Refresh recipes".into(), Message::RefreshClicked);
        let new_simple = SimpleButton::new("New recipes".into(), Message::NewRecipeClicked);

        Self {
            refresh: refresh_simple,
            new: new_simple,
        }
    }

    fn view(&mut self) -> Element<Message<i32>> {
        let mut column = Column::new();

        column = column.push(self.refresh.to_button());
        column = column.push(self.new.to_button());

        column.into()
    }
}

impl Pane {
    fn new(id: usize, is_nav_pane: bool) -> Self {
        Self {
            content: Content::new(id),
            is_nav_pane: is_nav_pane,
        }
    }
}

impl Content {
    fn new(id: usize) -> Self {
        Content {
            id,
            scroll: scrollable::State::new(),
            split_horizontally: button::State::new(),
            split_vertically: button::State::new(),
            close: button::State::new(),
            nav_pane: NavPane::new(),
        }
    }
    fn view(
        &mut self,
        pane: pane_grid::Pane,
        total_panes: usize,
        is_nav_bar: bool,
    ) -> Element<Message<i32>> {
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

                let content = Scrollable::new(scroll)
                    .width(Length::Fill)
                    .spacing(10)
                    .align_items(Align::Center)
                    .push(controls);

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

mod style {
    //use crate::PANE_ID_COLOR_FOCUSED;
    use iced::{button, container, Background, Color, Vector};

    pub struct Pane {
        pub is_nav_pane: bool,
    }

    impl container::StyleSheet for Pane {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::new(0.1, 0.1, 0.1, 0.1))),
                border_width: 2.0,
                border_color: if self.is_nav_pane {
                    //Color::BLACK
                    Color::from_rgb(0.7, 0.7, 0.7)
                } else {
                    Color::from_rgb(0.7, 0.7, 0.7)
                },
                ..Default::default()
            }
        }
    }
}
