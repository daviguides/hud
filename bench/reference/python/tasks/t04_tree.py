from rich.console import Console
from rich.tree import Tree

tree = Tree("hud/")
src = tree.add("src/")
src.add("lib.rs")
src.add("console.rs")
src.add("width.rs")
tree.add("tests/").add("golden.rs")
tree.add("Cargo.toml")
tree.add("README.md")
Console().print(tree)
