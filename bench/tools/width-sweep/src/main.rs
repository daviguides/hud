// width-sweep <rich.tsv>: compare per-code-point width of Rich, hud-width and the unicode-width crate
// over every Unicode scalar value, over assigned graphic characters, and print where they disagree.
use std::collections::BTreeMap;
use std::fs;

use unicode_width::UnicodeWidthChar;

#[derive(Default)]
struct Tally {
    universe: usize,
    hud_vs_rich: usize,
    uw_vs_rich: usize,
    hud_vs_uw: usize,
    by_category: BTreeMap<(&'static str, String), usize>,
    examples: BTreeMap<&'static str, Vec<String>>,
}

fn note(t: &mut Tally, which: &'static str, cat: &str, cp: u32) {
    *t.by_category.entry((which, cat.to_string())).or_default() += 1;
    let list = t.examples.entry(which).or_default();
    if list.len() < 12 {
        list.push(format!("U+{cp:04X}"));
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("rich.tsv");
    let data = fs::read_to_string(path).expect("read");
    let mut all = Tally::default();
    let mut graphic = Tally::default();
    for line in data.lines() {
        let mut parts = line.split('\t');
        let cp = u32::from_str_radix(parts.next().expect("cp"), 16).expect("hex");
        let rich: usize = parts.next().expect("w").parse().expect("num");
        let cat = parts.next().expect("cat");
        let Some(c) = char::from_u32(cp) else { continue };
        let hud = hud_width::char_width(c);
        let uw = c.width().unwrap_or(0);
        let is_graphic = matches!(cat.as_bytes()[0], b'L' | b'M' | b'N' | b'P' | b'S') || cat == "Zs";
        for (tally, include) in [(&mut all, true), (&mut graphic, is_graphic)] {
            if !include {
                continue;
            }
            tally.universe += 1;
            if hud != rich {
                tally.hud_vs_rich += 1;
                note(tally, "hud!=rich", cat, cp);
            }
            if uw != rich {
                tally.uw_vs_rich += 1;
                note(tally, "unicode-width!=rich", cat, cp);
            }
            if hud != uw {
                tally.hud_vs_uw += 1;
                note(tally, "hud!=unicode-width", cat, cp);
            }
        }
    }
    for (name, t) in [("all scalar values", &all), ("assigned graphic characters", &graphic)] {
        println!("== {name}: {} code points", t.universe);
        println!("   hud-width vs Rich:           {}", t.hud_vs_rich);
        println!("   unicode-width vs Rich:       {}", t.uw_vs_rich);
        println!("   hud-width vs unicode-width:  {}", t.hud_vs_uw);
        for which in ["hud!=rich", "unicode-width!=rich"] {
            let mut cats: Vec<_> = t
                .by_category
                .iter()
                .filter(|((w, _), _)| *w == which)
                .map(|((_, cat), n)| (n, cat.clone()))
                .collect();
            cats.sort_by(|a, b| b.cmp(a));
            let top: Vec<String> = cats.iter().take(8).map(|(n, c)| format!("{c}:{n}")).collect();
            println!("   {which} by category: {}", top.join(" "));
            if let Some(ex) = t.examples.get(which) {
                println!("   {which} examples: {}", ex.join(" "));
            }
        }
    }
}
