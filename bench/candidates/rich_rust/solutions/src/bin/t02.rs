use rich_rust::prelude::*;

fn main() {
    let console = Console::new();
    let text = Text::new(
        "hud renders terminal output that reads at a glance. Tables, panels, trees and \
         progress share one width model, so wide characters such as 日本語 and emoji \
         keep every border aligned on any terminal.",
    );
    let lines = text
        .wrap(console.width() - 4)
        .iter()
        .map(|line| line.render("").into_iter().map(Segment::into_owned).collect())
        .collect();
    console.print_renderable(&Panel::new(lines).title("Notice"));
}
