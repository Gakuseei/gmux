use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use copypasta::{ClipboardContext, ClipboardProvider};
use iced::keyboard::{Key, Modifiers};
use iced::widget::{
    button, column, container, pane_grid, pick_list, row, scrollable, text,
    text_input, Space,
};
use iced::{Background, Border, Element, Font, Length, Size, Theme};

mod config;
mod mouse_reporter;
mod notifications;
mod scrollback;
mod shortcuts;
mod terminal;
mod terminal_box;
mod theme;
mod workspace;
mod usage;
mod git;

use crate::config::{Config, CursorStyle};
use crate::notifications::NotificationDetector;
use crate::terminal::TerminalEvent;
use crate::terminal_box::TerminalBox;
use crate::theme::{ColorScheme, UiTheme};
use crate::workspace::{PaneContent, Workspace};

#[derive(Debug, Clone, PartialEq)]
enum AppView {
    Terminals,
    Insights,
}

#[derive(Debug, Clone, PartialEq)]
enum InsightsTab {
    Usage,
    Git,
    Info,
}

fn main() -> iced::Result {
    if cfg!(target_os = "linux") && std::env::var("WGPU_BACKEND").is_err() {
        std::env::set_var("WGPU_BACKEND", "gl");
    }

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
    active_view: AppView,
    settings_open: bool,
    title: String,
    key_binds: HashMap<shortcuts::KeyBind, shortcuts::Action>,
    clipboard: Option<ClipboardContext>,
    notification_detector: NotificationDetector,
    active_insights_tab: InsightsTab,
    usage_data: Option<usage::UsageData>,
    usage_period: String,
    git_branch: Option<String>,
    git_branches: Vec<git::BranchInfo>,
    git_files: Vec<git::FileStatus>,
    git_diff: Option<git::FileDiff>,
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
    ViewSwitch(AppView),
    SettingsToggle,
    SettingsFontSizeChange(i32),
    SettingsShellChange(String),
    SettingsScrollbackChange(i32),
    SettingsCursorStyleChange(CursorStyle),
    KeyEvent(Key, Modifiers),
    InsightsTabSwitch(InsightsTab),
    InsightsRefresh,
    UsagePeriodChange(String),
    GitSelectFile(String),
    GitBackFromDiff,
    GitSwitchBranch(String),
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
            active_view: AppView::Terminals,
            settings_open: false,
            title: String::from("gmux"),
            key_binds: shortcuts::default_keybindings(),
            clipboard: ClipboardContext::new().ok(),
            notification_detector: NotificationDetector::new(),
            active_insights_tab: InsightsTab::Usage,
            usage_data: None,
            usage_period: String::from("today"),
            git_branch: None,
            git_branches: Vec::new(),
            git_files: Vec::new(),
            git_diff: None,
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
                let mut pending_notifications: Vec<(String, String)> =
                    Vec::new();

                for (ws_idx, ws) in self.workspaces.iter_mut().enumerate() {
                    let is_active_ws = ws_idx == self.active_workspace;
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

                            let is_focused_pane =
                                is_active_ws && ws.focus == pane_key;
                            for (tab_idx, tab) in content.tabs.iter().enumerate()
                            {
                                let is_focused_tab = is_focused_pane
                                    && tab_idx == content.active_tab;
                                if !is_focused_tab {
                                    let line = tab.terminal.last_line();
                                    if !line.is_empty() {
                                        let result =
                                            self.notification_detector
                                                .detect(&line);
                                        if result.matched {
                                            pending_notifications.push((
                                                tab.name.clone(),
                                                result.pattern,
                                            ));
                                        }
                                    }
                                }
                            }

                            for idx in tab_exits.into_iter().rev() {
                                let _ = scrollback::save_scrollback(
                                    &content.tabs[idx].terminal.id,
                                    &content.tabs[idx].terminal.grid_content(),
                                );
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

                for (terminal_name, pattern_name) in pending_notifications {
                    let _ = notifications::send_desktop_notification(
                        &terminal_name,
                        &pattern_name,
                    );
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
                            let _ = scrollback::save_scrollback(
                                &content.tabs[idx].terminal.id,
                                &content.tabs[idx].terminal.grid_content(),
                            );
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
            Message::ViewSwitch(view) => {
                self.active_view = view.clone();
                if view == AppView::Insights {
                    return self.update(Message::InsightsRefresh);
                }
            }
            Message::SettingsToggle => {
                self.settings_open = !self.settings_open;
            }
            Message::SettingsFontSizeChange(delta) => {
                let new_size =
                    (self.config.appearance.font_size as i32 + delta).max(8).min(32) as u32;
                self.config.appearance.font_size = new_size;
                let _ = self.config.save();
            }
            Message::SettingsShellChange(shell) => {
                self.config.terminal.default_shell = shell;
                let _ = self.config.save();
            }
            Message::SettingsScrollbackChange(delta) => {
                let new_val =
                    (self.config.terminal.scrollback_lines as i32 + delta).max(100).min(100_000)
                        as u32;
                self.config.terminal.scrollback_lines = new_val;
                let _ = self.config.save();
            }
            Message::SettingsCursorStyleChange(style) => {
                self.config.terminal.cursor_style = style;
                let _ = self.config.save();
            }
            Message::KeyEvent(key, modifiers) => {
                if let Some(action) =
                    shortcuts::lookup(&self.key_binds, modifiers, &key)
                {
                    return self.handle_shortcut(action);
                }
            }
            Message::InsightsTabSwitch(tab) => {
                self.active_insights_tab = tab.clone();
                match tab {
                    InsightsTab::Git => {
                        self.refresh_git_data();
                        self.git_diff = None;
                    }
                    InsightsTab::Usage => {
                        if let Ok(data) = usage::get_usage_data(self.usage_period.clone()) {
                            self.usage_data = Some(data);
                        }
                    }
                    InsightsTab::Info => {}
                }
            }
            Message::InsightsRefresh => {
                match self.active_insights_tab {
                    InsightsTab::Usage => {
                        self.usage_data = usage::get_usage_data(self.usage_period.clone()).ok();
                    }
                    InsightsTab::Git => {
                        self.refresh_git_data();
                    }
                    InsightsTab::Info => {}
                }
            }
            Message::UsagePeriodChange(period) => {
                self.usage_period = period;
                self.usage_data = usage::get_usage_data(self.usage_period.clone()).ok();
            }
            Message::GitSelectFile(path) => {
                if let Some(ws) = self.workspaces.get(self.active_workspace) {
                    let cwd = ws.cwd.to_string_lossy().to_string();
                    self.git_diff = git::get_file_diff(cwd, path).ok();
                }
            }
            Message::GitBackFromDiff => {
                self.git_diff = None;
            }
            Message::GitSwitchBranch(branch) => {
                if let Some(ws) = self.workspaces.get(self.active_workspace) {
                    let cwd = ws.cwd.to_string_lossy().to_string();
                    if git::switch_branch(cwd.clone(), branch).is_ok() {
                        self.git_branch = git::get_current_branch(cwd.clone()).ok().flatten();
                        self.git_files = git::get_git_status(cwd).unwrap_or_default();
                        self.git_diff = None;
                    }
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

    fn top_bar_view(&self) -> Element<'_, Message> {
        let ui = &self.ui_theme;
        let font_size = self.config.appearance.font_size as f32;

        let bg = ui.bg_primary.to_iced();
        let text_primary = ui.text_primary.to_iced();
        let text_secondary = ui.text_secondary.to_iced();
        let accent = ui.accent.to_iced();
        let border_color = ui.border.to_iced();
        let hover_color = ui.hover_overlay.to_iced_alpha(ui.hover_overlay_alpha);
        let bar_height = font_size * 2.5;

        let title = text("gmux").size(font_size).color(text_primary);

        let terminals_active = self.active_view == AppView::Terminals;
        let insights_active = self.active_view == AppView::Insights;

        let terminals_btn = button(
            text("Terminals")
                .size(font_size * 0.85)
                .color(if terminals_active { accent } else { text_secondary }),
        )
        .on_press(Message::ViewSwitch(AppView::Terminals))
        .padding([4, 12])
        .style(if terminals_active {
            Self::ghost_button_style(accent, hover_color)
        } else {
            Self::ghost_button_style(text_secondary, hover_color)
        });

        let insights_btn = button(
            text("Insights")
                .size(font_size * 0.85)
                .color(if insights_active { accent } else { text_secondary }),
        )
        .on_press(Message::ViewSwitch(AppView::Insights))
        .padding([4, 12])
        .style(if insights_active {
            Self::ghost_button_style(accent, hover_color)
        } else {
            Self::ghost_button_style(text_secondary, hover_color)
        });

        let settings_btn = button(
            text("S")
                .size(font_size * 0.85)
                .color(text_secondary),
        )
        .on_press(Message::SettingsToggle)
        .padding([4, 8])
        .style(Self::ghost_button_style(text_secondary, hover_color));

        let left_spacer = Space::new().width(Length::Fill);
        let right_spacer = Space::new().width(Length::Fill);

        let bar_content = row![
            left_spacer,
            title,
            right_spacer,
            terminals_btn,
            insights_btn,
            settings_btn,
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center)
        .padding([0, 12]);

        container(bar_content)
            .width(Length::Fill)
            .height(bar_height)
            .center_y(bar_height)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(bg)),
                border: Border {
                    width: 1.0,
                    color: border_color,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
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

    fn settings_view(&self) -> Element<'_, Message> {
        let ui = &self.ui_theme;
        let font_size = self.config.appearance.font_size as f32;

        let surface_bg = ui.bg_surface.to_iced();
        let primary_bg = ui.bg_primary.to_iced();
        let text_primary = ui.text_primary.to_iced();
        let text_secondary = ui.text_secondary.to_iced();
        let accent = ui.accent.to_iced();
        let hover_color = ui.hover_overlay.to_iced_alpha(ui.hover_overlay_alpha);

        let close_btn = button(
            text("\u{00D7}").size(font_size * 1.2).color(text_secondary),
        )
        .on_press(Message::SettingsToggle)
        .padding([4, 8])
        .style(Self::ghost_button_style(text_secondary, hover_color));

        let header = row![
            text("Settings").size(font_size * 1.4).color(text_primary),
            Space::new().width(Length::Fill),
            close_btn,
        ]
        .align_y(iced::Alignment::Center)
        .padding([0, 16]);

        let appearance_header =
            text("Appearance").size(font_size * 1.1).color(accent);

        let font_size_label =
            text("Font Size").size(font_size * 0.9).color(text_secondary);
        let font_size_value = text(format!("{}", self.config.appearance.font_size))
            .size(font_size)
            .color(text_primary);
        let font_dec = button(text("\u{2212}").size(font_size).color(text_primary))
            .on_press(Message::SettingsFontSizeChange(-1))
            .padding([4, 10])
            .style(Self::ghost_button_style(text_primary, hover_color));
        let font_inc = button(text("+").size(font_size).color(text_primary))
            .on_press(Message::SettingsFontSizeChange(1))
            .padding([4, 10])
            .style(Self::ghost_button_style(text_primary, hover_color));
        let font_size_row = row![
            font_size_label,
            Space::new().width(Length::Fill),
            font_dec,
            font_size_value,
            font_inc,
        ]
        .align_y(iced::Alignment::Center)
        .spacing(8);

        let accent_label =
            text("Accent Color").size(font_size * 0.9).color(text_secondary);
        let accent_value = text(&self.config.appearance.accent_color)
            .size(font_size)
            .color(text_primary);
        let accent_row = row![
            accent_label,
            Space::new().width(Length::Fill),
            accent_value,
        ]
        .align_y(iced::Alignment::Center)
        .spacing(8);

        let terminal_header =
            text("Terminal").size(font_size * 1.1).color(accent);

        let shell_label =
            text("Default Shell").size(font_size * 0.9).color(text_secondary);
        let shell_input = text_input("", &self.config.terminal.default_shell)
            .on_input(Message::SettingsShellChange)
            .size(font_size)
            .width(Length::Fixed(font_size * 20.0))
            .style(move |_theme: &Theme, status| {
                let border_c = match status {
                    text_input::Status::Focused { .. } => accent,
                    _ => primary_bg,
                };
                text_input::Style {
                    background: Background::Color(primary_bg),
                    border: Border {
                        width: 1.0,
                        color: border_c,
                        radius: 4.0.into(),
                    },
                    icon: text_primary,
                    placeholder: text_secondary,
                    value: text_primary,
                    selection: accent,
                }
            });
        let shell_row = row![
            shell_label,
            Space::new().width(Length::Fill),
            shell_input,
        ]
        .align_y(iced::Alignment::Center)
        .spacing(8);

        let scrollback_label =
            text("Scrollback Lines")
                .size(font_size * 0.9)
                .color(text_secondary);
        let scrollback_value =
            text(format!("{}", self.config.terminal.scrollback_lines))
                .size(font_size)
                .color(text_primary);
        let scrollback_dec =
            button(text("\u{2212}").size(font_size).color(text_primary))
                .on_press(Message::SettingsScrollbackChange(-1000))
                .padding([4, 10])
                .style(Self::ghost_button_style(text_primary, hover_color));
        let scrollback_inc =
            button(text("+").size(font_size).color(text_primary))
                .on_press(Message::SettingsScrollbackChange(1000))
                .padding([4, 10])
                .style(Self::ghost_button_style(text_primary, hover_color));
        let scrollback_row = row![
            scrollback_label,
            Space::new().width(Length::Fill),
            scrollback_dec,
            scrollback_value,
            scrollback_inc,
        ]
        .align_y(iced::Alignment::Center)
        .spacing(8);

        let cursor_label =
            text("Cursor Style").size(font_size * 0.9).color(text_secondary);
        let cursor_pick = pick_list(
            &CursorStyle::ALL[..],
            Some(&self.config.terminal.cursor_style),
            Message::SettingsCursorStyleChange,
        )
        .text_size(font_size);
        let cursor_row = row![
            cursor_label,
            Space::new().width(Length::Fill),
            cursor_pick,
        ]
        .align_y(iced::Alignment::Center)
        .spacing(8);

        let section_spacing = font_size * 1.5;

        let content = column![
            header,
            appearance_header,
            font_size_row,
            accent_row,
            Space::new().height(section_spacing),
            terminal_header,
            shell_row,
            scrollback_row,
            cursor_row,
        ]
        .spacing(12)
        .padding(24)
        .max_width(600);

        let scrollable_content =
            scrollable(container(content).center_x(Length::Fill))
                .height(Length::Fill);

        container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(surface_bg)),
                ..Default::default()
            })
            .into()
    }

    fn format_tokens(n: u64) -> String {
        if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.0}K", n as f64 / 1_000.0)
        } else {
            format!("{}", n)
        }
    }

    fn format_cost(c: f64) -> String {
        if c < 0.01 && c > 0.0 {
            String::from("<$0.01")
        } else {
            format!("${:.2}", c)
        }
    }

    fn token_cost(tokens: u64, rate_per_million: f64) -> f64 {
        tokens as f64 * rate_per_million / 1_000_000.0
    }

    fn insights_view(&self) -> Element<'_, Message> {
        let ui = &self.ui_theme;
        let font_size = self.config.appearance.font_size as f32;
        let accent = ui.accent.to_iced();
        let text_secondary = ui.text_secondary.to_iced();
        let hover_color = ui.hover_overlay.to_iced_alpha(ui.hover_overlay_alpha);
        let border_color = ui.border.to_iced();
        let sidebar_bg = ui.bg_sidebar.to_iced();

        let nav_tabs = [
            (InsightsTab::Usage, "Usage"),
            (InsightsTab::Git, "Git"),
            (InsightsTab::Info, "Info"),
        ];

        let nav_items: Vec<Element<'_, Message>> = nav_tabs
            .iter()
            .map(|(tab, label)| {
                let is_active = self.active_insights_tab == *tab;
                let label_color = if is_active { accent } else { text_secondary };
                let bg = if is_active {
                    ui.accent.to_iced_alpha(ui.active_highlight_alpha)
                } else {
                    iced::Color::TRANSPARENT
                };
                let active_bg = bg;
                button(
                    text(*label)
                        .size(font_size * 0.85)
                        .color(label_color),
                )
                .on_press(Message::InsightsTabSwitch(tab.clone()))
                .padding([8, 16])
                .width(Length::Fill)
                .style(move |_theme: &Theme, status| {
                    let bg_color = match status {
                        button::Status::Hovered => hover_color,
                        _ => active_bg,
                    };
                    button::Style {
                        background: Some(Background::Color(bg_color)),
                        text_color: label_color,
                        border: Border::default(),
                        ..button::Style::default()
                    }
                })
                .into()
            })
            .collect();

        let nav_header = container(
            text("INSIGHTS")
                .size(font_size * 0.7)
                .color(text_secondary),
        )
        .padding([12, 16]);

        let nav_panel = container(
            column![nav_header]
                .push(column(nav_items).spacing(2)),
        )
        .width(Length::Fixed(font_size * 13.0))
        .height(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(sidebar_bg)),
            border: Border {
                width: 1.0,
                color: border_color,
                ..Default::default()
            },
            ..Default::default()
        });

        let content_panel: Element<'_, Message> = match self.active_insights_tab {
            InsightsTab::Usage => self.insights_usage_view(),
            InsightsTab::Git => self.insights_git_view(),
            InsightsTab::Info => self.insights_info_view(),
        };

        row![nav_panel, content_panel].into()
    }

    fn insights_usage_view(&self) -> Element<'_, Message> {
        let ui = &self.ui_theme;
        let font_size = self.config.appearance.font_size as f32;
        let text_primary = ui.text_primary.to_iced();
        let text_secondary = ui.text_secondary.to_iced();
        let accent = ui.accent.to_iced();
        let hover_color = ui.hover_overlay.to_iced_alpha(ui.hover_overlay_alpha);
        let surface_bg = ui.bg_surface.to_iced();
        let border_color = ui.border.to_iced();
        let bg_primary = ui.bg_primary.to_iced();

        let periods = [
            ("today", "Today"),
            ("weekly", "Weekly"),
            ("monthly", "Monthly"),
        ];

        let period_buttons: Vec<Element<'_, Message>> = periods
            .iter()
            .map(|(key, label)| {
                let is_active = self.usage_period == *key;
                let label_color = if is_active { accent } else { text_secondary };
                let active_bg = if is_active {
                    ui.accent.to_iced_alpha(ui.active_highlight_alpha)
                } else {
                    iced::Color::TRANSPARENT
                };
                button(
                    text(*label)
                        .size(font_size * 0.8)
                        .color(label_color),
                )
                .on_press(Message::UsagePeriodChange(key.to_string()))
                .padding([4, 12])
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered => hover_color,
                        _ => active_bg,
                    };
                    button::Style {
                        background: Some(Background::Color(bg)),
                        text_color: label_color,
                        border: Border::default(),
                        ..button::Style::default()
                    }
                })
                .into()
            })
            .collect();

        let refresh_btn = button(
            text("\u{21BB}")
                .size(font_size * 0.9)
                .color(text_secondary),
        )
        .on_press(Message::InsightsRefresh)
        .padding([4, 8])
        .style(Self::ghost_button_style(text_secondary, hover_color));

        let toolbar = row(period_buttons)
            .push(Space::new().width(Length::Fill))
            .push(refresh_btn)
            .spacing(4)
            .align_y(iced::Alignment::Center);

        let cost_rates = self
            .config
            .cost_rates
            .get("claude")
            .cloned()
            .unwrap_or_default();

        let (total_input, total_output, total_cache_read, total_cache_write, sessions) =
            match &self.usage_data {
                Some(data) => (
                    data.total_input,
                    data.total_output,
                    data.total_cache_read,
                    data.total_cache_write,
                    &data.sessions,
                ),
                None => (0, 0, 0, 0, &Vec::new() as &Vec<usage::SessionUsage>),
            };

        let input_cost = Self::token_cost(total_input, cost_rates.input);
        let output_cost = Self::token_cost(total_output, cost_rates.output);
        let cache_read_cost = Self::token_cost(total_cache_read, cost_rates.cache_read);
        let cache_write_cost = Self::token_cost(total_cache_write, cost_rates.cache_write);
        let total_cost = input_cost + output_cost + cache_read_cost + cache_write_cost;

        let token_header = text("TOKEN BREAKDOWN")
            .size(font_size * 0.7)
            .color(text_secondary);

        let make_token_row = |label: String, tokens: u64, cost: f64| -> Element<'_, Message> {
            row![
                text(label)
                    .size(font_size * 0.85)
                    .color(text_secondary)
                    .width(Length::FillPortion(2)),
                text(Self::format_tokens(tokens))
                    .size(font_size * 0.85)
                    .color(text_primary)
                    .width(Length::FillPortion(1)),
                text(Self::format_cost(cost))
                    .size(font_size * 0.85)
                    .color(text_primary)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .into()
        };

        let token_table = column![
            row![
                text("Type")
                    .size(font_size * 0.7)
                    .color(text_secondary)
                    .width(Length::FillPortion(2)),
                text("Tokens")
                    .size(font_size * 0.7)
                    .color(text_secondary)
                    .width(Length::FillPortion(1)),
                text("Cost")
                    .size(font_size * 0.7)
                    .color(text_secondary)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(8),
            make_token_row(String::from("Input"), total_input, input_cost),
            make_token_row(String::from("Output"), total_output, output_cost),
            make_token_row(String::from("Cache Read"), total_cache_read, cache_read_cost),
            make_token_row(String::from("Cache Write"), total_cache_write, cache_write_cost),
            container(Space::new().width(Length::Fill).height(1))
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(border_color)),
                    ..Default::default()
                }),
            row![
                text("Total")
                    .size(font_size * 0.85)
                    .color(accent)
                    .width(Length::FillPortion(2)),
                text(Self::format_tokens(
                    total_input + total_output + total_cache_read + total_cache_write,
                ))
                .size(font_size * 0.85)
                .color(text_primary)
                .width(Length::FillPortion(1)),
                text(Self::format_cost(total_cost))
                    .size(font_size * 0.85)
                    .color(accent)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(8),
        ]
        .spacing(6);

        let token_card = container(
            column![token_header, token_table].spacing(8).padding(16),
        )
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(surface_bg)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

        let rate_limit_card: Element<'_, Message> = {
            let rate_limits = self.config.rate_limits.get("claude").cloned().unwrap_or_default();
            let now = chrono::Utc::now();
            let five_hours_ago = now - chrono::Duration::hours(5);

            let five_hour_used: u64 = sessions
                .iter()
                .filter(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s.timestamp)
                        .map(|t| t >= five_hours_ago)
                        .unwrap_or(false)
                })
                .map(|s| s.input_tokens + s.output_tokens + s.cache_read_tokens + s.cache_write_tokens)
                .sum();

            let weekly_used: u64 = sessions
                .iter()
                .map(|s| s.input_tokens + s.output_tokens + s.cache_read_tokens + s.cache_write_tokens)
                .sum();

            let make_rate_bar = |label: String, used: u64, limit: u64| -> Element<'_, Message> {
                if limit == 0 {
                    return Space::new().height(0).into();
                }
                let pct = (used as f64 / limit as f64 * 100.0).min(100.0);
                let bar_color = if pct >= 90.0 {
                    ui.status_deleted.to_iced()
                } else if pct >= 70.0 {
                    ui.status_modified.to_iced()
                } else {
                    ui.accent.to_iced()
                };
                let bar_bg_color = ui.hover_overlay.to_iced_alpha(0.06);
                let fill_portion = ((pct * 100.0) as u16).max(1);
                let empty_portion = (10000_u16).saturating_sub(fill_portion).max(1);

                let usage_label = text(label)
                    .size(font_size * 0.8)
                    .color(text_secondary);
                let usage_value = text(format!("{} / {}", Self::format_tokens(used), Self::format_tokens(limit)))
                    .size(font_size * 0.8)
                    .color(text_primary);

                let bar_fill: Element<'_, Message> = container(Space::new().width(Length::Fill).height(6))
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(Background::Color(bar_color)),
                        border: Border { radius: 3.0.into(), ..Default::default() },
                        ..Default::default()
                    })
                    .width(Length::FillPortion(fill_portion))
                    .into();

                let bar_empty: Element<'_, Message> = container(Space::new().width(Length::Fill).height(6))
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(Background::Color(bar_bg_color)),
                        border: Border { radius: 3.0.into(), ..Default::default() },
                        ..Default::default()
                    })
                    .width(Length::FillPortion(empty_portion))
                    .into();

                column![
                    row![usage_label, Space::new().width(Length::Fill), usage_value]
                        .align_y(iced::Alignment::Center),
                    row![bar_fill, bar_empty].spacing(0),
                ]
                .spacing(4)
                .into()
            };

            let rate_header = text("RATE LIMITS")
                .size(font_size * 0.7)
                .color(text_secondary);

            let five_hour_bar = make_rate_bar(String::from("5h Window"), five_hour_used, rate_limits.five_hour_limit);
            let weekly_bar = make_rate_bar(String::from("Weekly"), weekly_used, rate_limits.weekly_limit);

            container(
                column![rate_header, five_hour_bar, weekly_bar].spacing(8).padding(16),
            )
            .width(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(surface_bg)),
                border: Border {
                    width: 1.0,
                    color: border_color,
                    radius: 6.0.into(),
                },
                ..Default::default()
            })
            .into()
        };

        let sessions_header = text("SESSIONS")
            .size(font_size * 0.7)
            .color(text_secondary);

        let session_items: Vec<Element<'_, Message>> = sessions
            .iter()
            .map(|session| {
                let session_cost = Self::token_cost(session.input_tokens, cost_rates.input)
                    + Self::token_cost(session.output_tokens, cost_rates.output)
                    + Self::token_cost(session.cache_read_tokens, cost_rates.cache_read)
                    + Self::token_cost(session.cache_write_tokens, cost_rates.cache_write);

                const SESSION_ID_DISPLAY_LEN: usize = 12;
                let id_display = if session.session_id.len() > SESSION_ID_DISPLAY_LEN {
                    let truncated: String = session.session_id.chars().take(SESSION_ID_DISPLAY_LEN).collect();
                    format!("{truncated}...")
                } else {
                    session.session_id.clone()
                };

                let total_tokens = session.input_tokens
                    + session.output_tokens
                    + session.cache_read_tokens
                    + session.cache_write_tokens;

                container(
                    row![
                        text(id_display)
                            .size(font_size * 0.8)
                            .color(text_primary)
                            .width(Length::FillPortion(3)),
                        text(Self::format_tokens(total_tokens))
                            .size(font_size * 0.8)
                            .color(text_secondary)
                            .width(Length::FillPortion(1)),
                        text(Self::format_cost(session_cost))
                            .size(font_size * 0.8)
                            .color(text_primary)
                            .width(Length::FillPortion(1)),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .padding([6, 12])
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(surface_bg)),
                    border: Border {
                        width: 1.0,
                        color: border_color,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect();

        let sessions_list =
            scrollable(column(session_items).spacing(4)).height(Length::Fill);

        let content = column![toolbar, token_card, rate_limit_card, sessions_header, sessions_list]
            .spacing(12)
            .padding(20)
            .width(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(bg_primary)),
                ..Default::default()
            })
            .into()
    }

    fn insights_git_view(&self) -> Element<'_, Message> {
        let ui = &self.ui_theme;
        let font_size = self.config.appearance.font_size as f32;
        let text_primary = ui.text_primary.to_iced();
        let text_secondary = ui.text_secondary.to_iced();
        let hover_color = ui.hover_overlay.to_iced_alpha(ui.hover_overlay_alpha);
        let surface_bg = ui.bg_surface.to_iced();
        let border_color = ui.border.to_iced();
        let bg_primary = ui.bg_primary.to_iced();
        let diff_add_bg = ui.diff_add.to_iced();
        let diff_del_bg = ui.diff_delete.to_iced();

        if let Some(diff) = &self.git_diff {
            let back_btn = button(
                text("\u{2190} Back")
                    .size(font_size * 0.85)
                    .color(text_secondary),
            )
            .on_press(Message::GitBackFromDiff)
            .padding([4, 12])
            .style(Self::ghost_button_style(text_secondary, hover_color));

            let file_header = text(&diff.path)
                .size(font_size)
                .color(text_primary);

            let mut diff_lines: Vec<Element<'_, Message>> = Vec::new();

            for hunk in &diff.hunks {
                for line in hunk {
                    let (line_bg, line_color) = match line.origin.as_str() {
                        "+" => (diff_add_bg, ui.text_primary.to_iced()),
                        "-" => (diff_del_bg, ui.text_primary.to_iced()),
                        _ => (iced::Color::TRANSPARENT, text_secondary),
                    };
                    let prefix = match line.origin.as_str() {
                        "+" => "+ ",
                        "-" => "- ",
                        _ => "  ",
                    };
                    let line_text = format!("{}{}", prefix, line.content.trim_end());
                    let line_bg_captured = line_bg;
                    let line_element: Element<'_, Message> = container(
                        text(line_text)
                            .size(font_size * 0.8)
                            .color(line_color)
                            .font(Font::MONOSPACE),
                    )
                    .width(Length::Fill)
                    .padding([1, 8])
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(Background::Color(line_bg_captured)),
                        ..Default::default()
                    })
                    .into();
                    diff_lines.push(line_element);
                }
            }

            let diff_content =
                scrollable(column(diff_lines).spacing(0)).height(Length::Fill);

            let content = column![back_btn, file_header, diff_content]
                .spacing(8)
                .padding(20)
                .width(Length::Fill);

            return container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(bg_primary)),
                    ..Default::default()
                })
                .into();
        }

        let branch_names: Vec<String> = self.git_branches.iter().map(|b| b.name.clone()).collect();
        let branch_picker = pick_list(
            branch_names,
            self.git_branch.clone(),
            Message::GitSwitchBranch,
        )
        .text_size(font_size * 0.9);

        let refresh_btn = button(
            text("\u{21BB}")
                .size(font_size * 0.9)
                .color(text_secondary),
        )
        .on_press(Message::InsightsRefresh)
        .padding([4, 8])
        .style(Self::ghost_button_style(text_secondary, hover_color));

        let header_row = row![branch_picker, Space::new().width(Length::Fill), refresh_btn]
            .align_y(iced::Alignment::Center);

        let files_header = text("CHANGED FILES")
            .size(font_size * 0.7)
            .color(text_secondary);

        let file_items: Vec<Element<'_, Message>> = self
            .git_files
            .iter()
            .map(|file| {
                let (status_char, status_color) = match file.status.as_str() {
                    "added" => ("A", ui.status_added.to_iced()),
                    "deleted" => ("D", ui.status_deleted.to_iced()),
                    _ => ("M", ui.status_modified.to_iced()),
                };

                let stats = format!("+{} -{}", file.additions, file.deletions);
                let file_path = file.path.clone();

                button(
                    row![
                        text(status_char)
                            .size(font_size * 0.8)
                            .color(status_color)
                            .width(Length::Fixed(font_size * 2.0)),
                        text(&file.path)
                            .size(font_size * 0.8)
                            .color(text_primary)
                            .width(Length::Fill),
                        text(stats)
                            .size(font_size * 0.75)
                            .color(text_secondary),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .on_press(Message::GitSelectFile(file_path))
                .padding([6, 12])
                .width(Length::Fill)
                .style(move |_theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered => Some(Background::Color(hover_color)),
                        _ => Some(Background::Color(surface_bg)),
                    };
                    button::Style {
                        background: bg,
                        text_color: text_primary,
                        border: Border {
                            width: 1.0,
                            color: border_color,
                            radius: 4.0.into(),
                        },
                        ..button::Style::default()
                    }
                })
                .into()
            })
            .collect();

        let empty_state: Element<'_, Message> = if self.git_files.is_empty() {
            container(
                text("No changes detected")
                    .size(font_size * 0.85)
                    .color(text_secondary),
            )
            .padding(20)
            .into()
        } else {
            column(file_items).spacing(4).into()
        };

        let files_list = scrollable(empty_state).height(Length::Fill);

        let content = column![header_row, files_header, files_list]
            .spacing(12)
            .padding(20)
            .width(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(bg_primary)),
                ..Default::default()
            })
            .into()
    }

    fn insights_info_view(&self) -> Element<'_, Message> {
        let ui = &self.ui_theme;
        let font_size = self.config.appearance.font_size as f32;
        let text_primary = ui.text_primary.to_iced();
        let text_secondary = ui.text_secondary.to_iced();
        let surface_bg = ui.bg_surface.to_iced();
        let border_color = ui.border.to_iced();
        let bg_primary = ui.bg_primary.to_iced();

        let make_info_row = |label: String, value: String| -> Element<'_, Message> {
            row![
                text(label)
                    .size(font_size * 0.85)
                    .color(text_secondary)
                    .width(Length::FillPortion(1)),
                text(value)
                    .size(font_size * 0.85)
                    .color(text_primary)
                    .width(Length::FillPortion(2)),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .into()
        };

        let app_header = text("APPLICATION")
            .size(font_size * 0.7)
            .color(text_secondary);
        let app_card = container(
            column![
                app_header,
                make_info_row(String::from("Name"), String::from("gmux")),
                make_info_row(String::from("Version"), String::from(env!("CARGO_PKG_VERSION"))),
            ]
            .spacing(8)
            .padding(16),
        )
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(surface_bg)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

        let sys_header = text("SYSTEM")
            .size(font_size * 0.7)
            .color(text_secondary);
        let sys_card = container(
            column![
                sys_header,
                make_info_row(String::from("OS"), String::from(std::env::consts::OS)),
                make_info_row(String::from("Architecture"), String::from(std::env::consts::ARCH)),
            ]
            .spacing(8)
            .padding(16),
        )
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(surface_bg)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

        let workspace_count = self.workspaces.len();
        let terminal_count: usize = self
            .workspaces
            .iter()
            .map(|ws| {
                ws.panes
                    .iter()
                    .map(|(_, content)| content.tabs.len())
                    .sum::<usize>()
            })
            .sum();

        let ws_header = text("WORKSPACES")
            .size(font_size * 0.7)
            .color(text_secondary);
        let ws_card = container(
            column![
                ws_header,
                make_info_row(String::from("Workspaces"), format!("{}", workspace_count)),
                make_info_row(String::from("Terminals"), format!("{}", terminal_count)),
            ]
            .spacing(8)
            .padding(16),
        )
        .width(Length::Fill)
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(surface_bg)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

        let content = column![app_card, sys_card, ws_card]
            .spacing(16)
            .padding(20)
            .max_width(600);

        let scrollable_content =
            scrollable(container(content).center_x(Length::Fill)).height(Length::Fill);

        container(scrollable_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(bg_primary)),
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

        let top_bar = self.top_bar_view();

        if self.settings_open {
            return column![top_bar, self.settings_view()].into();
        }

        let main_content: Element<'_, Message> = match self.active_view {
            AppView::Terminals => {
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
                    .style(Self::ghost_button_style(
                        text_secondary,
                        hover_color,
                    ));

                    let expand_col = container(
                        column![Space::new().height(Length::Fill), expand_btn],
                    )
                    .height(Length::Fill);

                    row![expand_col, grid].into()
                }
            }
            AppView::Insights => {
                self.insights_view()
            }
        };

        column![top_bar, main_content].into()
    }

    fn handle_shortcut(
        &mut self,
        action: shortcuts::Action,
    ) -> iced::Task<Message> {
        match action {
            shortcuts::Action::TabNew => self.update(Message::TabNew),
            shortcuts::Action::TabClose => {
                if let Some(ws) = self.workspaces.get(self.active_workspace) {
                    let pane = ws.focus;
                    if let Some(content) = ws.panes.get(pane) {
                        let idx = content.active_tab;
                        return self.update(Message::TabClose(pane, idx));
                    }
                }
                iced::Task::none()
            }
            shortcuts::Action::TabNext => self.update(Message::TabNext),
            shortcuts::Action::TabPrev => self.update(Message::TabPrev),
            shortcuts::Action::TabJump(idx) => {
                if let Some(ws) = self.workspaces.get(self.active_workspace) {
                    let pane = ws.focus;
                    return self
                        .update(Message::TabActivate(pane, idx));
                }
                iced::Task::none()
            }
            shortcuts::Action::PaneSplitHorizontal => self
                .update(Message::PaneSplit(pane_grid::Axis::Horizontal)),
            shortcuts::Action::PaneSplitVertical => self
                .update(Message::PaneSplit(pane_grid::Axis::Vertical)),
            shortcuts::Action::PaneClose => {
                self.update(Message::PaneClose)
            }
            shortcuts::Action::PaneFocusUp => {
                self.update(Message::PaneFocusUp)
            }
            shortcuts::Action::PaneFocusDown => {
                self.update(Message::PaneFocusDown)
            }
            shortcuts::Action::PaneFocusLeft => {
                self.update(Message::PaneFocusLeft)
            }
            shortcuts::Action::PaneFocusRight => {
                self.update(Message::PaneFocusRight)
            }
            shortcuts::Action::PaneMaximize => {
                self.update(Message::PaneToggleMaximized)
            }
            shortcuts::Action::WorkspaceNew => {
                self.update(Message::WorkspaceNew)
            }
            shortcuts::Action::SidebarToggle => {
                self.update(Message::SidebarToggle)
            }
            shortcuts::Action::SettingsToggle => {
                self.update(Message::SettingsToggle)
            }
            shortcuts::Action::Copy => {
                if let Some(ws) = self.workspaces.get(self.active_workspace) {
                    if let Some(content) = ws.panes.get(ws.focus) {
                        if let Some(tab) = content.active_tab() {
                            if let Some(selected) = tab.terminal.selection_text() {
                                if !selected.is_empty() {
                                    if let Some(cb) = self.clipboard.as_mut() {
                                        let _ = cb.set_contents(selected);
                                    }
                                }
                            }
                            tab.terminal.clear_selection();
                        }
                    }
                }
                iced::Task::none()
            }
            shortcuts::Action::Paste => {
                if let Some(ws) = self.workspaces.get(self.active_workspace) {
                    if let Some(content) = ws.panes.get(ws.focus) {
                        if let Some(tab) = content.active_tab() {
                            if let Some(cb) = self.clipboard.as_mut() {
                                if let Ok(text) = cb.get_contents() {
                                    tab.terminal.paste(&text);
                                }
                            }
                        }
                    }
                }
                iced::Task::none()
            }
            shortcuts::Action::CopyOrSigint => {
                if let Some(ws) = self.workspaces.get(self.active_workspace) {
                    if let Some(content) = ws.panes.get(ws.focus) {
                        if let Some(tab) = content.active_tab() {
                            let selected = tab
                                .terminal
                                .selection_text()
                                .filter(|s| !s.is_empty());
                            if let Some(text) = selected {
                                if let Some(cb) = self.clipboard.as_mut() {
                                    let _ = cb.set_contents(text);
                                }
                                tab.terminal.clear_selection();
                            } else {
                                tab.terminal.input(b"\x03");
                            }
                        }
                    }
                }
                iced::Task::none()
            }
            shortcuts::Action::Find
            | shortcuts::Action::FontSizeIncrease
            | shortcuts::Action::FontSizeDecrease
            | shortcuts::Action::FontSizeReset => iced::Task::none(),
        }
    }

    fn refresh_git_data(&mut self) {
        if let Some(ws) = self.workspaces.get(self.active_workspace) {
            let cwd = ws.cwd.to_string_lossy().to_string();
            self.git_branch = git::get_current_branch(cwd.clone()).ok().flatten();
            self.git_branches = git::get_branches(cwd.clone()).unwrap_or_default();
            self.git_files = git::get_git_status(cwd).unwrap_or_default();
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
        let tick =
            iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick);

        let keys = iced::event::listen_with(|event, _status, _window| {
            if let iced::event::Event::Keyboard(
                iced::keyboard::Event::KeyPressed {
                    key, modifiers, ..
                },
            ) = event
            {
                Some(Message::KeyEvent(key, modifiers))
            } else {
                None
            }
        });

        iced::Subscription::batch([tick, keys])
    }
}
