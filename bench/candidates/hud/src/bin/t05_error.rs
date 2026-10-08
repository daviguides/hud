use hud::{Console, ErrorReport};

fn main() {
    let report = ErrorReport::new("could not read config")
        .cause("No such file or directory (os error 2)")
        .cause("path: /etc/hud/config.toml")
        .hint("run again with --verbose for details");
    Console::stdout().print(&report);
}
