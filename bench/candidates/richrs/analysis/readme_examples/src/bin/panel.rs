use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();

    let panel = Panel::new("Hello from richrs!")
        .title("Greeting")
        .subtitle("A Rust port of Rich");

    console.write_segments(&panel.render(60))?;
    Ok(())
}
