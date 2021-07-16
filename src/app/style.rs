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
