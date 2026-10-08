//! Deterministic randomized checks over strings built from awkward code points.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use hud_width::{Align, cell_width, cell_width_of_cluster, clusters, fold, pad, truncate};

const POOL: [char; 40] = [
    'a',
    'Z',
    ' ',
    '9',
    '\t',
    '\r',
    '\n',
    '\u{7f}',
    '你',
    '世',
    'あ',
    '한',
    '\u{1100}',
    '\u{1161}',
    '\u{11a8}',
    '\u{301}',
    '\u{200d}',
    '\u{200b}',
    '\u{fe0f}',
    '\u{20e3}',
    '😀',
    '👨',
    '👩',
    '👧',
    '🏻',
    '🇧',
    '🇷',
    '🇺',
    '❤',
    '☺',
    '\u{915}',
    '\u{94d}',
    '\u{937}',
    '\u{93f}',
    '\u{600}',
    '\u{ad}',
    '\u{3099}',
    '\u{1f3f3}',
    '1',
    '#',
];

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn text(&mut self) -> String {
        let len = (self.next() % 24) as usize;
        (0..len)
            .map(|_| POOL[(self.next() as usize) % POOL.len()])
            .collect()
    }
}

#[test]
fn clusters_partition_the_input() {
    let mut rng = Lcg(7);
    for _ in 0..20_000 {
        let text = rng.text();
        let parts: Vec<&str> = clusters(&text).collect();
        assert_eq!(parts.concat(), text);
        assert!(parts.iter().all(|p| !p.is_empty()));
    }
}

#[test]
fn fold_keeps_every_byte_and_respects_width() {
    let mut rng = Lcg(11);
    for _ in 0..20_000 {
        let text = rng.text();
        let width = 1 + (rng.next() % 12) as usize;
        let lines: Vec<&str> = fold(&text, width).collect();
        assert_eq!(lines.concat(), text);
        for line in &lines {
            assert!(
                cell_width(line) <= width || clusters(line).count() <= 1,
                "{line:?} width {width}"
            );
        }
        assert_eq!(truncate(&text, width), lines[0]);
    }
}

#[test]
fn empty_input_folds_to_one_empty_line() {
    assert_eq!(fold("", 5).collect::<Vec<_>>(), [""]);
    assert_eq!(truncate("", 5), "");
}

#[test]
fn width_is_additive_over_cluster_boundaries() {
    let mut rng = Lcg(23);
    for _ in 0..20_000 {
        let text = rng.text();
        let sum: usize = clusters(&text).map(cell_width_of_cluster).sum();
        assert_eq!(cell_width(&text), sum);
    }
}

#[test]
fn pad_fills_to_the_cell_width() {
    assert_eq!(pad("你", 5, Align::Left).to_string(), "你   ");
    assert_eq!(pad("🇧🇷", 4, Align::Right).to_string(), "  🇧🇷");
    assert_eq!(pad("abc", 2, Align::Center).to_string(), "abc");
}

#[test]
fn known_widths() {
    for (text, want) in [
        ("hello", 5),
        ("你好", 4),
        ("e\u{301}", 1),
        ("👨\u{200d}👩\u{200d}👧\u{200d}👦", 2),
        ("👍🏽", 2),
        ("🇧🇷", 2),
        ("1\u{fe0f}\u{20e3}", 2),
        ("❤\u{fe0f}", 2),
        ("\u{915}\u{94d}\u{937}", 2),
        ("a\tb", 2),
        ("\u{ad}", 1),
        ("한", 2),
        ("\u{1100}\u{1161}\u{11a8}", 2),
    ] {
        assert_eq!(cell_width(text), want, "{text:?}");
    }
}
