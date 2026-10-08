use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();

    let mut tree = Tree::new("project");
    tree.add(
        TreeNode::new("src")
            .with_child(TreeNode::new("main.rs"))
            .with_child(TreeNode::new("lib.rs")),
    );
    tree.add(TreeNode::new("Cargo.toml"));

    console.write_segments(&tree.render())?;
    Ok(())
}
