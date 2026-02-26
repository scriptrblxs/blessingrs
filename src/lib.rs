//! # Blessingrs
//! 
//! Blessingrs is a dynamic terminal styling library inspired by Python's `blessings`.
//! It provides a simple way to format strings using readable names and handles
//! terminal state cleanup automatically.
//!
//! ## Example
//! ```rust
//! use blessingrs::Terminal;
//! let term = Terminal::new();
//! println!("{}", term.style("bold_red_on_black", "Hello!"));
//! ```

use std::io::{self, Stdout, Write, BufWriter};
use std::time::Duration;
use crossterm::{
    cursor, execute, queue,
    event::{self, Event, KeyEvent, KeyEventKind},
    style::{self, Stylize, Color},
    terminal,
};

/// Struct for .size()
pub struct Size {
    x: u16,
    y: u16,
}

/// The main entry point for managing the terminal state.
pub struct Terminal {
    writer: BufWriter<Stdout>,
}

impl Terminal {
    /// Initializes the terminal, enables raw mode, and switches to the alternate screen.
    pub fn new() -> Self {
        let mut writer = BufWriter::new(io::stdout());
        terminal::enable_raw_mode().expect("Failed to enable raw mode");
        execute!(writer, terminal::EnterAlternateScreen, cursor::Hide).expect("Failed to setup terminal");
        Self { writer }
    }
    
    /// Returns the size of the terminal (returns a struct with .x and .y u16s).
    pub fn size(&self) -> Size {
        let (x, y) = terminal::size().unwrap_or((80, 24));
        return Size {x, y}
    }

    /// Flushes the buffer and pauses execution for `ms` milliseconds.
    /// Requires an active Tokio runtime.
    pub async fn sleep(&mut self, ms: u64) {
        self.flush();
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }

    /// Polls for input and executes a closure if a key event occurs.
    /// 
    /// This is non-blocking with a tiny timeout (1ms), making it 
    /// perfect for high-frequency game loops or animations.
    pub fn add_input_handler<F>(&mut self, mut handler: F) -> io::Result<()>
    where
        F: FnMut(KeyEvent),
    {
        // Poll briefly to see if an event is available
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                // Filter for Press to avoid double-triggers on Windows
                if key.kind == KeyEventKind::Press {
                    handler(key);
                }
            }
        }
        Ok(())
    }

    /// Formats text based on a style string: `[bold_]foreground[_on_background]`
    pub fn style(&self, style_spec: &str, text: &str) -> String {
        let mut is_bold = false;
        let mut spec = style_spec;

        if spec.starts_with("bold_") {
            is_bold = true;
            spec = &spec[5..];
        }

        let parts: Vec<&str> = spec.split("_on_").collect();

        let styled = match parts.as_slice() {
            [fg_name] => {
                let fg = self.parse_color(fg_name);
                text.with(fg)
            }
            [fg_name, bg_name] => {
                let fg = self.parse_color(fg_name);
                let bg = self.parse_color(bg_name);
                text.with(fg).on(bg)
            }
            _ => panic!("Invalid style format: {}. Use '[bold_]fg_on_bg'.", style_spec),
        };

        if is_bold {
            format!("{}", styled.bold())
        } else {
            format!("{}", styled)
        }
    }

    fn parse_color(&self, name: &str) -> Color {
        match name.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            "grey" | "gray" => Color::Grey,
            _ => panic!("Unknown color member: {}", name),
        }
    }

    pub fn move_to(&mut self, x: u16, y: u16) -> &mut Self {
        queue!(self.writer, cursor::MoveTo(x, y)).unwrap();
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        queue!(self.writer, terminal::Clear(terminal::ClearType::All)).unwrap();
        self
    }

    pub fn print(&mut self, text: &str) -> &mut Self {
        write!(self.writer, "{}", text).unwrap();
        self
    }

    pub fn flush(&mut self) {
        let _ = self.writer.flush();
    }

    pub fn location(&mut self, x: u16, y: u16) -> LocationGuard<'_> {
        let (saved_x, saved_y) = cursor::position().unwrap_or((0, 0));
        queue!(self.writer, cursor::MoveTo(x, y)).unwrap();
        
        LocationGuard {
            term: self,
            saved_x,
            saved_y,
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        execute!(self.writer, terminal::LeaveAlternateScreen, cursor::Show).ok();
        terminal::disable_raw_mode().ok();
    }
}

/// A guard that restores cursor position when it goes out of scope.
pub struct LocationGuard<'a> {
    term: &'a mut Terminal,
    saved_x: u16,
    saved_y: u16,
}

impl<'a> Drop for LocationGuard<'a> {
    fn drop(&mut self) {
        queue!(self.term.writer, cursor::MoveTo(self.saved_x, self.saved_y)).unwrap();
    }
}