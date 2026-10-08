use rich_rust::prelude::*;

fn main() {
    let src = TreeNode::new("src/")
        .child(TreeNode::new("lib.rs"))
        .child(TreeNode::new("console.rs"))
        .child(TreeNode::new("width.rs"));
    let root = TreeNode::new("hud/")
        .child(src)
        .child(TreeNode::new("tests/").child(TreeNode::new("golden.rs")))
        .child(TreeNode::new("Cargo.toml"))
        .child(TreeNode::new("README.md"));
    Console::new().print_renderable(&Tree::new(root));
}
