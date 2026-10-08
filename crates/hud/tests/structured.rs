//! Structured output: the JSON document, the plain text and the choice between formats.

use std::path::PathBuf;

use hud::{
    Columns, Console, ErrorReport, Format, Group, Layout, Measure, Node, Padding, Panel, Progress,
    Renderable, Segment, Table, Tree,
};

fn console(width: u16) -> Console {
    Console::builder()
        .width(width)
        .color_system(hud::ColorSystem::TrueColor)
        .attributes(true)
        .build()
}

fn bench(path: &str) -> Option<String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bench");
    std::fs::read_to_string(root.join(path)).ok()
}

fn task_table() -> Table {
    Table::new()
        .column("Name")
        .column(hud::Column::new("Count").justify(hud::Justify::Right))
        .row(["api", "12"])
        .row(["cli", "7"])
}

#[test]
fn the_t09_table_is_the_pre_registered_document() {
    let Some(want) = bench("golden/tasks/t09-structured/pipe_json.bytes") else {
        return;
    };
    assert_eq!(console(100).render_as(&task_table(), Format::Json), want);
}

#[test]
fn the_shipped_schema_is_the_spec_schema() {
    let Some(spec) = bench("spec/structured-json.schema.json") else {
        return;
    };
    assert_eq!(include_str!("../schema/hud-1.json"), spec);
}

#[test]
fn json_does_not_depend_on_the_width_or_the_capabilities() {
    let panel = Panel::new(Group::new().push("[bold]one[/]").push(task_table())).title("T");
    let reference = console(80).render_as(&panel, Format::Json);
    for width in [10, 20, 40, 100, 200] {
        assert_eq!(console(width).render_as(&panel, Format::Json), reference);
    }
    let bare = Console::builder().width(33).plain().build();
    assert_eq!(bare.render_as(&panel, Format::Json), reference);
}

#[test]
fn plain_has_no_escape_even_when_color_and_attributes_are_on() {
    let tree = Tree::new("[red]root[/]").child(Tree::new("[bold]leaf[/]"));
    let out = console(40).render_as(&tree, Format::Plain);
    assert!(!out.contains('\x1b'));
    assert_eq!(out, "root\n└── leaf\n");
    let rich = console(40).render_as(&tree, Format::Rich);
    assert!(rich.contains('\x1b'));
}

#[test]
fn print_follows_the_format_of_the_console() {
    let json = Console::builder().format(Format::Json).width(40).build();
    assert_eq!(json.format(), Format::Json);
    assert_eq!(json.render_to_string("x"), Node::text("x").to_json());
    let plain = Console::builder().format(Format::Plain).width(40).build();
    assert_eq!(plain.render_to_string("[bold]x[/]"), "x\n");
}

#[test]
fn every_widget_describes_itself_and_the_document_reads_back() {
    let mut layout = Layout::new("body").name("main");
    layout.split_row([
        Layout::new("left").ratio(2),
        Layout::new(task_table()).size(30),
    ]);
    let progress = Progress::builder().disable(true).build();
    let done = progress.add_task("build", 10);
    done.advance(10);
    let hidden = progress.add_task("hidden", 4);
    hidden.set_visible(false);
    let nodes: Vec<Node> = vec![
        "[bold]text[/] with \\[brackets]".node(),
        task_table().node(),
        Panel::new("body").title("T").subtitle("S").node(),
        Tree::new("root")
            .child(Tree::new("a").child(Tree::new("b")))
            .node(),
        progress.node(),
        ErrorReport::new("boom")
            .cause("c1")
            .hint("try again")
            .node(),
        Columns::new(["one", "two"]).title("C").node(),
        layout.node(),
        Group::new().push("a").push("b").node(),
        Padding::new("x", (1, 2)).node(),
    ];
    for node in nodes {
        let json = node.to_json();
        let back = Node::from_json(&json).expect(&json);
        assert_eq!(back, node, "{json}");
        assert_eq!(back.to_json(), json);
    }
}

#[test]
fn markup_becomes_the_text_it_prints() {
    let json = console(40).render_as("[bold red]hi[/] \\[x] [oops", Format::Json);
    assert!(json.contains("\"text\": \"hi [x] [oops\""), "{json}");
}

#[test]
fn progress_state_is_in_the_document_without_timings() {
    let progress = Progress::builder().disable(true).build();
    let task = progress.add_task("job", 8);
    task.advance(3);
    let json = console(40).render_as(&progress, Format::Json);
    assert!(json.contains("\"completed\": 3"));
    assert!(json.contains("\"total\": 8"));
    assert!(json.contains("\"finished\": false"));
    assert!(!json.contains("elapsed") && !json.contains("speed"));
}

struct Custom;

impl Renderable for Custom {
    fn render(&self, _width: usize) -> Vec<Segment> {
        vec![Segment {
            text: "custom   \nlines  \n".into(),
            style: hud::Style::new(),
        }]
    }

    fn measure(&self, max_width: usize) -> Measure {
        Measure {
            min: 0,
            max: max_width,
        }
    }
}

#[test]
fn a_custom_renderable_without_a_node_is_its_plain_text() {
    assert_eq!(Custom.node(), Node::text("custom\nlines"));
}

#[test]
fn format_names_are_read_without_regard_to_case() {
    assert_eq!(Format::parse("Plain"), Some(Format::Plain));
    assert_eq!(Format::parse(""), None);
    assert_eq!(Format::parse("bogus"), None);
    assert_eq!(Format::Json.to_string(), "json");
    assert_eq!(Format::default(), Format::Rich);
}
