use iced::{Element, Font, Size, Theme};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title("gmux")
        .theme(App::theme)
        .default_font(Font::DEFAULT)
        .window_size(Size::new(1400.0, 900.0))
        .run()
}

struct App {}

#[derive(Debug, Clone)]
enum Message {}

impl App {
    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {}
    }

    fn view(&self) -> Element<'_, Message> {
        iced::widget::text("gmux v2").into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
