use rich::Console;

fn main() {
    let mut builder = Console::builder().highlight(false);
    if std::env::var_os("FORCE_COLOR").is_some() {
        builder = builder.force_terminal(true);
    }
    builder
        .build()
        .print_str("[bold red]error[/] [green]ok[/] plain");
}
