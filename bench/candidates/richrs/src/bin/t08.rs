use richrs::console::ColorSystem;
use richrs::prelude::*;

fn main() -> Result<()> {
    let mut console = Console::new();
    let forced = std::env::var_os("FORCE_COLOR").is_some();
    if !console.is_terminal() && !forced {
        console.set_color_system(ColorSystem::None);
    }
    if std::env::var_os("NO_COLOR").is_some() {
        console.print("[bold]error[/] ok plain")
    } else {
        console.print("[bold red]error[/] [green]ok[/] plain")
    }
}
