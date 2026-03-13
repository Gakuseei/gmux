use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

fn default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"))
}

fn default_accent_color() -> String {
    String::from("#10a37f")
}

fn default_font_ui() -> String {
    String::from("Inter, system-ui, sans-serif")
}

fn default_font_code() -> String {
    String::from("JetBrains Mono, monospace")
}

fn default_font_size() -> u32 {
    14
}

fn default_scrollback_lines() -> u32 {
    10000
}

fn default_desktop_enabled() -> bool {
    true
}

fn default_sidebar_width() -> f32 {
    250.0
}

fn default_window_width() -> f32 {
    1400.0
}

fn default_window_height() -> f32 {
    900.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub appearance: AppearanceConfig,
    pub terminal: TerminalConfig,
    pub ai_clis: AiClisConfig,
    pub rate_limits: HashMap<String, RateLimitConfig>,
    pub notifications: NotificationConfig,
    pub cost_rates: HashMap<String, CostRateConfig>,
    pub keybindings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    #[serde(default = "default_font_ui")]
    pub font_ui: String,
    #[serde(default = "default_font_code")]
    pub font_code: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    #[serde(default = "default_shell")]
    pub default_shell: String,
    #[serde(default = "default_scrollback_lines")]
    pub scrollback_lines: u32,
    pub cursor_style: CursorStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorStyle {
    Block,
    Bar,
    Underline,
}

impl CursorStyle {
    pub const ALL: [Self; 3] = [Self::Block, Self::Bar, Self::Underline];
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::Block
    }
}

