//! A tree built from nested nodes.
//!
//! ```bash
//! cargo run -p hud --example tree
//! ```

use hud::{Console, Tree};

fn main() {
    let tree = Tree::new("workspace/")
        .child(
            Tree::new("crates/")
                .child(Tree::new("core/").child("lib.rs").child("error.rs"))
                .child(Tree::new("cli/").child("main.rs")),
        )
        .child("Cargo.toml")
        .child("README.md");
    Console::stdout().print(&tree);
}
