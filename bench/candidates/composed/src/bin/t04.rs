struct Node {
    name: &'static str,
    children: Vec<Node>,
}

fn leaf(name: &'static str) -> Node {
    Node {
        name,
        children: Vec::new(),
    }
}

fn print_children(node: &Node, prefix: &str) {
    for (i, child) in node.children.iter().enumerate() {
        let last = i + 1 == node.children.len();
        println!(
            "{prefix}{}{}",
            if last { "└── " } else { "├── " },
            child.name
        );
        print_children(
            child,
            &format!("{prefix}{}", if last { "    " } else { "│   " }),
        );
    }
}

fn main() {
    let root = Node {
        name: "hud/",
        children: vec![
            Node {
                name: "src/",
                children: vec![leaf("lib.rs"), leaf("console.rs"), leaf("width.rs")],
            },
            Node {
                name: "tests/",
                children: vec![leaf("golden.rs")],
            },
            leaf("Cargo.toml"),
            leaf("README.md"),
        ],
    };
    println!("{}", root.name);
    print_children(&root, "");
}
