//! The 40 cells of the capability matrix, resolved with no terminal at all.
//!
//! Expectations come from `bench/cases/capability_matrix.json`, written from the
//! standards before any candidate was run.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use hud::{ColorSystem, EnvSnapshot, StreamInfo, resolve};

fn env_for(depth: &str, variable: &str) -> EnvSnapshot {
    let env = match depth {
        "truecolor" => EnvSnapshot::new()
            .term("xterm-256color")
            .colorterm("truecolor"),
        "256" => EnvSnapshot::new().term("xterm-256color"),
        "16" => EnvSnapshot::new().term("xterm"),
        "none" => EnvSnapshot::new().term("dumb"),
        other => panic!("unknown depth {other}"),
    };
    match variable {
        "unset" => env,
        "NO_COLOR" => env.no_color("1"),
        "FORCE_COLOR" => env.force_color("1"),
        "CLICOLOR" => env.clicolor("0"),
        "CLICOLOR_FORCE" => env.clicolor_force("1"),
        other => panic!("unknown variable {other}"),
    }
}

fn color_class(system: ColorSystem) -> &'static str {
    match system {
        ColorSystem::None => "none",
        ColorSystem::Standard => "standard",
        ColorSystem::EightBit => "256",
        ColorSystem::TrueColor => "truecolor",
        other => panic!("a color system the matrix does not know: {other:?}"),
    }
}

#[test]
fn all_forty_cells_match_the_written_expectation() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bench/cases/capability_matrix.json");
    let cells: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(cells.len(), 40);
    let mut failures = Vec::new();
    for cell in &cells {
        let env = env_for(
            cell["depth"].as_str().unwrap(),
            cell["env"].as_str().unwrap(),
        );
        let stream = if cell["stream"] == "tty" {
            StreamInfo::terminal(100, 24)
        } else {
            StreamInfo::new()
        };
        let caps = resolve(&env, stream);
        let expect = &cell["expect"];
        let got = (
            caps.emits_escapes(),
            caps.attributes,
            color_class(caps.color_system),
        );
        let want = (
            expect["escapes"].as_bool().unwrap(),
            expect["bold"].as_bool().unwrap(),
            expect["color_class"].as_str().unwrap(),
        );
        if got != want {
            failures.push(format!("{}: want {want:?}, got {got:?}", cell["id"]));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of 40 cells differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
