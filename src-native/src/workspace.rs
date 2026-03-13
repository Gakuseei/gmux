use std::path::PathBuf;

use iced::widget::pane_grid;

use crate::config::Config;
use crate::terminal::Terminal;

pub struct Workspace {
    pub id: String,
    pub name: String,
    pub folder: Option<String>,
    pub panes: pane_grid::State<PaneContent>,
    pub focus: pane_grid::Pane,
    pub cwd: PathBuf,
}

pub struct PaneContent {
    pub tabs: Vec<TabEntry>,
    pub active_tab: usize,
}

pub struct TabEntry {
    pub id: String,
    pub name: String,
    pub terminal: Terminal,
    pub bypass: bool,
}

impl Workspace {
    pub fn new(
        name: &str,
        cwd: &PathBuf,
        config: &Config,
    ) -> Option<Self> {
        let tab = Self::create_tab(cwd, config)?;
        let content = PaneContent {
            tabs: vec![tab],
            active_tab: 0,
        };
        let (panes, pane) = pane_grid::State::new(content);

        Some(Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            folder: None,
            panes,
            focus: pane,
            cwd: cwd.clone(),
        })
    }

    pub fn create_tab(cwd: &PathBuf, config: &Config) -> Option<TabEntry> {
        let shell = &config.terminal.default_shell;
        let cwd_str = cwd.to_string_lossy();
        let scrollback = config.terminal.scrollback_lines as usize;
        let font_size = config.appearance.font_size as f32;
        let cell_width = font_size * 0.6;
        let cell_height = font_size * 1.2;

        let terminal = Terminal::new(
            shell,
            &cwd_str,
            80,
            24,
            scrollback,
            cell_width,
            cell_height,
        )
        .ok()?;

        Some(TabEntry {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::from("Shell"),
            terminal,
            bypass: false,
        })
    }

    pub fn create_tab_with_command(
        cwd: &PathBuf,
        config: &Config,
        command: &str,
        name: &str,
        bypass: bool,
    ) -> Option<TabEntry> {
        let cwd_str = cwd.to_string_lossy();
        let scrollback = config.terminal.scrollback_lines as usize;
        let font_size = config.appearance.font_size as f32;
        let cell_width = font_size * 0.6;
        let cell_height = font_size * 1.2;

        let terminal = Terminal::new(
            command,
            &cwd_str,
            80,
            24,
            scrollback,
            cell_width,
            cell_height,
        )
        .ok()?;

        Some(TabEntry {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            terminal,
            bypass,
        })
    }

    pub fn active_pane(&self) -> Option<&PaneContent> {
        self.panes.get(self.focus)
    }

    pub fn active_pane_mut(&mut self) -> Option<&mut PaneContent> {
        self.panes.get_mut(self.focus)
    }

    pub fn active_terminal(&self) -> Option<&Terminal> {
        self.active_pane()?.active_tab().map(|t| &t.terminal)
    }

    pub fn active_terminal_mut(&mut self) -> Option<&mut Terminal> {
        self.active_pane_mut()?
            .active_tab_mut()
            .map(|t| &mut t.terminal)
    }
}

impl PaneContent {
    pub fn active_tab(&self) -> Option<&TabEntry> {
        self.tabs.get(self.active_tab)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut TabEntry> {
        self.tabs.get_mut(self.active_tab)
    }
}
