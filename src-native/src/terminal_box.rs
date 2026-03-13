use std::sync::Arc;
use std::time::{Duration, Instant};

use alacritty_terminal::index::{Column as TermColumn, Line, Point as TermPoint, Side as TermSide};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::color::Colors;
use alacritty_terminal::term::{self, TermMode};
use alacritty_terminal::vte::ansi::{Color as AnsiColor, CursorShape, NamedColor};

use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::widget::tree;
use iced::advanced::{Clipboard, Renderer as _, Shell, Widget};
use iced::alignment;
use iced::event::Event;
use iced::keyboard::key::Named;
use iced::keyboard::{Event as KeyEvent, Key, Modifiers};
use iced::mouse::{self, Button, Event as MouseEvent, ScrollDelta};
use iced::{Color, Element, Font, Length, Pixels, Point, Rectangle, Size, Theme};

use crate::terminal::{EventProxy, Terminal};
use crate::theme::ColorScheme;

const SCROLL_MULTIPLIER: f32 = 3.0;
const CLICK_TIMING_MS: u64 = 500;

pub struct TerminalBox<'a, Message> {
    terminal: &'a Arc<FairMutex<term::Term<EventProxy>>>,
    terminal_handle: &'a Terminal,
    color_scheme: &'a ColorScheme,
    font_size: f32,
    on_input: Box<dyn Fn(Vec<u8>) -> Message + 'a>,
    on_resize: Option<Box<dyn Fn(u16, u16) -> Message + 'a>>,
}

impl<'a, Message> TerminalBox<'a, Message> {
    pub fn new(
        terminal: &'a Terminal,
        color_scheme: &'a ColorScheme,
        font_size: f32,
        on_input: impl Fn(Vec<u8>) -> Message + 'a,
    ) -> Self {
        Self {
            terminal: &terminal.term,
            terminal_handle: terminal,
            color_scheme,
            font_size,
            on_input: Box::new(on_input),
            on_resize: None,
        }
    }

    pub fn on_resize(mut self, f: impl Fn(u16, u16) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(f));
        self
    }
}

enum ClickKind {
    Single,
    Double,
    Triple,
}

struct TerminalBoxState {
    is_focused: bool,
    modifiers: Modifiers,
    last_size: Option<(u16, u16)>,
    scroll_pixels: f32,
    click: Option<(ClickKind, Instant)>,
    dragging: bool,
}

impl TerminalBoxState {
    fn new() -> Self {
        Self {
            is_focused: true,
            modifiers: Modifiers::empty(),
            last_size: None,
            scroll_pixels: 0.0,
            click: None,
            dragging: false,
        }
    }
}

fn resolve_color(color: &AnsiColor, colors: &Colors, scheme: &ColorScheme) -> Color {
    match color {
        AnsiColor::Named(named) => {
            if let Some(rgb) = colors[*named] {
                return Color::from_rgb8(rgb.r, rgb.g, rgb.b);
            }
            named_color_fallback(*named, scheme)
        }
        AnsiColor::Spec(rgb) => Color::from_rgb8(rgb.r, rgb.g, rgb.b),
        AnsiColor::Indexed(idx) => {
            if let Some(rgb) = colors[*idx as usize] {
                return Color::from_rgb8(rgb.r, rgb.g, rgb.b);
            }
            indexed_color_fallback(*idx, scheme)
        }
    }
}

fn named_color_fallback(named: NamedColor, scheme: &ColorScheme) -> Color {
    let idx = match named {
        NamedColor::Black => 0,
        NamedColor::Red => 1,
        NamedColor::Green => 2,
        NamedColor::Yellow => 3,
        NamedColor::Blue => 4,
        NamedColor::Magenta => 5,
        NamedColor::Cyan => 6,
        NamedColor::White => 7,
        NamedColor::BrightBlack => 8,
        NamedColor::BrightRed => 9,
        NamedColor::BrightGreen => 10,
        NamedColor::BrightYellow => 11,
        NamedColor::BrightBlue => 12,
        NamedColor::BrightMagenta => 13,
        NamedColor::BrightCyan => 14,
        NamedColor::BrightWhite => 15,
        NamedColor::Foreground | NamedColor::BrightForeground => {
            return scheme.foreground.to_iced();
        }
        NamedColor::Background => {
            return scheme.background.to_iced();
        }
        NamedColor::Cursor => {
            return scheme.cursor.to_iced();
        }
        NamedColor::DimBlack => 0,
        NamedColor::DimRed => 1,
        NamedColor::DimGreen => 2,
        NamedColor::DimYellow => 3,
        NamedColor::DimBlue => 4,
        NamedColor::DimMagenta => 5,
        NamedColor::DimCyan => 6,
        NamedColor::DimWhite => 7,
        NamedColor::DimForeground => {
            return scheme.foreground.to_iced();
        }
    };
    if idx < scheme.ansi.len() {
        scheme.ansi[idx].to_iced()
    } else {
        scheme.foreground.to_iced()
    }
}

