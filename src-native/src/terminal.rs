use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use alacritty_terminal::event::{Event, EventListener, Notify, OnResize, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Notifier, State as LoopState};
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Flags as CellFlags;
use alacritty_terminal::term::{Config as TermConfig, TermMode};
use alacritty_terminal::tty::{self, Options, Shell};
use tokio::sync::mpsc;

pub enum TerminalEvent {
    TitleChanged(String),
    Bell,
    ClipboardStore(String),
    ChildExit(i32),
    Wakeup,
}

#[derive(Clone)]
pub struct EventProxy {
    sender: mpsc::UnboundedSender<TerminalEvent>,
}

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        let mapped = match event {
            Event::Title(title) => Some(TerminalEvent::TitleChanged(title)),
            Event::Bell => Some(TerminalEvent::Bell),
            Event::ClipboardStore(_, text) => Some(TerminalEvent::ClipboardStore(text)),
            Event::ChildExit(code) => Some(TerminalEvent::ChildExit(code)),
            Event::Wakeup => Some(TerminalEvent::Wakeup),
            _ => None,
        };
        if let Some(ev) = mapped {
            let _ = self.sender.send(ev);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
    pub cell_width: f32,
    pub cell_height: f32,
}

impl Dimensions for TerminalSize {
    fn total_lines(&self) -> usize {
        self.rows as usize
    }

    fn screen_lines(&self) -> usize {
        self.rows as usize
    }

    fn columns(&self) -> usize {
        self.cols as usize
    }
}

impl From<TerminalSize> for WindowSize {
    fn from(size: TerminalSize) -> Self {
        Self {
            num_lines: size.rows,
            num_cols: size.cols,
            cell_width: size.cell_width as u16,
            cell_height: size.cell_height as u16,
        }
    }
}

pub struct Terminal {
    pub term: Arc<FairMutex<alacritty_terminal::Term<EventProxy>>>,
    pub notifier: Notifier,
    pub size: TerminalSize,
    pub needs_update: bool,
    pub id: String,
    event_rx: mpsc::UnboundedReceiver<TerminalEvent>,
    _pty_thread: thread::JoinHandle<(EventLoop<tty::Pty, EventProxy>, LoopState)>,
}

impl Terminal {
    pub fn new(
        shell: &str,
        cwd: &str,
        cols: u16,
        rows: u16,
        scrollback_lines: usize,
        cell_width: f32,
        cell_height: f32,
    ) -> io::Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let event_proxy = EventProxy { sender: event_tx };

        let size = TerminalSize {
            cols,
            rows,
            cell_width,
            cell_height,
        };

        let mut term_config = TermConfig::default();
        term_config.scrolling_history = scrollback_lines;

        let term = Arc::new(FairMutex::new(alacritty_terminal::Term::new(
            term_config,
            &size,
            event_proxy.clone(),
        )));

        let mut env = HashMap::new();
        env.insert(String::from("TERM"), String::from("xterm-256color"));
        env.insert(String::from("COLORTERM"), String::from("truecolor"));

        let pty_options = Options {
            shell: Some(Shell::new(shell.to_string(), Vec::new())),
            working_directory: Some(PathBuf::from(cwd)),
            env,
            ..Options::default()
        };

        let window_id = 0;
        let pty = tty::new(&pty_options, size.into(), window_id)?;

        let pty_event_loop = EventLoop::new(
            term.clone(),
            event_proxy,
            pty,
            pty_options.drain_on_exit,
            false,
        )?;
        let notifier = Notifier(pty_event_loop.channel());
        let pty_thread = pty_event_loop.spawn();

        Ok(Self {
            term,
            notifier,
            size,
            needs_update: true,
            id: uuid::Uuid::new_v4().to_string(),
            event_rx,
            _pty_thread: pty_thread,
        })
    }

    pub fn input(&self, bytes: &[u8]) {
        self.notifier.notify(bytes.to_vec());
    }

    pub fn paste(&self, text: &str) {
        let bracketed_paste = {
            let term = self.term.lock();
            term.mode().contains(TermMode::BRACKETED_PASTE)
        };
        if bracketed_paste {
            self.notifier.notify(&b"\x1b[200~"[..]);
            self.notifier
                .notify(text.replace('\x1b', "").into_bytes());
            self.notifier.notify(&b"\x1b[201~"[..]);
        } else {
            self.notifier.notify(
                text.replace("\r\n", "\r")
                    .replace('\n', "\r")
                    .into_bytes(),
            );
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if cols != self.size.cols || rows != self.size.rows {
            self.size.cols = cols;
            self.size.rows = rows;
            self.notifier.on_resize(self.size.into());
            self.term.lock().resize(self.size);
            self.needs_update = true;
        }
    }

    pub fn scroll(&self, delta: i32) {
        self.term.lock().scroll_display(Scroll::Delta(delta));
    }

    pub fn selection_text(&self) -> Option<String> {
        self.term.lock().selection_to_string()
    }

    pub fn clear_selection(&self) {
        self.term.lock().selection = None;
    }

    pub fn try_recv_event(&mut self) -> Option<TerminalEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn grid_content(&self) -> String {
        let term = self.term.lock();
        let grid = term.grid();
        let cols = grid.columns();
        let top = grid.topmost_line();
        let bottom = grid.bottommost_line();
        let mut result = String::new();
        let mut line = top;
        while line <= bottom {
            let row = &grid[line];
            let mut row_text = String::with_capacity(cols);
            for col_idx in 0..cols {
                let cell = &row[Column(col_idx)];
                if cell.flags.contains(CellFlags::WIDE_CHAR_SPACER) {
                    continue;
                }
                row_text.push(cell.c);
            }
            result.push_str(row_text.trim_end());
            if line < bottom {
                result.push('\n');
            }
            line = Line(line.0 + 1);
        }
        result
    }

    pub fn last_line(&self) -> String {
        let term = self.term.lock();
        let grid = term.grid();
        let cursor_line = grid.cursor.point.line;
        let cols = grid.columns();
        let mut result = String::with_capacity(cols);
        let row = &grid[cursor_line];
        for col_idx in 0..cols {
            let cell = &row[Column(col_idx)];
            if cell.flags.contains(CellFlags::WIDE_CHAR_SPACER) {
                continue;
            }
            result.push(cell.c);
        }
        result.trim_end().to_string()
    }
}
