use std::path::PathBuf;
use std::time::Duration;

use iced::widget::{button, column, container, pane_grid, row, scrollable, text, Space};
use iced::{Background, Border, Element, Font, Length, Size, Theme};

mod config;
mod terminal;
mod terminal_box;
mod theme;
mod workspace;

use crate::config::Config;
use crate::terminal::TerminalEvent;
use crate::terminal_box::TerminalBox;
use crate::theme::{ColorScheme, UiTheme};
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
    ui_theme: UiTheme,
    sidebar_visible: bool,
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

    SidebarToggle,
    WorkspaceActivate(usize),
    WorkspaceNew,
}

impl App {
    fn new() -> Self {
        let config = Config::load();
        let color_scheme = ColorScheme::gmux_dark();
        let ui_theme = UiTheme::default();

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
            ui_theme,
            sidebar_visible: true,
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
                        let pane_count = ws.panes.len();
                        if pane_count > 1 {
                            if let Some(sibling) = ws.panes.close(pane) {
                                if ws.focus == pane {
                                    ws.focus = sibling.1;
                                }
                            }
                        } else if let Some(tab) = Workspace::create_tab(
                            &ws.cwd,
                            &self.config,
                        ) {
                            if let Some(content) = ws.panes.get_mut(pane) {
                                content.tabs.push(tab);
                                content.active_tab = 0;
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
                    let pane_count = ws.panes.len();
                    let should_remove_tab = ws
                        .panes
                        .get(pane)
                        .map(|c| idx < c.tabs.len())
                        .unwrap_or(false);

                    if should_remove_tab {
                        let became_empty = {
                            let content = ws.panes.get_mut(pane).unwrap();
                            content.tabs.remove(idx);
                            if content.tabs.is_empty() {
                                true
                            } else {
                                if content.active_tab >= content.tabs.len() {
                                    content.active_tab =
                                        content.tabs.len() - 1;
                                }
                                false
                            }
                        };

                        if became_empty {
                            if pane_count > 1 {
                                if let Some((_content, sibling)) =
                                    ws.panes.close(pane)
                                {
                                    if ws.focus == pane {
                                        ws.focus = sibling;
                                    }
                                }
                            } else if let Some(tab) =
                                Workspace::create_tab(
                                    &ws.cwd,
                                    &self.config,
                                )
                            {
                                if let Some(content) =
                                    ws.panes.get_mut(pane)
                                {
                                    content.tabs.push(tab);
                                    content.active_tab = 0;
                                }
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
            Message::SidebarToggle => {
                self.sidebar_visible = !self.sidebar_visible;
            }
            Message::WorkspaceActivate(idx) => {
                if idx < self.workspaces.len() {
                    self.active_workspace = idx;
                    if let Some(ws) = self.workspaces.get(self.active_workspace)
                    {
                        if let Some(content) = ws.panes.get(ws.focus) {
                            if let Some(tab) = content.active_tab() {
                                self.title = tab.name.clone();
                            }
                        }
                    }
                }
            }
            Message::WorkspaceNew => {
                let home =
                    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
                let name =
                    format!("Workspace {}", self.workspaces.len() + 1);
                if let Some(ws) = Workspace::new(&name, &home, &self.config) {
                    self.workspaces.push(ws);
                    self.active_workspace = self.workspaces.len() - 1;
                }
            }
        }
        iced::Task::none()
    }

    fn ghost_button_style(
        text_color: iced::Color,
        hover_bg: iced::Color,
    ) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(hover_bg))
                }
                _ => None,
            };
            button::Style {
                background: bg,
                text_color,
                border: Border::default(),
                ..button::Style::default()
            }
        }
    }

    fn sidebar_view(&self) -> Element<'_, Message> {
        let ui = &self.ui_theme;
        let font_size = self.config.appearance.font_size as f32;

        let sidebar_bg = ui.bg_sidebar.to_iced();
        let text_primary = ui.text_primary.to_iced();
        let text_secondary = ui.text_secondary.to_iced();
        let accent = ui.accent.to_iced();
        let border_color = ui.border.to_iced();
        let hover_color = ui.hover_overlay.to_iced_alpha(ui.hover_overlay_alpha);
        let active_bg = ui.accent.to_iced_alpha(ui.active_highlight_alpha);

        let new_ws_btn = button(
            text("+ New Workspace")
                .size(font_size * 0.85)
                .color(text_primary),
        )
        .on_press(Message::WorkspaceNew)
        .padding([8, 12])
        .width(Length::Fill)
        .style(Self::ghost_button_style(text_primary, accent));

        let header = container(
            text("WORKSPACES")
                .size(font_size * 0.7)
                .color(text_secondary),
        )
        .padding([8, 12]);

        let workspace_items: Vec<Element<'_, Message>> = self
            .workspaces
            .iter()
            .enumerate()
            .map(|(idx, ws)| {
                let is_active = idx == self.active_workspace;
                let name_color = if is_active { accent } else { text_primary };
                let bg_color = if is_active {
                    active_bg
                } else {
                    iced::Color::TRANSPARENT
                };

                let indicator: Element<'_, Message> = if is_active {
                    container(Space::new().width(3).height(Length::Fill))
                        .style(move |_theme: &Theme| container::Style {
                            background: Some(Background::Color(accent)),
                            ..Default::default()
                        })
                        .height(Length::Fill)
                        .into()
                } else {
                    Space::new().width(3).height(0).into()
                };

                let label =
                    text(&ws.name).size(font_size * 0.85).color(name_color);

                let ws_btn = button(
                    row![indicator, label]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                )
                .on_press(Message::WorkspaceActivate(idx))
                .padding([6, 12])
                .width(Length::Fill)
                .style(move |_theme: &Theme, status| {
                    let hover_bg = match status {
                        button::Status::Hovered => {
                            Some(Background::Color(hover_color))
                        }
                        _ => Some(Background::Color(bg_color)),
                    };
                    button::Style {
                        background: hover_bg,
                        text_color: name_color,
                        border: Border::default(),
                        ..button::Style::default()
                    }
                });

                ws_btn.into()
            })
            .collect();

        let ws_list = scrollable(column(workspace_items).spacing(2))
            .height(Length::Fill);

        let separator = container(
            Space::new().width(Length::Fill).height(1),
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(border_color)),
            ..Default::default()
        });

        let minimize_btn = button(
            text("\u{2039}")
                .size(font_size)
                .color(text_secondary),
        )
        .on_press(Message::SidebarToggle)
        .padding([4, 8])
        .style(Self::ghost_button_style(text_secondary, hover_color));

        let bottom_row = container(
            row![Space::new().width(Length::Fill), minimize_btn]
                .align_y(iced::Alignment::Center),
        )
        .padding([4, 8]);

        let sidebar_content = column![
            new_ws_btn,
            header,
            ws_list,
            separator,
            bottom_row,
        ];

        let sidebar_width = self.config.appearance.font_size as f32 * 18.0;

        container(sidebar_content)
            .width(sidebar_width)
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(sidebar_bg)),
                border: Border {
                    width: 1.0,
                    color: border_color,
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn pane_grid_view(&self) -> Element<'_, Message> {
        let Some(workspace) = self.workspaces.get(self.active_workspace) else {
            return container(text("No workspace"))
                .center(Length::Fill)
                .into();
        };
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

                let tab_bar: Element<'_, Message> = {
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

    fn view(&self) -> Element<'_, Message> {
        if self.workspaces.is_empty() {
            return container(text("No workspace"))
                .center(Length::Fill)
                .into();
        }

        let grid = self.pane_grid_view();

        if self.sidebar_visible {
            let sidebar = self.sidebar_view();
            row![sidebar, grid].into()
        } else {
            let ui = &self.ui_theme;
            let text_secondary = ui.text_secondary.to_iced();
            let hover_color =
                ui.hover_overlay.to_iced_alpha(ui.hover_overlay_alpha);
            let font_size = self.config.appearance.font_size as f32;

            let expand_btn = button(
                text("\u{203A}")
                    .size(font_size)
                    .color(text_secondary),
            )
            .on_press(Message::SidebarToggle)
            .padding([4, 4])
            .style(Self::ghost_button_style(text_secondary, hover_color));

            let expand_col = container(
                column![Space::new().height(Length::Fill), expand_btn],
            )
            .height(Length::Fill);

            row![expand_col, grid].into()
        }
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
