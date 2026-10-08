//! The public behavior of `Spinner` and `Status` that the Rich corpus cannot show: the document a
//! spinner makes (the same under every clock), the names, and the handles.

use hud::{Console, Format, Node, Renderable, Spinner, Status};

fn console() -> Console {
    Console::builder().width(40).plain().build()
}

#[test]
fn there_are_73_animations_and_every_name_builds() {
    let names: Vec<&str> = Spinner::names().collect();
    assert_eq!(names.len(), 73);
    assert!(
        names.windows(2).all(|pair| pair[0] < pair[1]),
        "alphabetical"
    );
    for name in names {
        assert_eq!(Spinner::try_new(name).unwrap().name(), name);
    }
}

#[test]
fn an_unknown_name_is_an_error_for_try_new_and_dots_for_new() {
    let error = Spinner::try_new("no such spinner").unwrap_err();
    assert_eq!(error.name(), "no such spinner");
    assert_eq!(error.to_string(), "no spinner called \"no such spinner\"");
    assert_eq!(Spinner::new("no such spinner").name(), "dots");
}

#[test]
fn the_json_document_is_the_message_and_does_not_depend_on_the_clock() {
    let slow = Spinner::new("dots")
        .text("[bold]loading[/] crates")
        .clock(|| 0.0);
    let fast = Spinner::new("moon")
        .text("[bold]loading[/] crates")
        .clock(|| 123_456.789);
    let console = console();
    let first = console.render_as(&slow, Format::Json);
    assert_eq!(first, console.render_as(&fast, Format::Json));
    assert_eq!(first, console.render_as(&slow, Format::Json));
    assert_eq!(slow.node(), Node::text("loading crates"));
    assert_eq!(fast.node(), Node::text("loading crates"));
}

#[test]
fn a_status_describes_itself_like_its_spinner_on_every_clock() {
    let one = Status::builder().text("syncing").clock(|| 0.0).build();
    let other = Status::builder().text("syncing").clock(|| 99.0).build();
    let console = console();
    assert_eq!(
        console.render_as(&one, Format::Json),
        console.render_as(&other, Format::Json)
    );
    assert_eq!(one.node(), Node::text("syncing"));
}

#[test]
fn clones_share_one_animation_and_one_message() {
    let spinner = Spinner::new("line").clock(|| 0.0);
    let copy = spinner.clone();
    copy.update().text("shared");
    assert_eq!(spinner.node(), Node::text("shared"));
}

#[test]
fn an_empty_message_or_an_empty_style_in_an_update_changes_nothing() {
    let spinner = Spinner::new("dots").text("kept").clock(|| 0.0);
    spinner
        .update()
        .text("")
        .style(hud::Style::new())
        .speed(0.0);
    assert_eq!(spinner.node(), Node::text("kept"));
    assert_eq!(console().render_to_plain(&spinner), "⠋ kept\n");
}

#[test]
fn a_console_makes_a_status_on_itself() {
    let status = console().status("[bold]working[/]");
    assert!(!status.is_started());
    assert_eq!(status.node(), Node::text("working"));
}
