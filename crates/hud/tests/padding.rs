//! `Padding`, the renderable wrapper, byte for byte against Rich 15.0.0 (`rich.padding.Padding`).
//! Expected outputs were produced by the pinned Rich with `Console(width=14)`; regenerate with the
//! script in `bench/` if the pinned version moves.

use hud::{ColorSystem, Console, Padding, Panel, Style};

fn console(color: bool) -> Console {
    let builder = Console::builder().width(14);
    if color {
        builder
            .color_system(ColorSystem::TrueColor)
            .attributes(true)
            .build()
    } else {
        builder.plain().build()
    }
}

#[test]
fn plain() {
    let padded = Padding::new("ok", (1, 2));
    assert_eq!(
        console(false).render_to_string(&padded),
        "              \n  ok          \n              \n"
    );
}

#[test]
fn four_sides() {
    let padded = Padding::new("ok", (1, 2, 0, 3));
    assert_eq!(
        console(false).render_to_string(&padded),
        "              \n   ok         \n"
    );
}

#[test]
fn single_number() {
    let padded = Padding::new("ok", 1);
    assert_eq!(
        console(false).render_to_string(&padded),
        "              \n ok           \n              \n"
    );
}

#[test]
fn wraps_inside() {
    let padded = Padding::new("one two three four five", (0, 2));
    assert_eq!(
        console(false).render_to_string(&padded),
        "  one two     \n  three four  \n  five        \n"
    );
}

#[test]
fn no_expand() {
    let padded = Padding::new("ok", (1, 2)).expand(false);
    assert_eq!(
        console(false).render_to_string(&padded),
        "      \n  ok  \n      \n"
    );
}

#[test]
fn styled() {
    let padded = Padding::new("ok", (1, 2)).style(Style::parse("on red").unwrap());
    assert_eq!(
        console(true).render_to_string(&padded),
        "\x1b[41m              \x1b[0m\n\x1b[41m  \x1b[0m\x1b[41mok\x1b[0m\x1b[41m        \x1b[0m\x1b[41m  \x1b[0m\n\x1b[41m              \x1b[0m\n"
    );
}

#[test]
fn styled_markup() {
    let padded =
        Padding::new("[bold]ok[/] hi", (0, 1)).style(Style::parse("white on blue").unwrap());
    assert_eq!(
        console(true).render_to_string(&padded),
        "\x1b[37;44m \x1b[0m\x1b[1;37;44mok\x1b[0m\x1b[37;44m hi\x1b[0m\x1b[37;44m       \x1b[0m\x1b[37;44m \x1b[0m\n"
    );
}

#[test]
fn styled_no_expand() {
    let padded = Padding::new("ok", (1, 2))
        .style(Style::parse("on red").unwrap())
        .expand(false);
    assert_eq!(
        console(true).render_to_string(&padded),
        "\x1b[41m      \x1b[0m\n\x1b[41m  \x1b[0m\x1b[41mok\x1b[0m\x1b[41m  \x1b[0m\n\x1b[41m      \x1b[0m\n"
    );
}

#[test]
fn inside_a_panel() {
    let panel = Panel::new(Padding::new("x", (0, 3)));
    assert_eq!(
        console(false).render_to_string(&panel),
        "╭────────────╮\n│    x       │\n╰────────────╯\n"
    );
}
