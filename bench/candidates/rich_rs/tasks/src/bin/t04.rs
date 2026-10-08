use rich_rs::{Console, Text, Tree};

fn main() {
    let mut console = Console::new();
    let mut tree = Tree::new(Box::new(Text::plain("hud/")));
    let src = tree.add(Box::new(Text::plain("src/")));
    src.add(Box::new(Text::plain("lib.rs")));
    src.add(Box::new(Text::plain("console.rs")));
    src.add(Box::new(Text::plain("width.rs")));
    tree.add(Box::new(Text::plain("tests/"))).add(Box::new(Text::plain("golden.rs")));
    tree.add(Box::new(Text::plain("Cargo.toml")));
    tree.add(Box::new(Text::plain("README.md")));
    console.print(&tree, None, None, None, false, "").unwrap();
}