fn indexed_color_fallback(idx: u8, scheme: &ColorScheme) -> Color {
    if (idx as usize) < scheme.ansi.len() {
        return scheme.ansi[idx as usize].to_iced();
    }

    if idx >= 16 && idx <= 231 {
        let idx = idx - 16;
        let b = (idx % 6) as f32 / 5.0;
        let g = ((idx / 6) % 6) as f32 / 5.0;
        let r = (idx / 36) as f32 / 5.0;
        return Color::from_rgb(r, g, b);
    }

    if idx >= 232 {
        let gray = ((idx - 232) as f32 * 10.0 + 8.0) / 255.0;
        return Color::from_rgb(gray, gray, gray);
    }

    scheme.foreground.to_iced()
}

fn calculate_modifier_number(modifiers: &Modifiers) -> u8 {
    let mut n: u8 = 0;
    if modifiers.shift() {
        n |= 1;
    }
    if modifiers.alt() {
        n |= 2;
    }
    if modifiers.control() {
        n |= 4;
    }
    if modifiers.logo() {
        n |= 8;
    }
    n + 1
}

fn csi(code: &str, suffix: &str, modifiers: u8) -> Option<Vec<u8>> {
    if modifiers == 1 {
        Some(format!("\x1B[{code}{suffix}").into_bytes())
    } else {
        Some(format!("\x1B[{code};{modifiers}{suffix}").into_bytes())
    }
}

fn csi2(code: &str, modifiers: u8) -> Option<Vec<u8>> {
    if modifiers == 1 {
        Some(format!("\x1B[{code}").into_bytes())
    } else {
        Some(format!("\x1B[1;{modifiers}{code}").into_bytes())
    }
}

fn ss3(code: &str, modifiers: u8) -> Option<Vec<u8>> {
    if modifiers == 1 {
        Some(format!("\x1B\x4F{code}").into_bytes())
    } else {
        Some(format!("\x1B[1;{modifiers}{code}").into_bytes())
    }
}

