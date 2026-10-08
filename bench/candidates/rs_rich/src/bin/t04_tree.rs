use rich::{Console, Tree};

fn main() {
    let mut tree = Tree::new("hud/");
    let src = tree.add("src/");
    src.add("lib.rs");
    src.add("console.rs");
    src.add("width.rs");
    tree.add("tests/").add("golden.rs");
    tree.add("Cargo.toml");
    tree.add("README.md");
    Console::new().print(&tree);
}
