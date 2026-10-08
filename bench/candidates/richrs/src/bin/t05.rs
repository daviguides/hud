use richrs::console::ColorSystem;
use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();
    if !console.is_terminal() || std::env::var_os("NO_COLOR").is_some() {
        console.set_color_system(ColorSystem::None);
    }

    let body = Markup::parse(
        "[bold]could not read config[/]\n\n[dim]Caused by:[/]\n    0: No such file or directory \
         (os error 2)\n    1: path: /etc/hud/config.toml\n\n[cyan]hint: run again with --verbose \
         for details[/]",
    )?
    .to_text();
    let panel = Panel::new(body)
        .title("Error")
        .border_style(Style::parse("red")?);

    console.write_segments(&panel.render(console.width()))
}
