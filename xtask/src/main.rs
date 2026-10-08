//! Development tasks for hud: Unicode table generation and layer checks.
//!
//! Usage: `cargo xtask <gen-width|check-layers|sync-copies|check-copies>`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

type Res<T> = Result<T, String>;

const UCD_VERSION: &str = "17.0.0";
const BLOCK_SHIFT: u32 = 7;
const CODE_POINTS: usize = 0x11_0000;

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let result = match task.as_str() {
        "gen-width" => gen_width(),
        "check-layers" => check_layers(),
        "sync-copies" => sync_copies(),
        "check-copies" => check_copies(),
        _ => {
            Err("usage: cargo xtask <gen-width|check-layers|sync-copies|check-copies>".to_string())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("xtask: {message}");
            ExitCode::FAILURE
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Files a published crate carries from the repository root. `cargo package` ships what is inside
// the crate directory and a crate can only `include_str!` a file inside itself, so the README of
// `hud` and the two license files live in the crate directories as copies of the ones at the root
// of the repository; the copies are checked in CI and written by `sync-copies`.
// ---------------------------------------------------------------------------

const COPIES: &[(&str, &str)] = &[
    ("README.md", "crates/hud/README.md"),
    ("LICENSE-MIT", "crates/hud/LICENSE-MIT"),
    ("LICENSE-APACHE", "crates/hud/LICENSE-APACHE"),
    ("LICENSE-MIT", "crates/hud-width/LICENSE-MIT"),
    ("LICENSE-APACHE", "crates/hud-width/LICENSE-APACHE"),
];

fn sync_copies() -> Res<()> {
    for (source, copy) in COPIES {
        let (source, copy) = (root().join(source), root().join(copy));
        fs::copy(&source, &copy).map_err(|e| format!("{}: {e}", copy.display()))?;
        println!("{} written from {}", copy.display(), source.display());
    }
    Ok(())
}

fn check_copies() -> Res<()> {
    let mut stale = Vec::new();
    for (source, copy) in COPIES {
        let (source, copy) = (root().join(source), root().join(copy));
        let want = fs::read(&source).map_err(|e| format!("{}: {e}", source.display()))?;
        let have = fs::read(&copy).unwrap_or_default();
        if want != have {
            stale.push(copy.display().to_string());
        }
    }
    if stale.is_empty() {
        println!("check-copies: ok");
        Ok(())
    } else {
        Err(format!(
            "{} differ from the files at the root; run `cargo xtask sync-copies`",
            stale.join(", ")
        ))
    }
}

// ---------------------------------------------------------------------------
// SHA-256, used to verify the pinned data files (the data is untrusted input).
// ---------------------------------------------------------------------------

fn sha256(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for (slot, add) in h.iter_mut().zip(v) {
            *slot = slot.wrapping_add(add);
        }
    }
    h.iter().map(|word| format!("{word:08x}")).collect()
}

// ---------------------------------------------------------------------------
// UCD parsing
// ---------------------------------------------------------------------------

struct Entry {
    start: u32,
    end: u32,
    fields: Vec<String>,
}

fn data_dir() -> PathBuf {
    root().join("xtask/data").join(format!("ucd-{UCD_VERSION}"))
}

fn read_verified(name: &str, sums: &HashMap<String, String>) -> Res<String> {
    let path = data_dir().join(name);
    let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let want = sums
        .get(name)
        .ok_or_else(|| format!("{name}: missing from SHA256SUMS"))?;
    let got = sha256(&bytes);
    if &got != want {
        return Err(format!(
            "{name}: checksum mismatch (want {want}, got {got})"
        ));
    }
    String::from_utf8(bytes).map_err(|e| format!("{name}: {e}"))
}

fn parse_code_point(text: &str) -> Res<u32> {
    u32::from_str_radix(text.trim(), 16).map_err(|e| format!("bad code point {text:?}: {e}"))
}

fn parse_entries(text: &str) -> Res<Vec<Entry>> {
    let mut out = Vec::new();
    for line in text.lines() {
        let body = line.split('#').next().unwrap_or("").trim();
        if body.is_empty() {
            continue;
        }
        let mut parts = body.split(';').map(|p| p.trim().to_string());
        let range = parts.next().unwrap_or_default();
        let fields: Vec<String> = parts.collect();
        let (start, end) = match range.split_once("..") {
            Some((a, b)) => (parse_code_point(a)?, parse_code_point(b)?),
            None => {
                let first = range.split_whitespace().next().unwrap_or("");
                let cp = parse_code_point(first)?;
                (cp, cp)
            }
        };
        out.push(Entry { start, end, fields });
    }
    Ok(out)
}

fn check_header(text: &str, name: &str) -> Res<()> {
    let head: String = text.lines().take(3).collect::<Vec<_>>().join("\n");
    if head.contains(UCD_VERSION)
        || name == "emoji-data.txt"
        || name == "emoji-variation-sequences.txt"
    {
        Ok(())
    } else {
        Err(format!(
            "{name}: header does not name version {UCD_VERSION}"
        ))
    }
}

// ---------------------------------------------------------------------------
// Property model (mirrors crates/hud-width/src/props.rs)
// ---------------------------------------------------------------------------

const GCB_NAMES: [&str; 14] = [
    "Other",
    "CR",
    "LF",
    "Control",
    "Extend",
    "ZWJ",
    "Regional_Indicator",
    "Prepend",
    "SpacingMark",
    "L",
    "V",
    "T",
    "LV",
    "LVT",
];
const GCB_OTHER: u8 = 0;
const GCB_EXTEND: u8 = 4;
const GCB_PREPEND: u8 = 7;
const GCB_L: u8 = 9;
const GCB_V: u8 = 10;
const GCB_T: u8 = 11;

const INCB_LINKER: u8 = 1;
const INCB_CONSONANT: u8 = 2;
const INCB_EXTEND: u8 = 3;

const BIT_EXT_PICT: u16 = 1 << 6;
const BIT_VS16: u16 = 1 << 9;

fn gen_width() -> Res<()> {
    let sums_text = fs::read_to_string(data_dir().join("SHA256SUMS")).map_err(|e| e.to_string())?;
    let sums: HashMap<String, String> = sums_text
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let hash = it.next()?;
            let name = it.next()?;
            Some((name.to_string(), hash.to_string()))
        })
        .collect();

    let eaw_text = read_verified("EastAsianWidth.txt", &sums)?;
    let gcb_text = read_verified("GraphemeBreakProperty.txt", &sums)?;
    let gc_text = read_verified("DerivedGeneralCategory.txt", &sums)?;
    let core_text = read_verified("DerivedCoreProperties.txt", &sums)?;
    let emoji_text = read_verified("emoji-data.txt", &sums)?;
    let vs_text = read_verified("emoji-variation-sequences.txt", &sums)?;
    let license = read_verified("LICENSE-UNICODE.txt", &sums)?;
    for (name, text) in [
        ("EastAsianWidth.txt", &eaw_text),
        ("GraphemeBreakProperty.txt", &gcb_text),
        ("DerivedGeneralCategory.txt", &gc_text),
        ("DerivedCoreProperties.txt", &core_text),
        ("emoji-data.txt", &emoji_text),
        ("emoji-variation-sequences.txt", &vs_text),
    ] {
        check_header(text, name)?;
    }

    let mut gc = vec![*b"Cn"; CODE_POINTS];
    for e in parse_entries(&gc_text)? {
        let name = e.fields.first().ok_or("gc: missing value")?.as_bytes();
        let pair = [name[0], *name.get(1).unwrap_or(&b' ')];
        for cp in e.start..=e.end {
            gc[cp as usize] = pair;
        }
    }

    let mut eaw = vec![b'N'; CODE_POINTS];
    for e in parse_entries(&eaw_text)? {
        let value = e.fields.first().ok_or("eaw: missing value")?;
        let class = match value.as_str() {
            "W" | "F" => b'W',
            "A" => b'A',
            _ => b'N',
        };
        for cp in e.start..=e.end {
            eaw[cp as usize] = class;
        }
    }

    let mut gcb = vec![GCB_OTHER; CODE_POINTS];
    for e in parse_entries(&gcb_text)? {
        let value = e.fields.first().ok_or("gcb: missing value")?;
        let id = GCB_NAMES
            .iter()
            .position(|n| n == value)
            .ok_or_else(|| format!("unknown Grapheme_Cluster_Break value {value:?}"))?;
        for cp in e.start..=e.end {
            gcb[cp as usize] = id as u8;
        }
    }

    let mut incb = vec![0u8; CODE_POINTS];
    let mut default_ignorable = vec![false; CODE_POINTS];
    for e in parse_entries(&core_text)? {
        match e.fields.first().map(String::as_str) {
            Some("InCB") => {
                let id = match e.fields.get(1).map(String::as_str) {
                    Some("Linker") => INCB_LINKER,
                    Some("Consonant") => INCB_CONSONANT,
                    Some("Extend") => INCB_EXTEND,
                    other => return Err(format!("unknown InCB value {other:?}")),
                };
                for cp in e.start..=e.end {
                    incb[cp as usize] = id;
                }
            }
            Some("Default_Ignorable_Code_Point") => {
                for cp in e.start..=e.end {
                    default_ignorable[cp as usize] = true;
                }
            }
            _ => {}
        }
    }

    let mut ext_pict = vec![false; CODE_POINTS];
    for e in parse_entries(&emoji_text)? {
        if e.fields.first().map(String::as_str) == Some("Extended_Pictographic") {
            for cp in e.start..=e.end {
                ext_pict[cp as usize] = true;
            }
        }
    }

    let mut vs16 = vec![false; CODE_POINTS];
    for e in parse_entries(&vs_text)? {
        if e.fields.first().map(String::as_str) == Some("emoji style") {
            vs16[e.start as usize] = true;
        }
    }

    let mut packed = vec![0u16; CODE_POINTS];
    for cp in 0..CODE_POINTS {
        let width = cell_width_of(gc[cp], gcb[cp], eaw[cp], default_ignorable[cp], cp as u32);
        let mut v = u16::from(gcb[cp]);
        v |= u16::from(incb[cp]) << 4;
        if ext_pict[cp] {
            v |= BIT_EXT_PICT;
        }
        v |= u16::from(width) << 7;
        if vs16[cp] {
            v |= BIT_VS16;
        }
        packed[cp] = v;
    }

    write_tables(&packed, &license)
}

