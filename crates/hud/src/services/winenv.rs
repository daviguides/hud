//! Windows consoles hand over fewer portable variables than a Unix terminal: `TERM` is usually
//! unset and `COLORTERM` almost never is. This folds what a Windows console reports into the
//! snapshot the portable resolver already understands, so precedence stays in one place.
//!
//! Policy: only virtual terminal sequences are emitted, never the legacy console API. A console
//! that cannot take them gets no styling at all.

use crate::model::{EnvSnapshot, WindowsFacts};

fn is_set(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|v| !v.is_empty())
}

fn is_value(value: &Option<String>, expected: &str) -> bool {
    value
        .as_deref()
        .is_some_and(|v| v.eq_ignore_ascii_case(expected))
}

/// Completes `env` with what the Windows console reports.
///
/// - `COLORTERM=truecolor` is added under Windows Terminal and the VS Code terminal.
/// - A terminal stream whose `TERM` is unset gets one: `xterm-256color` when virtual terminal
///   processing is on or the host translates ANSI itself (Windows Terminal, ConEmu, a named
///   `TERM_PROGRAM`), `xterm` under ANSICON alone, and `dumb` for a console that cannot take
///   escape sequences. Without build detection the depth on a plain console is the 256 colors
///   every virtual-terminal build supports.
/// - A `TERM` the user set (MSYS, Cygwin, WSL interop) is kept as it is.
/// - A stream that is not a terminal keeps `TERM` unset, so `FORCE_COLOR` still styles it with
///   the standard colors, as on Unix.
pub(crate) fn apply_windows(
    mut env: EnvSnapshot,
    facts: &WindowsFacts,
    is_tty: bool,
) -> EnvSnapshot {
    let conemu = is_value(&facts.conemu_ansi, "on");
    let named_host = is_set(&facts.term_program);

    if !is_set(&env.colorterm) && (facts.wt_session || is_value(&facts.term_program, "vscode")) {
        env.colorterm = Some("truecolor".into());
    }

    if is_tty && !is_set(&env.term) {
        let term = if facts.vt_enabled || facts.wt_session || conemu || named_host {
            "xterm-256color"
        } else if facts.ansicon {
            "xterm"
        } else {
            "dumb"
        };
        env.term = Some(term.into());
    }
    env
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ColorSystem, StreamInfo};
    use crate::services::resolve::resolve;

    #[derive(Default)]
    struct Host {
        vt: bool,
        wt: bool,
        conemu: Option<&'static str>,
        ansicon: bool,
        program: Option<&'static str>,
    }

    impl Host {
        fn facts(&self) -> WindowsFacts {
            WindowsFacts {
                vt_enabled: self.vt,
                wt_session: self.wt,
                conemu_ansi: self.conemu.map(Into::into),
                ansicon: self.ansicon,
                term_program: self.program.map(Into::into),
            }
        }
    }

    fn cell(host: &Host, pairs: &[(&str, &str)], is_tty: bool) -> (bool, bool, ColorSystem) {
        let mut env = EnvSnapshot::default();
        for (key, value) in pairs {
            let slot = match *key {
                "NO_COLOR" => &mut env.no_color,
                "FORCE_COLOR" => &mut env.force_color,
                "CLICOLOR" => &mut env.clicolor,
                "TERM" => &mut env.term,
                "COLORTERM" => &mut env.colorterm,
                _ => &mut env.columns,
            };
            *slot = Some((*value).to_string());
        }
        let env = apply_windows(env, &host.facts(), is_tty);
        let stream = StreamInfo {
            is_tty,
            size: is_tty.then_some((120, 30)),
        };
        let caps = resolve(&env, stream);
        (caps.emits_escapes(), caps.attributes, caps.color_system)
    }

    const NONE: (bool, bool, ColorSystem) = (false, false, ColorSystem::None);

    #[test]
    fn windows_terminal_is_truecolor() {
        let host = Host {
            vt: true,
            wt: true,
            ..Host::default()
        };
        assert_eq!(cell(&host, &[], true), (true, true, ColorSystem::TrueColor));
    }

    #[test]
    fn plain_console_with_vt_is_256_colors() {
        let host = Host {
            vt: true,
            ..Host::default()
        };
        assert_eq!(cell(&host, &[], true), (true, true, ColorSystem::EightBit));
    }

    #[test]
    fn legacy_console_without_vt_gets_no_styling_even_when_forced() {
        let host = Host::default();
        assert_eq!(cell(&host, &[], true), NONE);
        assert_eq!(cell(&host, &[("FORCE_COLOR", "1")], true), NONE);
    }

    #[test]
    fn ansicon_alone_gives_the_standard_colors() {
        let host = Host {
            ansicon: true,
            ..Host::default()
        };
        assert_eq!(cell(&host, &[], true), (true, true, ColorSystem::Standard));
    }

    #[test]
    fn conemu_that_translates_ansi_is_256_colors() {
        let host = Host {
            conemu: Some("ON"),
            ..Host::default()
        };
        assert_eq!(cell(&host, &[], true), (true, true, ColorSystem::EightBit));
        let off = Host {
            conemu: Some("OFF"),
            ..Host::default()
        };
        assert_eq!(cell(&off, &[], true), NONE);
    }

    #[test]
    fn vscode_terminal_is_truecolor() {
        let host = Host {
            vt: true,
            program: Some("vscode"),
            ..Host::default()
        };
        assert_eq!(cell(&host, &[], true), (true, true, ColorSystem::TrueColor));
    }

    #[test]
    fn another_named_host_is_256_colors() {
        let host = Host {
            program: Some("WezTerm"),
            ..Host::default()
        };
        assert_eq!(cell(&host, &[], true), (true, true, ColorSystem::EightBit));
    }

    #[test]
    fn a_term_the_user_set_is_kept() {
        let host = Host {
            vt: true,
            ..Host::default()
        };
        assert_eq!(
            cell(&host, &[("TERM", "xterm")], true),
            (true, true, ColorSystem::Standard)
        );
        assert_eq!(cell(&host, &[("TERM", "dumb")], true), NONE);
    }

    #[test]
    fn a_colorterm_the_user_set_is_kept() {
        let host = Host {
            vt: true,
            wt: true,
            ..Host::default()
        };
        assert_eq!(
            cell(&host, &[("COLORTERM", "24bit")], true),
            (true, true, ColorSystem::TrueColor)
        );
    }

    #[test]
    fn no_color_keeps_bold_and_removes_color() {
        let host = Host {
            vt: true,
            wt: true,
            ..Host::default()
        };
        assert_eq!(
            cell(&host, &[("NO_COLOR", "1")], true),
            (true, true, ColorSystem::None)
        );
    }

    #[test]
    fn a_pipe_is_not_styled_unless_forced() {
        let host = Host {
            wt: true,
            ..Host::default()
        };
        assert_eq!(cell(&host, &[], false), NONE);
        assert_eq!(
            cell(&host, &[("FORCE_COLOR", "1")], false),
            (true, true, ColorSystem::TrueColor)
        );
    }

    #[test]
    fn a_forced_pipe_without_a_host_gets_the_standard_colors() {
        let host = Host::default();
        assert_eq!(
            cell(&host, &[("FORCE_COLOR", "1")], false),
            (true, true, ColorSystem::Standard)
        );
    }

    #[test]
    fn clicolor_zero_turns_a_console_off() {
        let host = Host {
            vt: true,
            wt: true,
            ..Host::default()
        };
        assert_eq!(cell(&host, &[("CLICOLOR", "0")], true), NONE);
    }

    #[test]
    fn the_size_precedence_is_unchanged() {
        let host = Host {
            vt: true,
            ..Host::default()
        };
        let env = apply_windows(
            EnvSnapshot {
                columns: Some("90".into()),
                ..EnvSnapshot::default()
            },
            &host.facts(),
            true,
        );
        let caps = resolve(
            &env,
            StreamInfo {
                is_tty: true,
                size: Some((120, 30)),
            },
        );
        assert_eq!((caps.width, caps.height), (90, 30));
    }
}
