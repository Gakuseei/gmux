use std::time::Duration;

use iced::{Element, Font, Size, Theme};

mod config;
mod terminal;
mod terminal_box;
mod theme;

use crate::config::Config;
use crate::terminal::{Terminal, TerminalEvent};
use crate::terminal_box::TerminalBox;
use crate::theme::ColorScheme;

fn main() -> iced::Result {
    let config = Config::load();
    let window_width = config.appearance.font_size as f32 * 100.0;
    let window_height = config.appearance.font_size as f32 * 40.0;

    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .default_font(Font::MONOSPACE)
        .window_size(Size::new(window_width, window_height))
        .subscription(App::subscription)
        .run()
}

struct App {
    terminal: Option<Terminal>,
    config: Config,
    color_scheme: ColorScheme,
    title: String,
}

#[derive(Debug, Clone)]
enum Message {
    TerminalInput(Vec<u8>),
    TerminalResize(u16, u16),
    Tick,
}

impl App {
    fn new() -> Self {
        let config = Config::load();
        let color_scheme = ColorScheme::gmux_dark();

        let shell = &config.terminal.default_shell;
        let home = dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("/"));
        let scrollback = config.terminal.scrollback_lines as usize;
        let font_size = config.appearance.font_size as f32;
        let cell_width = font_size * 0.6;
        let cell_height = font_size * 1.2;

        let terminal = Terminal::new(
            shell,
            &home,
            80,
            24,
            scrollback,
            cell_width,
            cell_height,
        )
        .ok();

        Self {
            terminal,
            config,
            color_scheme,
            title: String::from("gmux"),
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::TerminalInput(bytes) => {
                if let Some(ref terminal) = self.terminal {
                    terminal.input(&bytes);
                }
            }
            Message::TerminalResize(cols, rows) => {
                if let Some(ref mut terminal) = self.terminal {
                    terminal.resize(cols, rows);
                }
            }
            Message::Tick => {
                if let Some(ref mut terminal) = self.terminal {
                    while let Some(event) = terminal.try_recv_event() {
                        match event {
                            TerminalEvent::TitleChanged(new_title) => {
                                self.title = new_title;
                            }
                            TerminalEvent::ChildExit(_) => {
                                self.terminal = None;
                                return iced::Task::none();
                            }
                            TerminalEvent::Wakeup
                            | TerminalEvent::Bell
                            | TerminalEvent::ClipboardStore(_) => {}
                        }
                    }
                    terminal.needs_update = true;
                }
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if let Some(ref terminal) = self.terminal {
            let font_size = self.config.appearance.font_size as f32;
            TerminalBox::new(terminal, &self.color_scheme, font_size, Message::TerminalInput)
                .on_resize(Message::TerminalResize)
                .into()
        } else {
            iced::widget::text("No terminal").into()
        }
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Theme {
        let palette = iced::theme::Palette {
            background: self.color_scheme.background.to_iced(),
            text: self.color_scheme.foreground.to_iced(),
            primary: self.color_scheme.cursor.to_iced(),
            success: self.color_scheme.ansi[2].to_iced(),
            warning: self.color_scheme.ansi[3].to_iced(),
            danger: self.color_scheme.ansi[1].to_iced(),
        };
        Theme::custom("gmux-dark".to_string(), palette)
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
    }
}
