# Blessingrs

![Build Status](https://github.com/YOUR_USERNAME/blessingrs/actions/workflows/ci.yml/badge.svg)

Blessingrs is a lightweight terminal styling and manipulation library for Rust. Inspired by the Python blessings package, it provides a dynamic approach to terminal formatting using string specifications.

Built on top of the crossterm engine, Blessingrs manages raw mode, screen switching, and cursor movement automatically.

## Features

* Dynamic Styling: Format text using string specs like "bold_red_on_black" or "cyan_on_white".
* Member Validation: Immediate feedback via panic if an unsupported color is used.
* Location Guards: Move the cursor to print and have it automatically return to its original position when the guard drops.
* RAII Management: Automatic cleanup of raw mode and alternate screens when the Terminal struct is dropped.
* Efficient Memory Use: Leverages Rust's &str references to avoid unnecessary allocations.

## Quick Start

### Basic Usage

```rust
use blessingrs::Terminal;

fn main() {
    // Initializes terminal, enters raw mode, and hides cursor
    let mut term = Terminal::new();

    // Create styled strings
    let warning = term.style("bold_yellow_on_red", " ALERT ");
    let message = term.style("cyan", "System update in progress...");

    term.move_to(2, 2)
        .print(&warning)
        .print(" ")
        .print(&message)
        .flush();

    // Restoration happens automatically when term goes out of scope
}
```

### Using Location Guards

```rust
{
    let _guard = term.location(0, 0);
    term.print("Updating status...").flush();
} // Cursor jumps back to original position here
```

## Supported Style Members

The style method parses strings in the format [bold_]foreground[_on_background].

* Colors: black, red, green, yellow, blue, magenta, cyan, white, grey
* Modifiers: bold

Example valid specifications:
* "red"
* "bold_green"
* "white_on_blue"
* "bold_magenta_on_black"

## Development and Testing

To test the parsing logic and panic safety:

```bash
cargo test
```

To see a live demonstration of colors:

```bash
cargo run --example gallery
```

## License

Distributed under the MIT License.