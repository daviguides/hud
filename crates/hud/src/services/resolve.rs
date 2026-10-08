//! Capability resolution: the one place where precedence is decided.
//!
//! The rules follow no-color.org, bixense.com/clicolors and the `FORCE_COLOR`
//! convention, as written in `bench/spec/capability.md`, not Rich's behavior.

use crate::model::{Capabilities, ColorSystem, EnvSnapshot, StreamInfo};

const DEFAULT_WIDTH: u16 = 80;
const DEFAULT_HEIGHT: u16 = 24;

fn non_empty(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|v| !v.is_empty())
}

fn is_on(value: &Option<String>) -> bool {
    non_empty(value).is_some_and(|v| v != "0")
}

fn positive(value: &Option<String>) -> Option<u16> {
    non_empty(value)
        .and_then(|v| v.trim().parse::<u16>().ok())
        .filter(|&n| n > 0)
}

fn depth(env: &EnvSnapshot) -> ColorSystem {
    let colorterm = env.colorterm.as_deref().unwrap_or("").to_ascii_lowercase();
    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        return ColorSystem::TrueColor;
    }
    let term = env.term.as_deref().unwrap_or("").to_ascii_lowercase();
    if term.contains("truecolor") || term.contains("24bit") || term.contains("direct") {
        ColorSystem::TrueColor
    } else if term.contains("256color") {
        ColorSystem::EightBit
    } else {
        ColorSystem::Standard
    }
}

/// Decides what a stream can show from the environment and what the OS reports.
///
/// Precedence, highest first:
///
/// 1. `TERM=dumb` has no color depth: nothing is styled, even when forced.
/// 2. `FORCE_COLOR` or `CLICOLOR_FORCE` (non-empty, not `0`) turn styling on for any stream.
/// 3. `CLICOLOR=0` turns styling off on a terminal; a stream that is not a terminal is not
///    styled unless forced; so is a terminal with no `TERM`.
/// 4. `NO_COLOR` (non-empty) removes color from a styled stream and keeps bold, italic and
///    underline.
/// 5. The color depth comes from `COLORTERM`, then `TERM`.
///
/// Size: `COLUMNS` and `LINES`, then the size the OS reports, then 80 by 24.
///
/// ```
/// use hud::{ColorSystem, EnvSnapshot, StreamInfo, resolve};
///
/// let env = EnvSnapshot {
///     term: Some("xterm-256color".into()),
///     ..EnvSnapshot::default()
/// };
/// let tty = StreamInfo { is_tty: true, size: Some((120, 40)) };
/// let caps = resolve(&env, tty);
/// assert_eq!(caps.color_system, ColorSystem::EightBit);
/// assert_eq!((caps.width, caps.height), (120, 40));
///
/// let piped = resolve(&env, StreamInfo::default());
/// assert!(!piped.emits_escapes());
/// ```
pub fn resolve(env: &EnvSnapshot, stream: StreamInfo) -> Capabilities {
    let forced = is_on(&env.force_color) || is_on(&env.clicolor_force);
    let dumb = env.term.as_deref() == Some("dumb");
    let term_known = non_empty(&env.term).is_some();
    let clicolor_off = env.clicolor.as_deref() == Some("0");
    let styled = !dumb && (forced || (stream.is_tty && !clicolor_off && term_known));

    let (color_system, attributes) = if !styled {
        (ColorSystem::None, false)
    } else if non_empty(&env.no_color).is_some() {
        (ColorSystem::None, true)
    } else {
        (depth(env), true)
    };

    let (os_width, os_height) = stream.size.unzip();
    let (os_width, os_height) = (os_width.filter(|&n| n > 0), os_height.filter(|&n| n > 0));
    Capabilities {
        color_system,
        attributes,
        is_tty: stream.is_tty,
        interactive: !dumb && (forced || stream.is_tty),
        width: positive(&env.columns).or(os_width).unwrap_or(DEFAULT_WIDTH),
        height: positive(&env.lines).or(os_height).unwrap_or(DEFAULT_HEIGHT),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvSnapshot {
        let mut e = EnvSnapshot::default();
        for (key, value) in pairs {
            let slot = match *key {
                "NO_COLOR" => &mut e.no_color,
                "FORCE_COLOR" => &mut e.force_color,
                "CLICOLOR" => &mut e.clicolor,
                "CLICOLOR_FORCE" => &mut e.clicolor_force,
                "COLORTERM" => &mut e.colorterm,
                "TERM" => &mut e.term,
                "COLUMNS" => &mut e.columns,
                _ => &mut e.lines,
            };
            *slot = Some((*value).to_string());
        }
        e
    }

    const TTY: StreamInfo = StreamInfo {
        is_tty: true,
        size: Some((100, 24)),
    };
    const PIPE: StreamInfo = StreamInfo {
        is_tty: false,
        size: None,
    };

    #[test]
    fn empty_values_count_as_unset() {
        let caps = resolve(&env(&[("TERM", "xterm"), ("NO_COLOR", "")]), TTY);
        assert_eq!(caps.color_system, ColorSystem::Standard);
        let caps = resolve(&env(&[("TERM", "xterm"), ("FORCE_COLOR", "")]), PIPE);
        assert!(!caps.emits_escapes());
    }

    #[test]
    fn force_color_zero_does_not_force() {
        let caps = resolve(&env(&[("TERM", "xterm"), ("FORCE_COLOR", "0")]), PIPE);
        assert!(!caps.emits_escapes());
    }

    #[test]
    fn no_color_wins_over_force_for_color_but_keeps_attributes() {
        let caps = resolve(
            &env(&[("TERM", "xterm"), ("NO_COLOR", "1"), ("FORCE_COLOR", "1")]),
            PIPE,
        );
        assert_eq!(caps.color_system, ColorSystem::None);
        assert!(caps.attributes);
    }

    #[test]
    fn dumb_terminal_is_never_styled() {
        let caps = resolve(&env(&[("TERM", "dumb"), ("FORCE_COLOR", "1")]), TTY);
        assert!(!caps.emits_escapes());
    }

    #[test]
    fn forced_without_term_falls_back_to_standard_colors() {
        let caps = resolve(&env(&[("FORCE_COLOR", "1")]), PIPE);
        assert_eq!(caps.color_system, ColorSystem::Standard);
        let caps = resolve(&EnvSnapshot::default(), TTY);
        assert!(!caps.emits_escapes());
    }

    #[test]
    fn colorterm_beats_term_for_depth() {
        let caps = resolve(&env(&[("TERM", "xterm"), ("COLORTERM", "24bit")]), TTY);
        assert_eq!(caps.color_system, ColorSystem::TrueColor);
    }

    #[test]
    fn a_size_of_zero_from_the_os_counts_as_unknown() {
        let tty = StreamInfo {
            is_tty: true,
            size: Some((63, 0)),
        };
        let caps = resolve(&env(&[("TERM", "xterm")]), tty);
        assert_eq!((caps.width, caps.height), (63, 24));
    }

    #[test]
    fn size_precedence_is_env_then_os_then_default() {
        let e = env(&[("TERM", "xterm"), ("COLUMNS", "132")]);
        assert_eq!(resolve(&e, TTY).width, 132);
        assert_eq!(resolve(&e, TTY).height, 24);
        assert_eq!(resolve(&env(&[("TERM", "xterm")]), TTY).width, 100);
        let caps = resolve(&env(&[("COLUMNS", "abc"), ("LINES", "0")]), PIPE);
        assert_eq!((caps.width, caps.height), (80, 24));
    }
}
