mod paprika;

use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::{thread, time};

use iced::{
    button, executor,
    pane_grid::{self, Axis},
    scrollable, Align, Application, Button, Clipboard, Column, Command, Container, Element,
    HorizontalAlignment, Length, PaneGrid, Scrollable, Subscription, Text,
};

use iced_futures::futures;

pub struct HomePage {
    panes: pane_grid::State<Pane>,
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
    pub is_nav_pane: bool,
}

struct Content {
    scroll: scrollable::State,
    split_horizontally: button::State,
    split_vertically: button::State,
    close: button::State,
    nav_pane: NavPane,
}

#[derive(Debug, Clone, Copy)]
pub enum Message<T> {
    Split(pane_grid::Axis, pane_grid::Pane),
    Close(pane_grid::Pane),
    RefreshClicked,
    NewRecipeClicked,
    RecipeFetched(T),
}

struct RecipeFetcher<T> {
    id: T,
    paprika: Arc<Mutex<paprika::Paprika>>,
}

impl<H, I, T> iced_native::subscription::Recipe<H, I> for RecipeFetcher<T>
where
    T: 'static + Hash + Copy + Send,
    H: Hasher,
{
    type Output = T;

    fn hash(&self, state: &mut H) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.id.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<I>,
    ) -> futures::stream::BoxStream<Self::Output> {
        let _id = self.id;

        Box::pin(futures::stream::unfold(
            self.paprika.clone(),
            move |paprika| async move {
                let uid;
                {
                    {
                        let mut _paprika = paprika.lock().unwrap();

                        if _paprika.recipe_entries.len() == 0 {
                            tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap()
                                .block_on(_paprika.fetch_recipe_list());
                        }
                    }

                    // the render thread uses the same mutex, so this is to
                    // prevent that thread from being blocked too long
                    // TODO: After fetching, copy to a vec on HomePage to prevent
                    // need to block during the fetch
                    thread::sleep(time::Duration::from_millis(15));
                    {
                        let mut _paprika = paprika.lock().unwrap();

                        if _paprika.recipe_entries.len() != 0 {
                            uid = _paprika.recipe_entries[_paprika.last_fetched]
                                .uid
                                .to_owned();
                            _paprika.last_fetched += 1;

                            tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap()
                                .block_on(_paprika.fetch_recipe_by_id(&uid));
                        }
                    }
                }
                // the render thread uses the same mutex, so this is to
                // prevent that thread from being blocked too long
                thread::sleep(time::Duration::from_millis(15));
                Some((_id, paprika))
            },
        ))
    }
}
impl Application for HomePage {
    type Message = Message<i32>;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message<i32>>) {
        let paprika = paprika::Paprika::new();
        let mutex = std::sync::Mutex::new(paprika);
        let arc = std::sync::Arc::new(mutex);

        // create the State<Pane>, then split it
        let (mut panes, pane) = pane_grid::State::new(Pane::new(true));
        let (_split_panes, _split) = panes
            .split(Axis::Vertical, &pane, Pane::new(false))
            .expect("Failed to split panes");

        panes.resize(&_split, 0.15);

        (
            HomePage {
                panes: panes,
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
                let _result = self.panes.split(axis, &pane, Pane::new(false));
            }
            Message::Close(_) => todo!(),
            Message::RefreshClicked => {
                let mut mutex = self.paprika.lock().unwrap();
                {
                    mutex.recipe_entries.clear();
                    mutex.recipes.clear();
                    mutex.last_fetched = 0;
                }
            }
            Message::NewRecipeClicked => {
                println!("New recipe!")
            }
            Message::RecipeFetched(_id) => {
                let recipes;
                let recipe_entries;
                {
                    let paprika = self.paprika.lock().unwrap();
                    recipes = paprika.recipes.len();
                    recipe_entries = paprika.recipe_entries.len();
                }
                println!("Fetched! {}/{} total. ID: {}", recipes, recipe_entries, _id);
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
        });
        test.map(|id| Message::RecipeFetched(id))
    }
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
    fn new(is_nav_pane: bool) -> Self {
        Self {
            content: Content::new(),
            is_nav_pane: is_nav_pane,
        }
    }
}

impl Content {
    fn new() -> Self {
        Content {
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
    #[allow(unused)]
    use iced::{button, container, Background, Color};

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
