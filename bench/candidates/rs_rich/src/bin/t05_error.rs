use rich::{Console, Panel, Text};

fn main() {
    let mut body = Text::new("");
    body.append("could not read config", Some("bold".into()));
    body.append("\n\n", None);
    body.append("Caused by:", Some("dim".into()));
    body.append("\n    0: No such file or directory (os error 2)", None);
    body.append("\n    1: path: /etc/hud/config.toml", None);
    body.append("\n\n", None);
    body.append(
        "hint: run again with --verbose for details",
        Some("cyan".into()),
    );
    let panel = Panel::new(Box::new(body))
        .title("Error")
        .border_style("red");
    Console::new().print(&panel);
}
