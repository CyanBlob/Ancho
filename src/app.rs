use iced::{
    button, executor, keyboard,
    pane_grid::{self, Axis},
    scrollable, Align, Application, Button, Clipboard, Color, Column, Command, Container, Element,
    HorizontalAlignment, Length, PaneGrid, Row, Scrollable, Settings, Subscription, Text,
};

pub struct HomePage {
    panes: pane_grid::State<Pane>,
    recipes: Vec<paprika_api::api::Recipe>,
}

struct NavPane {
    refresh: SimpleButton,
    new: SimpleButton,
}

struct SimpleButton {
    state: button::State,
    text: String,
    on_pressed: Message,
}

impl SimpleButton {
    fn to_button(&mut self) -> button::Button<Message> {
        Button::new(&mut self.state, Text::new(&self.text)).into()
    }
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
pub enum Message {
    Split(pane_grid::Axis, pane_grid::Pane),
    SplitFocused(pane_grid::Axis),
    FocusAdjacent(pane_grid::Direction),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
    TogglePin(pane_grid::Pane),
    Close(pane_grid::Pane),
    CloseFocused,
    Button1Clicked,
    Button2Clicked,
}

impl Application for HomePage {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
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
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
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
            Message::Button1Clicked => todo!(),
            Message::Button2Clicked => todo!(),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let pane_grid = PaneGrid::new(&mut self.panes, |id, pane| {
            pane_grid::Content::new(pane.content.view(id, 2, pane.is_nav_pane))
            .style(style::Pane { is_nav_pane: pane.is_nav_pane })
        });
        pane_grid.into()
    }

    fn title(&self) -> String {
        String::from("Ancho Recipe Manager")
    }
}

impl SimpleButton {
    fn new(text: String, on_pressed: Message) -> Self {
        Self {
            state: button::State::new(),
            text: text.into(),
            on_pressed: on_pressed,
        }
    }
}

impl NavPane {
    fn new() -> Self {
        let refresh_simple = SimpleButton::new("Refresh recipes".into(), Message::Button1Clicked);
        let new_simple = SimpleButton::new("New recipes".into(), Message::Button2Clicked);

        Self {
            refresh: refresh_simple,
            new: new_simple,
        }
    }

    fn view(&mut self) -> Element<Message> {
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
