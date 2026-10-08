//! An error report built by hand, then one built from a `std::error::Error` chain.
//!
//! ```bash
//! cargo run -p hud --example error
//! ```

use hud::{Console, ErrorReport};

fn main() {
    let console = Console::stdout();

    let by_hand = ErrorReport::new("could not start the server")
        .cause("address already in use (os error 48)")
        .cause("port: 8080")
        .hint("stop the other process or pass --port");
    console.print(&by_hand);

    if let Err(io_error) = std::fs::read_to_string("/nonexistent/settings.toml") {
        console.print(&ErrorReport::from_error(&io_error));
    }
}
