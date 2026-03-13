use std::path::PathBuf;
use std::time::Duration;

use iced::widget::{button, column, container, pane_grid, row, text};
use iced::{Element, Font, Length, Size, Theme};

mod config;
mod terminal;
mod terminal_box;
mod theme;
mod workspace;

use crate::config::Config;
use crate::terminal::TerminalEvent;
use crate::terminal_box::TerminalBox;
use crate::theme::ColorScheme;
use crate::workspace::{PaneContent, Workspace};

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
    workspaces: Vec<Workspace>,
    active_workspace: usize,
    config: Config,
    color_scheme: ColorScheme,
    title: String,
}

#[derive(Debug, Clone)]
enum Message {
    TerminalInput(pane_grid::Pane, Vec<u8>),
    TerminalResize(pane_grid::Pane, u16, u16),
    Tick,

    PaneClicked(pane_grid::Pane),
    PaneResized(pane_grid::ResizeEvent),
    PaneDragged(pane_grid::DragEvent),
    PaneSplit(pane_grid::Axis),
    PaneClose,
    PaneFocusUp,
    PaneFocusDown,
    PaneFocusLeft,
    PaneFocusRight,
    PaneToggleMaximized,

    TabNew,
    TabClose(pane_grid::Pane, usize),
    TabActivate(pane_grid::Pane, usize),
    TabNext,
    TabPrev,
}

