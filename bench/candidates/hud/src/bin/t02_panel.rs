use hud::{Console, Panel};

fn main() {
    let text = "hud renders terminal output that reads at a glance. Tables, panels, trees and progress share one width model, so wide characters such as 日本語 and emoji keep every border aligned on any terminal.";
    Console::stdout().print(&Panel::new(text).title("Notice"));
}
