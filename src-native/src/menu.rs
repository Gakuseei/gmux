#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub items: Vec<MenuItem>,
    pub position: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
    pub shortcut: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    Copy,
    Paste,
    SplitHorizontal,
    SplitVertical,
    ClosePane,
    Search,
}

pub fn terminal_menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem {
            label: String::from("Copy"),
            action: MenuAction::Copy,
            shortcut: Some(String::from("Ctrl+Shift+C")),
        },
        MenuItem {
            label: String::from("Paste"),
            action: MenuAction::Paste,
            shortcut: Some(String::from("Ctrl+Shift+V")),
        },
        MenuItem {
            label: String::from("Split Horizontal"),
            action: MenuAction::SplitHorizontal,
            shortcut: None,
        },
        MenuItem {
            label: String::from("Split Vertical"),
            action: MenuAction::SplitVertical,
            shortcut: None,
        },
        MenuItem {
            label: String::from("Close Pane"),
            action: MenuAction::ClosePane,
            shortcut: Some(String::from("Ctrl+Shift+X")),
        },
        MenuItem {
            label: String::from("Search"),
            action: MenuAction::Search,
            shortcut: Some(String::from("Ctrl+Shift+F")),
        },
    ]
}
