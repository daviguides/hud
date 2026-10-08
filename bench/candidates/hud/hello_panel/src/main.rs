use hud::{Console, Panel, Tree};

fn main() {
    let tree = Tree::new("hud/")
        .child(Tree::new("src/").child("lib.rs").child("console.rs"))
        .child("README.md");
    Console::stdout().print(&Panel::new(tree).title("Layout"));
}