/// Terminal cell width of one code point, derived from the standards and the
/// pinned UCD files. Deviations from Python Rich are listed in DEVIATIONS.md.
fn cell_width_of(gc: [u8; 2], gcb: u8, eaw: u8, default_ignorable: bool, cp: u32) -> u8 {
    let category = &gc;
    let zero_category = matches!(category, b"Cc" | b"Mn" | b"Me" | b"Mc" | b"Zl" | b"Zp");
    if zero_category {
        return 0;
    }
    if category == b"Cf" && gcb != GCB_PREPEND && cp != 0xAD {
        return 0;
    }
    if gcb == GCB_V || gcb == GCB_T {
        return 0;
    }
    if gcb == GCB_EXTEND && category != b"Lm" && category != b"Lo" {
        return 0;
    }
    if default_ignorable && gcb != GCB_L && cp != 0xAD {
        return 0;
    }
    if eaw == b'W' { 2 } else { 1 }
}

fn write_tables(packed: &[u16], license: &str) -> Res<()> {
    let block = 1usize << BLOCK_SHIFT;
    let mut unique: HashMap<Vec<u16>, usize> = HashMap::new();
    let mut stage2: Vec<u16> = Vec::new();
    let mut stage1: Vec<usize> = Vec::new();
    for chunk in packed.chunks(block) {
        let index = match unique.get(chunk) {
            Some(&i) => i,
            None => {
                let i = unique.len();
                unique.insert(chunk.to_vec(), i);
                stage2.extend_from_slice(chunk);
                i
            }
        };
        stage1.push(index);
    }
    let max_index = stage1.iter().copied().max().unwrap_or(0);
    let stage1_type = if max_index < 256 { "u8" } else { "u16" };

    let mut out = String::new();
    out.push_str("// @generated by `cargo xtask gen-width`. DO NOT EDIT.\n//\n");
    out.push_str(&format!(
        "// Derived from the Unicode Character Database, version {UCD_VERSION}\n// (EastAsianWidth.txt, GraphemeBreakProperty.txt, DerivedGeneralCategory.txt,\n// DerivedCoreProperties.txt, emoji-data.txt, emoji-variation-sequences.txt).\n//\n"
    ));
    out.push_str(
        "// The Unicode data files are used under the Unicode License v3, reproduced below.\n//\n",
    );
    for line in license.lines() {
        if line.is_empty() {
            out.push_str("//\n");
        } else {
            out.push_str(&format!("// {line}\n"));
        }
    }
    out.push('\n');
    out.push_str(&format!(
        "pub(crate) const UNICODE_VERSION: (u8, u8, u8) = ({});\n\n",
        UCD_VERSION.replace('.', ", ")
    ));
    out.push_str(&format!(
        "pub(crate) const BLOCK_SHIFT: u32 = {BLOCK_SHIFT};\n\n"
    ));
    out.push_str("#[rustfmt::skip]\n");
    out.push_str(&format!(
        "pub(crate) static STAGE1: [{stage1_type}; {}] = [\n",
        stage1.len()
    ));
    push_numbers(&mut out, stage1.iter().map(|&v| v as u64));
    out.push_str("];\n\n#[rustfmt::skip]\n");
    out.push_str(&format!(
        "pub(crate) static STAGE2: [u16; {}] = [\n",
        stage2.len()
    ));
    push_numbers(&mut out, stage2.iter().map(|&v| u64::from(v)));
    out.push_str("];\n");

    let path = root().join("crates/hud-width/src/tables.rs");
    fs::write(&path, out).map_err(|e| format!("{}: {e}", path.display()))?;
    println!(
        "wrote {} ({} blocks, {} bytes of tables)",
        path.display(),
        unique.len(),
        stage2.len() * 2 + stage1.len() * if stage1_type == "u8" { 1 } else { 2 }
    );
    Ok(())
}

