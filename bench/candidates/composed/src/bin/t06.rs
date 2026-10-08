use owo_colors::OwoColorize;

fn main() {
    println!(
        "{} {} in {}",
        "Deploy".bold(),
        "ok".green(),
        "3.2s".italic().yellow()
    );
    println!(
        "{}{}{}{}",
        "outer ".bold(),
        "inner ".bold().italic(),
        "deep".bold().italic().underline(),
        " outer".bold()
    );
    println!(
        "{} {} in 5s",
        " FAIL ".bold().red().on_white(),
        "retry".strikethrough()
    );
}
