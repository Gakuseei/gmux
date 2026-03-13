use std::collections::HashMap;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct KeyBind {
    pub modifiers: Modifiers,
    pub key: Key,
}

#[derive(Debug, Clone)]
pub enum Action {
    Copy,
    Paste,
    CopyOrSigint,
    TabNew,
    TabClose,
    TabNext,
    TabPrev,
    TabJump(usize),
    PaneSplitHorizontal,
    PaneSplitVertical,
    PaneClose,
    PaneFocusUp,
    PaneFocusDown,
    PaneFocusLeft,
    PaneFocusRight,
    PaneMaximize,
    WorkspaceNew,
    Find,
    FontSizeIncrease,
    FontSizeDecrease,
    FontSizeReset,
    SidebarToggle,
    SettingsToggle,
}

pub fn default_keybindings() -> HashMap<KeyBind, Action> {
    let ctrl_shift = Modifiers::CTRL | Modifiers::SHIFT;
    let ctrl_alt = Modifiers::CTRL | Modifiers::ALT;

    let mut map = HashMap::new();

    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("c".into()),
        },
        Action::Copy,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("v".into()),
        },
        Action::Paste,
    );
    map.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("c".into()),
        },
        Action::CopyOrSigint,
    );
    map.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("v".into()),
        },
        Action::Paste,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("t".into()),
        },
        Action::TabNew,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("w".into()),
        },
        Action::TabClose,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("n".into()),
        },
        Action::WorkspaceNew,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("f".into()),
        },
        Action::Find,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Named(Named::ArrowUp),
        },
        Action::PaneSplitHorizontal,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Named(Named::ArrowDown),
        },
        Action::PaneSplitHorizontal,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Named(Named::ArrowLeft),
        },
        Action::PaneSplitVertical,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Named(Named::ArrowRight),
        },
        Action::PaneSplitVertical,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_alt,
            key: Key::Named(Named::ArrowUp),
        },
        Action::PaneFocusUp,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_alt,
            key: Key::Named(Named::ArrowDown),
        },
        Action::PaneFocusDown,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_alt,
            key: Key::Named(Named::ArrowLeft),
        },
        Action::PaneFocusLeft,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_alt,
            key: Key::Named(Named::ArrowRight),
        },
        Action::PaneFocusRight,
    );
    map.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Named(Named::Tab),
        },
        Action::TabNext,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Named(Named::Tab),
        },
        Action::TabPrev,
    );

    for i in 1..=9u8 {
        map.insert(
            KeyBind {
                modifiers: ctrl_shift,
                key: Key::Character(format!("{i}").into()),
            },
            Action::TabJump((i - 1) as usize),
        );
    }

    map.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("+".into()),
        },
        Action::FontSizeIncrease,
    );
    map.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("=".into()),
        },
        Action::FontSizeIncrease,
    );
    map.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("-".into()),
        },
        Action::FontSizeDecrease,
    );
    map.insert(
        KeyBind {
            modifiers: Modifiers::CTRL,
            key: Key::Character("0".into()),
        },
        Action::FontSizeReset,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("m".into()),
        },
        Action::PaneMaximize,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("b".into()),
        },
        Action::SidebarToggle,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("s".into()),
        },
        Action::SettingsToggle,
    );
    map.insert(
        KeyBind {
            modifiers: ctrl_shift,
            key: Key::Character("x".into()),
        },
        Action::PaneClose,
    );

    map
}

pub fn lookup(
    bindings: &HashMap<KeyBind, Action>,
    modifiers: Modifiers,
    key: &Key,
) -> Option<Action> {
    let normalized_key = match key {
        Key::Character(c) => Key::Character(c.to_lowercase().into()),
        other => other.clone(),
    };

    bindings
        .get(&KeyBind {
            modifiers,
            key: normalized_key,
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keybinds_no_conflicts() {
        let bindings = default_keybindings();
        let mut seen = HashMap::new();
        for (keybind, _action) in &bindings {
            assert!(
                seen.insert(keybind.clone(), true).is_none(),
                "duplicate keybinding: {:?}",
                keybind
            );
        }
    }

    #[test]
    fn lookup_finds_binding() {
        let bindings = default_keybindings();
        let result = lookup(
            &bindings,
            Modifiers::CTRL | Modifiers::SHIFT,
            &Key::Character("t".into()),
        );
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), Action::TabNew));
    }

    #[test]
    fn lookup_returns_none_for_unbound() {
        let bindings = default_keybindings();
        let result = lookup(
            &bindings,
            Modifiers::CTRL | Modifiers::SHIFT,
            &Key::Character("z".into()),
        );
        assert!(result.is_none());
    }
}
