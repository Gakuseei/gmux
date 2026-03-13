#[derive(Clone, Copy)]
pub enum MouseEventType {
    Press,
    Release,
    Motion,
    ScrollUp,
    ScrollDown,
}

#[derive(Clone, Copy)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    None,
}

pub struct MouseModifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

#[derive(Clone, Copy)]
pub enum MouseMode {
    Normal,
    Utf8,
    Sgr,
}

fn button_code(event_type: &MouseEventType, button: &MouseButton, is_motion: bool) -> u8 {
    let base = match event_type {
        MouseEventType::ScrollUp => 64,
        MouseEventType::ScrollDown => 65,
        MouseEventType::Release => 3,
        MouseEventType::Press | MouseEventType::Motion => match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            MouseButton::None => 3,
        },
    };

    if is_motion {
        base + 32
    } else {
        base
    }
}

fn modifier_bits(modifiers: &MouseModifiers) -> u8 {
    let mut bits: u8 = 0;
    if modifiers.shift {
        bits |= 4;
    }
    if modifiers.alt {
        bits |= 8;
    }
    if modifiers.ctrl {
        bits |= 16;
    }
    bits
}

fn encode_normal(button_val: u8, col: usize, row: usize) -> Option<Vec<u8>> {
    if col > 222 || row > 222 {
        return None;
    }
    let cb = button_val + 32;
    let cx = (col as u8) + 33;
    let cy = (row as u8) + 33;
    Some(vec![b'\x1b', b'[', b'M', cb, cx, cy])
}

fn encode_utf8(button_val: u8, col: usize, row: usize) -> Option<Vec<u8>> {
    let mut buf = Vec::with_capacity(10);
    buf.extend_from_slice(b"\x1b[M");
    buf.push(button_val + 32);

    let cx = (col as u32) + 33;
    let cy = (row as u32) + 33;

    let cx_char = char::from_u32(cx)?;
    let cy_char = char::from_u32(cy)?;

    let mut tmp = [0u8; 4];
    let encoded = cx_char.encode_utf8(&mut tmp);
    buf.extend_from_slice(encoded.as_bytes());

    let encoded = cy_char.encode_utf8(&mut tmp);
    buf.extend_from_slice(encoded.as_bytes());

    Some(buf)
}

fn encode_sgr(
    event_type: &MouseEventType,
    button_val: u8,
    col: usize,
    row: usize,
) -> Option<Vec<u8>> {
    let suffix = match event_type {
        MouseEventType::Release => 'm',
        _ => 'M',
    };
    let col_1 = col + 1;
    let row_1 = row + 1;
    Some(format!("\x1b[<{button_val};{col_1};{row_1}{suffix}").into_bytes())
}

pub fn encode_mouse_event(
    event_type: MouseEventType,
    button: MouseButton,
    col: usize,
    row: usize,
    modifiers: MouseModifiers,
    mode: MouseMode,
) -> Option<Vec<u8>> {
    let is_motion = matches!(event_type, MouseEventType::Motion);
    let btn = button_code(&event_type, &button, is_motion);
    let mods = modifier_bits(&modifiers);
    let button_val = btn | mods;

    match mode {
        MouseMode::Normal => encode_normal(button_val, col, row),
        MouseMode::Utf8 => encode_utf8(button_val, col, row),
        MouseMode::Sgr => encode_sgr(&event_type, button_val, col, row),
    }
}