fn push_numbers(out: &mut String, values: impl Iterator<Item = u64>) {
    let mut line = String::from("   ");
    for v in values {
        let item = format!(" {v},");
        if line.len() + item.len() > 96 {
            out.push_str(&line);
            out.push('\n');
            line = String::from("   ");
        }
        line.push_str(&item);
    }
    if !line.trim().is_empty() {
        out.push_str(&line);
        out.push('\n');
    }
}

// ---------------------------------------------------------------------------
// Layer checks (architecture.md, "Layers inside hud")
// ---------------------------------------------------------------------------

fn check_layers() -> Res<()> {
    let src = root().join("crates/hud/src");
    // layer directory -> substrings its files MUST NOT contain
    let rules: [(&str, &[&str]); 4] = [
        (
            "model",
            &[
                "crate::services",
                "crate::integrations",
                "crate::widgets",
                "crate::console",
                "std::io",
                "std::env",
            ],
        ),
        (
            "services",
            &[
                "crate::integrations",
                "crate::widgets",
                "crate::console",
                "std::io",
                "std::env",
                "std::process",
                "OnceLock",
                "LazyLock",
                "thread_local",
                "Mutex",
                "static mut",
            ],
        ),
        (
            "integrations",
            &["crate::services", "crate::widgets", "crate::console"],
        ),
        ("widgets", &["crate::integrations", "crate::console"]),
    ];
    let mut violations = Vec::new();
    for (layer, banned) in rules {
        for file in rust_files(&src.join(layer))? {
            let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            for (number, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                for needle in banned {
                    if code.contains(needle) {
                        violations.push(format!(
                            "{}:{}: layer `{layer}` must not use `{needle}`",
                            file.display(),
                            number + 1
                        ));
                    }
                }
            }
        }
    }
    for file in rust_files(&src)? {
        let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        for (number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("Command::new") || code.contains("process::Command") {
                violations.push(format!(
                    "{}:{}: no child process is allowed in the library",
                    file.display(),
                    number + 1
                ));
            }
        }
    }
    if violations.is_empty() {
        println!("check-layers: ok");
        Ok(())
    } else {
        Err(violations.join("\n"))
    }
}

fn rust_files(dir: &Path) -> Res<Vec<PathBuf>> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(out);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(rust_files(&path)?);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}
