//! How the environment changes what is printed. `NO_COLOR` removes color and keeps bold;
//! `FORCE_COLOR` keeps styling on in a pipe.
//!
//! ```bash
//! cargo run -p hud --example env
//! NO_COLOR=1 cargo run -p hud --example env
//! FORCE_COLOR=1 cargo run -p hud --example env | cat
//! ```

use hud::{Stream, capabilities};

fn main() {
    let caps = capabilities(Stream::Stdout);
    hud::println!(
        "color={:?} attributes={} terminal={} interactive={}",
        caps.color_system,
        caps.attributes,
        caps.is_tty,
        caps.interactive
    );
    hud::println!("[bold red]error[/] [green]ok[/] [italic]plain[/]");
}
