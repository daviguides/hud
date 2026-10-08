use richrs::console::ColorSystem;
use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();
    if !console.is_terminal() || std::env::var_os("NO_COLOR").is_some() {
        console.set_color_system(ColorSystem::None);
    }

    let mut tree = Tree::new("hud/");
    tree.add(
        TreeNode::new("src/")
            .with_child(TreeNode::new("lib.rs"))
            .with_child(TreeNode::new("console.rs"))
            .with_child(TreeNode::new("width.rs")),
    );
    tree.add(TreeNode::new("tests/").with_child(TreeNode::new("golden.rs")));
    tree.add(TreeNode::new("Cargo.toml"));
    tree.add(TreeNode::new("README.md"));

    console.write_segments(&tree.render())
}
