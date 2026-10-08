//! The 40 cells of the capability matrix, resolved with no terminal at all.
//!
//! Expectations come from `bench/cases/capability_matrix.json`, written from the
//! standards before any candidate was run.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;

use hud::{ColorSystem, EnvSnapshot, StreamInfo, resolve};

fn env_for(depth: &str, variable: &str) -> EnvSnapshot {
    let mut env = EnvSnapshot::default();
    match depth {
        "truecolor" => {
            env.term = Some("xterm-256color".into());
            env.colorterm = Some("truecolor".into());
        }
        "256" => env.term = Some("xterm-256color".into()),
        "16" => env.term = Some("xterm".into()),
        "none" => env.term = Some("dumb".into()),
        other => panic!("unknown depth {other}"),
    }
    match variable {
        "unset" => {}
        "NO_COLOR" => env.no_color = Some("1".into()),
        "FORCE_COLOR" => env.force_color = Some("1".into()),
        "CLICOLOR" => env.clicolor = Some("0".into()),
        "CLICOLOR_FORCE" => env.clicolor_force = Some("1".into()),
        other => panic!("unknown variable {other}"),
    }
    env
}

fn color_class(system: ColorSystem) -> &'static str {
    match system {
        ColorSystem::None => "none",
        ColorSystem::Standard => "standard",
        ColorSystem::EightBit => "256",
        ColorSystem::TrueColor => "truecolor",
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
        let stream = StreamInfo {
            is_tty: cell["stream"] == "tty",
            size: cell["stream"].eq("tty").then_some((100, 24)),
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