impl<'a, Message: Clone> Widget<Message, Theme, iced::Renderer> for TerminalBox<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<TerminalBoxState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(TerminalBoxState::new())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut tree::Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fill).height(Length::Fill);
        layout::Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<TerminalBoxState>();
        let bounds = layout.bounds();

        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            return;
        }

        let term = self.terminal.lock();
        let content = term.renderable_content();
        let colors = content.colors;
        let cursor = content.cursor;
        let selection = content.selection;
        let _mode = content.mode;
        let display_offset = content.display_offset;

        let cell_width = self.terminal_handle.size.cell_width;
        let cell_height = self.terminal_handle.size.cell_height;
        let cols = self.terminal_handle.size.cols as usize;
        let rows = self.terminal_handle.size.rows as usize;

        let bg_color = self.color_scheme.background.to_iced();
        renderer.fill_quad(
            Quad {
                bounds,
                ..Default::default()
            },
            bg_color,
        );

        let view_origin = bounds.position();

        renderer.with_layer(bounds, |renderer: &mut iced::Renderer| {
            for indexed in content.display_iter {
                let point = indexed.point;
                let cell = indexed.cell;

                let col = point.column.0;
                let row = point.line.0 as usize;

                if row >= rows || col >= cols {
                    continue;
                }

                let x = view_origin.x + col as f32 * cell_width;
                let y = view_origin.y + row as f32 * cell_height;

                let is_selected = selection
                    .as_ref()
                    .map(|sel| sel.contains(point))
                    .unwrap_or(false);

                let (fg_color, bg_cell_color) = if cell.flags.contains(Flags::INVERSE) || is_selected {
                    let fg = resolve_color(&cell.bg, colors, self.color_scheme);
                    let bg = resolve_color(&cell.fg, colors, self.color_scheme);
                    if is_selected && !cell.flags.contains(Flags::INVERSE) {
                        (
                            self.color_scheme.selection_foreground.to_iced(),
                            self.color_scheme.selection_background.to_iced(),
                        )
                    } else {
                        (fg, bg)
                    }
                } else {
                    (
                        resolve_color(&cell.fg, colors, self.color_scheme),
                        resolve_color(&cell.bg, colors, self.color_scheme),
                    )
                };

                let cell_rect = Rectangle::new(
                    Point::new(x, y),
                    Size::new(cell_width, cell_height),
                );

                if bg_cell_color != bg_color || is_selected {
                    renderer.fill_quad(
                        Quad {
                            bounds: cell_rect,
                            ..Default::default()
                        },
                        bg_cell_color,
                    );
                }

                let ch = cell.c;
                if ch != ' ' && ch != '\t' && ch != '\0' {
                    let text = iced::advanced::text::Text {
                        content: String::from(ch),
                        bounds: Size::new(cell_width, cell_height),
                        size: Pixels(self.font_size),
                        line_height: iced::widget::text::LineHeight::Absolute(Pixels(cell_height)),
                        font: Font::MONOSPACE,
                        align_x: iced::advanced::text::Alignment::Default,
                        align_y: alignment::Vertical::Top,
                        shaping: iced::advanced::text::Shaping::Basic,
                        wrapping: iced::advanced::text::Wrapping::None,
                    };

                    renderer.fill_text(
                        text,
                        Point::new(x, y),
                        fg_color,
                        cell_rect,
                    );
                }

                if cell.flags.contains(Flags::UNDERLINE) {
                    let underline_y = y + cell_height - 1.0;
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(x, underline_y),
                                Size::new(cell_width, 1.0),
                            ),
                            ..Default::default()
                        },
                        fg_color,
                    );
                }

                if cell.flags.contains(Flags::STRIKEOUT) {
                    let strike_y = y + cell_height / 2.0;
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle::new(
                                Point::new(x, strike_y),
                                Size::new(cell_width, 1.0),
                            ),
                            ..Default::default()
                        },
                        fg_color,
                    );
                }
            }

            if display_offset == 0 {
                let cursor_col = cursor.point.column.0;
                let cursor_line = cursor.point.line.0 as usize;
                let cursor_x = view_origin.x + cursor_col as f32 * cell_width;
                let cursor_y = view_origin.y + cursor_line as f32 * cell_height;

                let cursor_color = self.color_scheme.cursor.to_iced();

                match cursor.shape {
                    CursorShape::Block if state.is_focused => {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(cursor_x, cursor_y),
                                    Size::new(cell_width, cell_height),
                                ),
                                ..Default::default()
                            },
                            cursor_color,
                        );
                    }
                    CursorShape::Block => {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(cursor_x, cursor_y),
                                    Size::new(cell_width, cell_height),
                                ),
                                border: iced::Border {
                                    width: 1.0,
                                    color: cursor_color,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            Color::TRANSPARENT,
                        );
                    }
                    CursorShape::Beam => {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(cursor_x, cursor_y),
                                    Size::new(2.0, cell_height),
                                ),
                                ..Default::default()
                            },
                            cursor_color,
                        );
                    }
                    CursorShape::Underline => {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(cursor_x, cursor_y + cell_height - 2.0),
                                    Size::new(cell_width, 2.0),
                                ),
                                ..Default::default()
                            },
                            cursor_color,
                        );
                    }
                    CursorShape::HollowBlock => {
                        renderer.fill_quad(
                            Quad {
                                bounds: Rectangle::new(
                                    Point::new(cursor_x, cursor_y),
                                    Size::new(cell_width, cell_height),
                                ),
                                border: iced::Border {
                                    width: 1.0,
                                    color: cursor_color,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            Color::TRANSPARENT,
                        );
                    }
                    CursorShape::Hidden => {}
                }
            }
        });
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<TerminalBoxState>();
        let bounds = layout.bounds();

        let cell_width = self.terminal_handle.size.cell_width;
        let cell_height = self.terminal_handle.size.cell_height;

        if cell_width > 0.0 && cell_height > 0.0 {
            let new_cols = (bounds.width / cell_width).floor() as u16;
            let new_rows = (bounds.height / cell_height).floor() as u16;
            if new_cols > 0 && new_rows > 0 {
                let current = state.last_size;
                if current.map(|(c, r)| c != new_cols || r != new_rows).unwrap_or(true) {
                    state.last_size = Some((new_cols, new_rows));
                    if let Some(ref on_resize) = self.on_resize {
                        shell.publish((on_resize)(new_cols, new_rows));
                    }
                }
            }
        }

        let is_app_cursor = self.terminal.lock().mode().contains(TermMode::APP_CURSOR);

        match event {
            Event::Keyboard(KeyEvent::KeyPressed {
                key: Key::Named(named),
                modified_key: Key::Named(modified_named),
                modifiers,
                text,
                ..
            }) if state.is_focused && named == modified_named => {
                let mod_no = calculate_modifier_number(&state.modifiers);

                let escape_code = match named {
                    Named::Insert => csi("2", "~", mod_no),
                    Named::Delete => csi("3", "~", mod_no),
                    Named::PageUp => {
                        if modifiers.shift() {
                            self.terminal_handle.scroll(-((self.terminal_handle.size.rows as i32)));
                            None
                        } else {
                            csi("5", "~", mod_no)
                        }
                    }
                    Named::PageDown => {
                        if modifiers.shift() {
                            self.terminal_handle.scroll(self.terminal_handle.size.rows as i32);
                            None
                        } else {
                            csi("6", "~", mod_no)
                        }
                    }
                    Named::ArrowUp => {
                        if is_app_cursor {
                            ss3("A", mod_no)
                        } else {
                            csi2("A", mod_no)
                        }
                    }
                    Named::ArrowDown => {
                        if is_app_cursor {
                            ss3("B", mod_no)
                        } else {
                            csi2("B", mod_no)
                        }
                    }
                    Named::ArrowRight => {
                        if is_app_cursor {
                            ss3("C", mod_no)
                        } else {
                            csi2("C", mod_no)
                        }
                    }
                    Named::ArrowLeft => {
                        if is_app_cursor {
                            ss3("D", mod_no)
                        } else {
                            csi2("D", mod_no)
                        }
                    }
                    Named::End => {
                        if is_app_cursor {
                            ss3("F", mod_no)
                        } else {
                            csi2("F", mod_no)
                        }
                    }
                    Named::Home => {
                        if is_app_cursor {
                            ss3("H", mod_no)
                        } else {
                            csi2("H", mod_no)
                        }
                    }
                    Named::F1 => ss3("P", mod_no),
                    Named::F2 => ss3("Q", mod_no),
                    Named::F3 => ss3("R", mod_no),
                    Named::F4 => ss3("S", mod_no),
                    Named::F5 => csi("15", "~", mod_no),
                    Named::F6 => csi("17", "~", mod_no),
                    Named::F7 => csi("18", "~", mod_no),
                    Named::F8 => csi("19", "~", mod_no),
                    Named::F9 => csi("20", "~", mod_no),
                    Named::F10 => csi("21", "~", mod_no),
                    Named::F11 => csi("23", "~", mod_no),
                    Named::F12 => csi("24", "~", mod_no),
                    _ => None,
                };

                if let Some(bytes) = escape_code {
                    shell.publish((self.on_input)(bytes));
                    return;
                }

                let alt_prefix = if state.modifiers.alt() { "\x1B" } else { "" };
                match named {
                    Named::Backspace => {
                        let code = if state.modifiers.control() {
                            "\x08"
                        } else {
                            "\x7f"
                        };
                        shell.publish((self.on_input)(
                            format!("{alt_prefix}{code}").into_bytes(),
                        ));
                    }
                    Named::Enter => {
                        shell.publish((self.on_input)(
                            format!("{alt_prefix}\x0D").into_bytes(),
                        ));
                    }
                    Named::Escape => {
                        shell.publish((self.on_input)(
                            format!("{alt_prefix}\x1B").into_bytes(),
                        ));
                    }
                    Named::Space => {
                        let character = text
                            .as_ref()
                            .and_then(|t| t.chars().next())
                            .unwrap_or(' ');
                        if state.modifiers.control() {
                            shell.publish((self.on_input)(b"\x00".to_vec()));
                        } else {
                            shell.publish((self.on_input)(
                                format!("{alt_prefix}{character}").into_bytes(),
                            ));
                        }
                    }
                    Named::Tab => {
                        let code = if state.modifiers.shift() {
                            "\x1b[Z"
                        } else {
                            "\x09"
                        };
                        shell.publish((self.on_input)(
                            format!("{alt_prefix}{code}").into_bytes(),
                        ));
                    }
                    _ => {}
                }
            }
            Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                state.modifiers = *modifiers;
            }
            Event::Keyboard(KeyEvent::KeyPressed {
                text,
                modifiers,
                key,
                ..
            }) if state.is_focused => {
                let character = text
                    .as_ref()
                    .and_then(|c| c.chars().next())
                    .unwrap_or_default();

                match (
                    modifiers.logo(),
                    modifiers.control(),
                    modifiers.alt(),
                    modifiers.shift(),
                ) {
                    (true, _, _, _) => {}
                    (false, true, true, _) => {
                        if !character.is_control() || (character as u32) < 32 {
                            let mut buf = [0x1B, 0, 0, 0, 0];
                            let len = {
                                let s = character.encode_utf8(&mut buf[1..]);
                                s.len() + 1
                            };
                            shell.publish((self.on_input)(buf[..len].to_vec()));
                        }
                    }
                    (false, true, _, false) => {
                        if character.is_control() {
                            let mut buf = [0, 0, 0, 0];
                            let s = character.encode_utf8(&mut buf);
                            shell.publish((self.on_input)(s.as_bytes().to_vec()));
                        }
                    }
                    (false, true, _, true) => {
                        if *key == Key::Character("_".into()) {
                            shell.publish((self.on_input)(b"\x1F".to_vec()));
                        }
                    }
                    (false, false, true, _) => {
                        if !character.is_control() {
                            let mut buf = [0x1B, 0, 0, 0, 0];
                            let len = {
                                let s = character.encode_utf8(&mut buf[1..]);
                                s.len() + 1
                            };
                            shell.publish((self.on_input)(buf[..len].to_vec()));
                        }
                    }
                    (false, false, false, _) => {
                        if !character.is_control() {
                            let mut buf = [0, 0, 0, 0];
                            let s = character.encode_utf8(&mut buf);
                            shell.publish((self.on_input)(s.as_bytes().to_vec()));
                        }
                    }
                }
            }
            Event::Mouse(MouseEvent::ButtonPressed(Button::Left)) => {
                if cursor.position_in(bounds).is_some() {
                    state.is_focused = true;
                    state.dragging = true;

                    if let Some(p) = cursor.position_in(bounds) {
                        let col = p.x / cell_width;
                        let row = p.y / cell_height;

                        let click_kind =
                            if let Some((prev_kind, prev_time)) = state.click.take() {
                                if prev_time.elapsed()
                                    < Duration::from_millis(CLICK_TIMING_MS)
                                {
                                    match prev_kind {
                                        ClickKind::Single => ClickKind::Double,
                                        ClickKind::Double => ClickKind::Triple,
                                        ClickKind::Triple => ClickKind::Single,
                                    }
                                } else {
                                    ClickKind::Single
                                }
                            } else {
                                ClickKind::Single
                            };

                        let location =
                            TermPoint::new(Line(row as i32), TermColumn(col as usize));
                        let side = if col.fract() < 0.5 {
                            TermSide::Left
                        } else {
                            TermSide::Right
                        };

                        let selection_type = match click_kind {
                            ClickKind::Single => SelectionType::Simple,
                            ClickKind::Double => SelectionType::Semantic,
                            ClickKind::Triple => SelectionType::Lines,
                        };

                        {
                            let mut term = self.terminal.lock();
                            term.selection =
                                Some(Selection::new(selection_type, location, side));
                        }

                        state.click = Some((click_kind, Instant::now()));
                    }
                }
            }
            Event::Mouse(MouseEvent::ButtonReleased(Button::Left)) => {
                state.dragging = false;
            }
            Event::Mouse(MouseEvent::CursorMoved { .. }) => {
                if state.dragging {
                    if let Some(p) = cursor.position_in(bounds) {
                        let col = p.x / cell_width;
                        let row = p.y / cell_height;
                        let location =
                            TermPoint::new(Line(row as i32), TermColumn(col as usize));
                        let side = if col.fract() < 0.5 {
                            TermSide::Left
                        } else {
                            TermSide::Right
                        };

                        let mut term = self.terminal.lock();
                        if let Some(ref mut selection) = term.selection {
                            selection.update(location, side);
                        }
                    }
                }
            }
            Event::Mouse(MouseEvent::WheelScrolled { delta }) => {
                if cursor.position_in(bounds).is_some() {
                    let lines = match delta {
                        ScrollDelta::Lines { y, .. } => (-y * SCROLL_MULTIPLIER) as i32,
                        ScrollDelta::Pixels { y, .. } => {
                            state.scroll_pixels -= y * SCROLL_MULTIPLIER;
                            let lines = (state.scroll_pixels / cell_height) as i32;
                            state.scroll_pixels -= lines as f32 * cell_height;
                            lines
                        }
                    };
                    if lines != 0 {
                        self.terminal_handle.scroll(lines);
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.position_in(layout.bounds()).is_some() {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message: Clone + 'a> From<TerminalBox<'a, Message>> for Element<'a, Message> {
    fn from(widget: TerminalBox<'a, Message>) -> Self {
        Self::new(widget)
    }
}
