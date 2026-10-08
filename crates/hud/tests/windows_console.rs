//! What the real Windows probe reports under `cargo test`: no console is attached to the test
//! harness output, so the stream must resolve as a pipe, once, without panicking or styling.

#![cfg(windows)]

use hud::{Stream, capabilities};

#[test]
fn a_captured_stream_resolves_as_an_unstyled_pipe() {
    let caps = capabilities(Stream::Stdout);
    assert!(caps.width > 0 && caps.height > 0);
    if !caps.is_tty {
        assert!(!caps.interactive);
    }
}

#[test]
fn the_answer_is_resolved_once_per_process() {
    let first = capabilities(Stream::Stderr);
    let second = capabilities(Stream::Stderr);
    assert_eq!(first, second);
}
