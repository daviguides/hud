//! Conformance against `GraphemeBreakTest.txt` of the pinned Unicode version.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use hud_width::clusters;

const TEST_FILE: &str = include_str!("../../../xtask/data/ucd-17.0.0/GraphemeBreakTest.txt");

fn parse_case(line: &str) -> (String, Vec<String>) {
    let mut text = String::new();
    let mut expected = Vec::new();
    let mut current = String::new();
    for token in line.split_whitespace() {
        match token {
            "\u{f7}" => {
                if !current.is_empty() {
                    expected.push(core::mem::take(&mut current));
                }
            }
            "\u{d7}" => {}
            hex => {
                let c = u32::from_str_radix(hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .expect("valid code point");
                text.push(c);
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        expected.push(current);
    }
    (text, expected)
}

#[test]
fn matches_every_unicode_17_test_case() {
    let mut checked = 0;
    for line in TEST_FILE.lines() {
        let body = line.split('#').next().unwrap_or("").trim();
        if body.is_empty() {
            continue;
        }
        let (text, expected) = parse_case(body);
        let got: Vec<&str> = clusters(&text).collect();
        assert_eq!(got, expected, "case: {line}");
        checked += 1;
    }
    assert!(
        checked > 700,
        "expected the full test file, checked {checked}"
    );
}