impl std::fmt::Display for CursorStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block => write!(f, "Block"),
            Self::Bar => write!(f, "Bar"),
            Self::Underline => write!(f, "Underline"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiClisConfig {
    pub claude: CliEntry,
    pub codex: CliEntry,
    pub gemini: CliEntry,
    pub custom: Vec<CustomCliEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliEntry {
    pub path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCliEntry {
    pub name: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RateLimitConfig {
    pub five_hour_limit: u64,
    pub weekly_limit: u64,
    pub reset_day: String,
    pub reset_hour: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationConfig {
    #[serde(default = "default_desktop_enabled")]
    pub desktop_enabled: bool,
    pub sound_enabled: bool,
    pub custom_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CostRateConfig {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub workspaces: Vec<WorkspaceState>,
    pub folders: Vec<FolderState>,
    pub active_workspace_id: Option<String>,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,
    pub sidebar_minimized: bool,
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    pub window_x: i32,
    pub window_y: i32,
    pub recent_paths: Vec<RecentPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub id: String,
    pub name: String,
    pub folder_id: Option<String>,
    pub cwd: String,
    pub sessions: Vec<SessionState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderState {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub id: String,
    pub name: String,
    pub shell: String,
    pub cwd: String,
    pub command: Option<String>,
    pub bypass_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentPath {
    pub path: String,
    pub frequency: u32,
    pub last_used: String,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            accent_color: default_accent_color(),
            font_ui: default_font_ui(),
            font_code: default_font_code(),
            font_size: default_font_size(),
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            default_shell: default_shell(),
            scrollback_lines: default_scrollback_lines(),
            cursor_style: CursorStyle::default(),
        }
    }
}

impl Default for AiClisConfig {
    fn default() -> Self {
        Self {
            claude: CliEntry {
                path: String::from("claude"),
                enabled: true,
            },
            codex: CliEntry {
                path: String::from("codex"),
                enabled: true,
            },
            gemini: CliEntry {
                path: String::from("gemini"),
                enabled: true,
            },
            custom: Vec::new(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            five_hour_limit: 0,
            weekly_limit: 0,
            reset_day: String::from("monday"),
            reset_hour: 0,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            desktop_enabled: default_desktop_enabled(),
            sound_enabled: false,
            custom_patterns: Vec::new(),
        }
    }
}

impl Default for CostRateConfig {
    fn default() -> Self {
        Self {
            input: 0.0,
            output: 0.0,
            cache_read: 0.0,
            cache_write: 0.0,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            workspaces: Vec::new(),
            folders: Vec::new(),
            active_workspace_id: None,
            sidebar_width: default_sidebar_width(),
            sidebar_minimized: false,
            window_width: default_window_width(),
            window_height: default_window_height(),
            window_x: 0,
            window_y: 0,
            recent_paths: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut rate_limits = HashMap::new();
        rate_limits.insert(
            String::from("claude"),
            RateLimitConfig {
                five_hour_limit: 1_000_000,
                weekly_limit: 5_000_000,
                reset_day: String::from("monday"),
                reset_hour: 0,
            },
        );
        rate_limits.insert(
            String::from("codex"),
            RateLimitConfig::default(),
        );
        rate_limits.insert(
            String::from("gemini"),
            RateLimitConfig::default(),
        );

        let mut cost_rates = HashMap::new();
        cost_rates.insert(
            String::from("claude"),
            CostRateConfig {
                input: 3.0,
                output: 15.0,
                cache_read: 0.3,
                cache_write: 3.75,
            },
        );
        cost_rates.insert(
            String::from("codex"),
            CostRateConfig {
                input: 2.5,
                output: 10.0,
                cache_read: 0.25,
                cache_write: 3.0,
            },
        );
        cost_rates.insert(
            String::from("gemini"),
            CostRateConfig {
                input: 1.25,
                output: 5.0,
                cache_read: 0.3,
                cache_write: 1.25,
            },
        );

        let mut keybindings = HashMap::new();
        keybindings.insert(String::from("splitHorizontal"), String::from("Ctrl+Shift+D"));
        keybindings.insert(String::from("splitVertical"), String::from("Ctrl+Shift+R"));
        keybindings.insert(String::from("closePane"), String::from("Ctrl+Shift+W"));
        keybindings.insert(String::from("newWorkspace"), String::from("Ctrl+Shift+N"));
        keybindings.insert(String::from("newTerminal"), String::from("Ctrl+Shift+T"));
        keybindings.insert(String::from("toggleSidebar"), String::from("Ctrl+B"));
        keybindings.insert(String::from("search"), String::from("Ctrl+Shift+F"));
        keybindings.insert(String::from("nextPane"), String::from("Ctrl+Tab"));
        keybindings.insert(String::from("prevPane"), String::from("Ctrl+Shift+Tab"));

        Self {
            appearance: AppearanceConfig::default(),
            terminal: TerminalConfig::default(),
            ai_clis: AiClisConfig::default(),
            rate_limits,
            notifications: NotificationConfig::default(),
            cost_rates,
            keybindings,
        }
    }
}

pub fn config_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("/"))))
        .join("gmux");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn scrollback_dir() -> PathBuf {
    let dir = config_dir().join("scrollback");
    fs::create_dir_all(&dir).ok();
    dir
}

fn atomic_write(path: &Path, data: &[u8]) -> anyhow::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

impl Config {
    pub fn load() -> Self {
        let path = config_dir().join("settings.ron");
        match fs::read_to_string(&path) {
            Ok(contents) => ron::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_dir().join("settings.ron");
        let pretty = ron::ser::PrettyConfig::default();
        let data = ron::ser::to_string_pretty(self, pretty)?;
        atomic_write(&path, data.as_bytes())
    }
}

impl AppState {
    pub fn load() -> Self {
        let path = config_dir().join("state.json");
        match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_dir().join("state.json");
        let data = serde_json::to_string_pretty(self)?;
        atomic_write(&path, data.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_roundtrip() {
        let config = Config::default();
        let serialized = ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default())
            .expect("serialize config");
        let deserialized: Config = ron::from_str(&serialized).expect("deserialize config");

        assert_eq!(deserialized.appearance.accent_color, config.appearance.accent_color);
        assert_eq!(deserialized.appearance.font_ui, config.appearance.font_ui);
        assert_eq!(deserialized.appearance.font_code, config.appearance.font_code);
        assert_eq!(deserialized.appearance.font_size, config.appearance.font_size);
        assert_eq!(deserialized.terminal.scrollback_lines, config.terminal.scrollback_lines);
        assert_eq!(deserialized.terminal.default_shell, config.terminal.default_shell);
        assert_eq!(deserialized.notifications.desktop_enabled, config.notifications.desktop_enabled);
        assert_eq!(deserialized.notifications.sound_enabled, config.notifications.sound_enabled);
        assert_eq!(deserialized.keybindings.len(), config.keybindings.len());
        assert_eq!(deserialized.rate_limits.len(), config.rate_limits.len());
        assert_eq!(deserialized.cost_rates.len(), config.cost_rates.len());

        let claude_rate = deserialized.rate_limits.get("claude").expect("claude rate limit");
        assert_eq!(claude_rate.five_hour_limit, 1_000_000);
        assert_eq!(claude_rate.weekly_limit, 5_000_000);

        let claude_cost = deserialized.cost_rates.get("claude").expect("claude cost rate");
        assert!((claude_cost.input - 3.0).abs() < f64::EPSILON);
        assert!((claude_cost.output - 15.0).abs() < f64::EPSILON);

        let codex_cost = deserialized.cost_rates.get("codex").expect("codex cost rate");
        assert!((codex_cost.input - 2.5).abs() < f64::EPSILON);
        assert!((codex_cost.output - 10.0).abs() < f64::EPSILON);

        let gemini_cost = deserialized.cost_rates.get("gemini").expect("gemini cost rate");
        assert!((gemini_cost.input - 1.25).abs() < f64::EPSILON);
        assert!((gemini_cost.output - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn app_state_default_roundtrip() {
        let state = AppState::default();
        let serialized = serde_json::to_string_pretty(&state).expect("serialize state");
        let deserialized: AppState = serde_json::from_str(&serialized).expect("deserialize state");

        assert_eq!(deserialized.workspaces.len(), 0);
        assert_eq!(deserialized.folders.len(), 0);
        assert!(deserialized.active_workspace_id.is_none());
        assert!((deserialized.sidebar_width - 250.0).abs() < f32::EPSILON);
        assert!(!deserialized.sidebar_minimized);
        assert!((deserialized.window_width - 1400.0).abs() < f32::EPSILON);
        assert!((deserialized.window_height - 900.0).abs() < f32::EPSILON);
        assert_eq!(deserialized.window_x, 0);
        assert_eq!(deserialized.window_y, 0);
        assert_eq!(deserialized.recent_paths.len(), 0);
    }

    #[test]
    fn atomic_write_creates_file() {
        let dir = std::env::temp_dir().join(format!("gmux_test_{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join("test_file.txt");
        let content = b"gmux atomic write test";

        atomic_write(&path, content).expect("atomic write");

        assert!(path.exists());
        let read_back = fs::read(&path).expect("read back");
        assert_eq!(read_back, content);

        fs::remove_dir_all(&dir).ok();
    }
}
