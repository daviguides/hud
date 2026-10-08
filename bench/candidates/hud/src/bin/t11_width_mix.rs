use hud::{Console, Panel, Table};

fn main() {
    let table = Table::new()
        .column("Locale")
        .column("Greeting")
        .column("Note")
        .row(["ja", "こんにちは世界", "wide characters"])
        .row(["ko", "안녕하세요", "wide characters"])
        .row(["pt", "Olá, mundo", "one accent"])
        .row(["emoji", "🚀 launch 🎉", "two wide symbols"]);
    Console::stdout().print(&Panel::new(table).title("Release notes"));
}