impl App {
    fn new() -> Self {
        let config = Config::load();
        let color_scheme = ColorScheme::gmux_dark();

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let workspace = Workspace::new("Default", &home, &config);

        let workspaces = match workspace {
            Some(ws) => vec![ws],
            None => Vec::new(),
        };

        Self {
            workspaces,
            active_workspace: 0,
            config,
            color_scheme,
            title: String::from("gmux"),
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::TerminalInput(pane, bytes) => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if let Some(content) = ws.panes.get_mut(pane) {
                        if let Some(tab) = content.active_tab_mut() {
                            tab.terminal.input(&bytes);
                        }
                    }
                }
            }
            Message::TerminalResize(pane, cols, rows) => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if let Some(content) = ws.panes.get_mut(pane) {
                        if let Some(tab) = content.active_tab_mut() {
                            tab.terminal.resize(cols, rows);
                        }
                    }
                }
            }
            Message::Tick => {
                for ws in &mut self.workspaces {
                    let pane_keys: Vec<pane_grid::Pane> =
                        ws.panes.iter().map(|(p, _)| *p).collect();
                    for pane_key in pane_keys {
                        if let Some(content) = ws.panes.get_mut(pane_key) {
                            let mut tab_exits = Vec::new();
                            for (tab_idx, tab) in
                                content.tabs.iter_mut().enumerate()
                            {
                                while let Some(event) =
                                    tab.terminal.try_recv_event()
                                {
                                    match event {
                                        TerminalEvent::TitleChanged(
                                            new_title,
                                        ) => {
                                            tab.name = new_title.clone();
                                            if ws.focus == pane_key
                                                && content.active_tab
                                                    == tab_idx
                                            {
                                                self.title = new_title;
                                            }
                                        }
                                        TerminalEvent::ChildExit(_) => {
                                            tab_exits.push(tab_idx);
                                        }
                                        TerminalEvent::Wakeup
                                        | TerminalEvent::Bell
                                        | TerminalEvent::ClipboardStore(_) => {
                                        }
                                    }
                                }
                                tab.terminal.needs_update = true;
                            }

                            for idx in tab_exits.into_iter().rev() {
                                content.tabs.remove(idx);
                                if content.active_tab >= content.tabs.len()
                                    && !content.tabs.is_empty()
                                {
                                    content.active_tab =
                                        content.tabs.len() - 1;
                                }
                            }
                        }
                    }

                    let empty_panes: Vec<pane_grid::Pane> = ws
                        .panes
                        .iter()
                        .filter(|(_, c)| c.tabs.is_empty())
                        .map(|(p, _)| *p)
                        .collect();
                    for pane in empty_panes {
                        if ws.panes.len() > 1 {
                            if let Some(sibling) = ws.panes.close(pane) {
                                if ws.focus == pane {
                                    ws.focus = sibling.1;
                                }
                            }
                        }
                    }
                }
            }
            Message::PaneClicked(pane) => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    ws.focus = pane;
                }
            }
            Message::PaneResized(event) => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    ws.panes.resize(event.split, event.ratio);
                }
            }
            Message::PaneDragged(event) => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    match event {
                        pane_grid::DragEvent::Dropped { pane, target } => {
                            ws.panes.drop(pane, target);
                        }
                        pane_grid::DragEvent::Picked { .. }
                        | pane_grid::DragEvent::Canceled { .. } => {}
                    }
                }
            }
            Message::PaneSplit(axis) => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if let Some(tab) =
                        Workspace::create_tab(&ws.cwd, &self.config)
                    {
                        let content = PaneContent {
                            tabs: vec![tab],
                            active_tab: 0,
                        };
                        if let Some((new_pane, _)) =
                            ws.panes.split(axis, ws.focus, content)
                        {
                            ws.focus = new_pane;
                        }
                    }
                }
            }
            Message::PaneClose => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if ws.panes.len() > 1 {
                        if let Some((_content, sibling)) =
                            ws.panes.close(ws.focus)
                        {
                            ws.focus = sibling;
                        }
                    }
                }
            }
            Message::PaneFocusUp => {
                self.move_focus(pane_grid::Direction::Up);
            }
            Message::PaneFocusDown => {
                self.move_focus(pane_grid::Direction::Down);
            }
            Message::PaneFocusLeft => {
                self.move_focus(pane_grid::Direction::Left);
            }
            Message::PaneFocusRight => {
                self.move_focus(pane_grid::Direction::Right);
            }
            Message::PaneToggleMaximized => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if ws.panes.maximized().is_some() {
                        ws.panes.restore();
                    } else {
                        ws.panes.maximize(ws.focus);
                    }
                }
            }
            Message::TabNew => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    let cwd = ws.cwd.clone();
                    if let Some(tab) =
                        Workspace::create_tab(&cwd, &self.config)
                    {
                        if let Some(content) = ws.panes.get_mut(ws.focus) {
                            content.tabs.push(tab);
                            content.active_tab = content.tabs.len() - 1;
                        }
                    }
                }
            }
            Message::TabClose(pane, idx) => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if let Some(content) = ws.panes.get_mut(pane) {
                        if idx < content.tabs.len() {
                            content.tabs.remove(idx);
                            if content.tabs.is_empty() {
                                if ws.panes.len() > 1 {
                                    if let Some((_content, sibling)) =
                                        ws.panes.close(pane)
                                    {
                                        if ws.focus == pane {
                                            ws.focus = sibling;
                                        }
                                    }
                                }
                            } else if content.active_tab >= content.tabs.len() {
                                content.active_tab = content.tabs.len() - 1;
                            }
                        }
                    }
                }
            }
            Message::TabActivate(pane, idx) => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if let Some(content) = ws.panes.get_mut(pane) {
                        if idx < content.tabs.len() {
                            content.active_tab = idx;
                        }
                    }
                }
            }
            Message::TabNext => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if let Some(content) = ws.panes.get_mut(ws.focus) {
                        if !content.tabs.is_empty() {
                            content.active_tab =
                                (content.active_tab + 1) % content.tabs.len();
                        }
                    }
                }
            }
            Message::TabPrev => {
                if let Some(ws) = self.workspaces.get_mut(self.active_workspace)
                {
                    if let Some(content) = ws.panes.get_mut(ws.focus) {
                        if !content.tabs.is_empty() {
                            content.active_tab = if content.active_tab == 0 {
                                content.tabs.len() - 1
                            } else {
                                content.active_tab - 1
                            };
                        }
                    }
                }
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.workspaces.is_empty() {
            return container(text("No workspace"))
                .center(Length::Fill)
                .into();
        }

        let workspace = &self.workspaces[self.active_workspace];
        let font_size = self.config.appearance.font_size as f32;
        let color_scheme = &self.color_scheme;

        let grid = pane_grid::PaneGrid::new(
            &workspace.panes,
            |pane, content, _is_maximized| {
                let is_focused = workspace.focus == pane;

                let body: Element<'_, Message> =
                    if let Some(tab) = content.active_tab() {
                        TerminalBox::new(
                            &tab.terminal,
                            color_scheme,
                            font_size,
                            move |bytes| Message::TerminalInput(pane, bytes),
                        )
                        .on_resize(move |cols, rows| {
                            Message::TerminalResize(pane, cols, rows)
                        })
                        .into()
                    } else {
                        text("No active tab").into()
                    };

                let tab_bar: Element<'_, Message> = if content.tabs.len() > 1 {
                    let tabs = content.tabs.iter().enumerate().map(|(i, tab)| {
                        let is_active = i == content.active_tab;
                        let label = text(&tab.name).size(font_size * 0.8);
                        let btn = button(label)
                            .on_press(Message::TabActivate(pane, i))
                            .padding([2, 8]);
                        let btn: Element<'_, Message> = if is_active {
                            btn.style(button::primary).into()
                        } else {
                            btn.style(button::secondary).into()
                        };
                        btn
                    });
                    let new_tab_btn: Element<'_, Message> =
                        button(text("+").size(font_size * 0.8))
                            .on_press(Message::TabNew)
                            .padding([2, 6])
                            .style(button::secondary)
                            .into();
                    let mut tab_items: Vec<Element<'_, Message>> =
                        tabs.collect();
                    tab_items.push(new_tab_btn);
                    container(row(tab_items).spacing(2))
                        .padding([2, 4])
                        .into()
                } else {
                    container(row![]).height(0).into()
                };

                let content_view = column![tab_bar, body];

                let tab_name = content
                    .active_tab()
                    .map(|t| t.name.as_str())
                    .unwrap_or("Shell");

                let title_label = text(tab_name).size(font_size * 0.85);

                let close_btn = button(text("\u{00D7}").size(font_size * 0.85))
                    .on_press(Message::TabClose(
                        pane,
                        content.active_tab,
                    ))
                    .padding([0, 6])
                    .style(button::text);

                let title_bar_style = if is_focused {
                    container::dark
                } else {
                    container::transparent
                };

                let title_bar =
                    pane_grid::TitleBar::new(title_label)
                        .controls(pane_grid::Controls::new(close_btn))
                        .padding([2, 4])
                        .style(title_bar_style);

                pane_grid::Content::new(content_view).title_bar(title_bar)
            },
        )
        .on_click(Message::PaneClicked)
        .on_resize(4, Message::PaneResized)
        .on_drag(Message::PaneDragged)
        .spacing(2);

        grid.into()
    }

    fn move_focus(&mut self, direction: pane_grid::Direction) {
        if let Some(ws) = self.workspaces.get_mut(self.active_workspace) {
            if let Some(adj) = ws.panes.adjacent(ws.focus, direction) {
                ws.focus = adj;
            }
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
