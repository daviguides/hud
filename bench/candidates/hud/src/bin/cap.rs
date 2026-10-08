// The one-line capability adapter: `x` in bold and #ff8800 through the default capabilities, no overrides.
// v0.1 has no Style type yet (v0.2), so the SGR sequences are written here from the resolved capabilities.
use hud::{ColorSystem, Stream, capabilities};

fn main() {
    let caps = capabilities(Stream::Stdout);
    let mut params: Vec<&str> = Vec::new();
    if caps.attributes {
        params.push("1");
    }
    match caps.color_system {
        ColorSystem::TrueColor => params.push("38;2;255;136;0"),
        ColorSystem::EightBit => params.push("38;5;208"),
        ColorSystem::Standard => params.push("33"),
        ColorSystem::None => {}
    }
    if params.is_empty() {
        println!("x");
    } else {
        println!("\x1b[{}mx\x1b[0m", params.join(";"));
    }
}
