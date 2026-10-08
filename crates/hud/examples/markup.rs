//! Inline markup: nested tags, colors, attributes, and an escaped bracket.
//!
//! ```bash
//! cargo run -p hud --example markup
//! ```

fn main() {
    hud::println!("[bold]Backup[/bold] [green]finished[/green] in [italic]41s[/italic]");
    hud::println!("[underline]nested [bold magenta]styles[/] close in order[/]");
    hud::println!("[white on blue] INFO [/] a literal bracket needs a backslash: \\[ok]");
    hud::eprintln!("[yellow]warning[/]: {} files skipped", 3);
}
