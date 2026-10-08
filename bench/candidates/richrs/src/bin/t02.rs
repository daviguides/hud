use richrs::console::ColorSystem;
use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();
    if !console.is_terminal() || std::env::var_os("NO_COLOR").is_some() {
        console.set_color_system(ColorSystem::None);
    }

    let panel = Panel::new(
        "hud renders terminal output that reads at a glance. Tables, panels, trees and \
         progress share one width model, so wide characters such as 日本語 and emoji keep \
         every border aligned on any terminal.",
    )
    .title("Notice");

    console.write_segments(&panel.render(console.width()))
}
