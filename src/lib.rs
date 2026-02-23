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
use crossterm::{
    cursor, execute, queue,
    style::{self, Stylize, Color},
    terminal,
};

/// The main entry point for managing the terminal state.
/// When created, it enters raw mode and switches to the alternate screen.
/// When dropped, it restores the terminal to its original state.
pub struct Terminal {
    writer: BufWriter<Stdout>,
}

impl Terminal {
    /// Initializes the terminal, enables raw mode, and hides the cursor.
    pub fn new() -> Self {
        let mut writer = BufWriter::new(io::stdout());
        terminal::enable_raw_mode().unwrap();
        execute!(writer, terminal::EnterAlternateScreen, cursor::Hide).unwrap();
        Self { writer }
    }

    /// Formats text based on a style string.
    /// 
    /// Supported format: `[bold_]foreground[_on_background]`
    /// Examples: "red", "bold_blue", "green_on_black", "bold_white_on_blue"
    /// 
    /// # Panics
    /// Panics if a color name is not recognized or if the format is invalid.
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

    /// Returns the current terminal size as (width, height).
    pub fn size(&self) -> (u16, u16) {
        terminal::size().unwrap_or((80, 24))
    }

    /// Moves the cursor to the specified coordinates.
    pub fn move_to(&mut self, x: u16, y: u16) -> &mut Self {
        queue!(self.writer, cursor::MoveTo(x, y)).unwrap();
        self
    }

    /// Clears the entire terminal screen.
    pub fn clear(&mut self) -> &mut Self {
        queue!(self.writer, terminal::Clear(terminal::ClearType::All)).unwrap();
        self
    }

    /// Appends text to the internal buffer. Use `flush()` to render to screen.
    pub fn print(&mut self, text: &str) -> &mut Self {
        write!(self.writer, "{}", text).unwrap();
        self
    }

    /// Flushes the buffer, sending all queued commands to the terminal.
    pub fn flush(&mut self) {
        let _ = self.writer.flush();
    }

    /// Moves the cursor to a location and returns a guard.
    /// When the guard is dropped, the cursor returns to its previous position.
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

/// A guard that restores the cursor position when it goes out of scope.
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