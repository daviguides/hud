use hud::{Console, Tree};

fn main() {
    let tree = Tree::new("hud/")
        .child(
            Tree::new("src/")
                .child("lib.rs")
                .child("console.rs")
                .child("width.rs"),
        )
        .child(Tree::new("tests/").child("golden.rs"))
        .child("Cargo.toml")
        .child("README.md");
    Console::stdout().print(&tree);
}
