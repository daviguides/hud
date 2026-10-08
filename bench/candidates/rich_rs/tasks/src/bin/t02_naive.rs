use rich_rs::{Console, Panel, Text};

fn main() {
    let mut console = Console::new();
    let text = "hud renders terminal output that reads at a glance. Tables, panels, trees and \
        progress share one width model, so wide characters such as 日本語 and emoji \
        keep every border aligned on any terminal.";
    let panel = Panel::new(Box::new(Text::plain(text))).with_title("Notice");
    console.print(&panel, None, None, None, false, "").unwrap();
}
