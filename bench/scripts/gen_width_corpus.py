"""Generate the Unicode width corpus (cases/width_corpus.jsonl): 500 strings."""

import json
import random
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "cases" / "width_corpus.jsonl"

ASCII_WORDS = "alpha beta build deploy cache index parser render stream socket worker queue token".split()

HAN = "漢字日本語中文汉语你好世界東京北京上海大阪京都山川海空雲風"
HIRA = "ひらがなこんにちはありがとうさようなら"
KATA = "カタカナコンピュータインターネット"
HANGUL = "한국어안녕하세요감사합니다서울부산"
FULLWIDTH = "ＡＢＣＤＥ１２３４５！＠＃（）"
HALFKANA = "ｶﾀｶﾅﾊﾝｶｸｶﾅ"

EMOJI = ["😀", "🚀", "🎉", "🔥", "✨", "👍", "🙏", "🐍", "🦀", "🌍", "❤️", "☺️", "✔️", "❤", "☺", "✔",
         "⭐", "⚡", "☀️", "☂️", "⚠️", "⌛", "⏰", "🅰️", "🅱️", "🆗", "🈶", "🀄", "♻️", "♥️", "©️", "™️", "↔️", "▶️",
         "🧑", "👩", "👨", "🐻", "❄️", "🏳️", "🏴"]
ZWJ = [
    "👨‍👩‍👧‍👦", "👩‍👩‍👦", "👨‍👨‍👧", "👩‍💻", "👨‍🚀", "🧑‍🚀", "🧑‍🤝‍🧑", "🏳️‍🌈", "🏴‍☠️", "🐻‍❄️",
    "👁️‍🗨️", "👩🏽‍🚀", "👨🏿‍🍳", "🧑🏻‍🎄", "👩‍❤️‍👨", "👨‍❤️‍💋‍👨", "🏃‍♀️", "🏊‍♂️", "🤷‍♀️", "💁‍♂️",
    "🧔‍♂️", "🕵️‍♀️", "👩‍🦰", "👨‍🦳", "🧑‍🦽",
]
TONES = ["\U0001F3FB", "\U0001F3FC", "\U0001F3FD", "\U0001F3FE", "\U0001F3FF"]
TONE_BASES = ["👍", "👋", "🙌", "👏", "🤝", "🧑", "👩", "👨", "🙏", "💪"]
FLAGS = ["🇧🇷", "🇺🇸", "🇯🇵", "🇩🇪", "🇫🇷", "🇬🇧", "🇨🇳", "🇰🇷", "🇮🇳", "🇨🇦", "🇦🇺", "🇮🇹", "🇪🇸", "🇲🇽", "🇷🇺"]
KEYCAPS = ["1️⃣", "2️⃣", "3️⃣", "#️⃣", "*️⃣", "0️⃣", "9️⃣"]
COMBINING = [
    "é", "à́̂", "ñ", "ö", "ů", "ç", "i̇́",
    "क्षत्रिय", "नमस्ते", "สวัสดี", "ภาษาไทย", "שָׁלוֹם", "מַה", "مَرْحَبًا", "كِتَابٌ",
    "Việt Nam", "tiếng", "한", "긱",
    "Źàl̂g̃ō", "ក្ខ", "ကျွန်ုပ်", "తెలుగు", "ਪੰਜਾਬੀ",
]
ZERO_WIDTH = ["​", "‌", "‍", "﻿", "⁠", "­", "‎", "‏", "⁣"]
AMBIGUOUS = ["αβγδ", "ДЖЗ", "─│┌┐└┘", "±×÷", "★☆●○■□", "①②③⑩", "½¼¾", "Ⅳ", "°℃", "§¶", "≈≠≤≥",
             "←↑→↓", "ÀÉÎÕÜ", "ǎǐǒǔ", "ɑ", "ø", "·", "…"]
CONTROLS = ["\t", "\x07", "\x7f", "\x85", " ", " ", "\x00", "\x08", "\x0b", "\x0c", "\x9b"]


def pick(rng, pool, lo, hi):
    return "".join(rng.choice(pool) for _ in range(rng.randint(lo, hi)))


def join_some(rng, parts):
    sep = rng.choice(["", " ", "-", "/"])
    return sep.join(parts)


def build():
    rng = random.Random(201)
    items = []

    def add(cat, text):
        items.append({"category": cat, "text": text})

    for _ in range(30):
        add("ascii", join_some(rng, [rng.choice(ASCII_WORDS) for _ in range(rng.randint(1, 5))]))
    for pool in (HAN, HIRA, KATA, HANGUL, FULLWIDTH, HALFKANA):
        for _ in range(10):
            add("cjk", pick(rng, pool, 1, 10))
    for _ in range(10):
        add("cjk", pick(rng, HAN + HIRA + KATA + HANGUL, 3, 8) + " " + rng.choice(ASCII_WORDS))
    for e in EMOJI:
        add("emoji", e)
    for _ in range(20):
        add("emoji", "".join(rng.choice(EMOJI) for _ in range(rng.randint(2, 5))))
    for z in ZWJ:
        add("emoji_zwj", z)
    for _ in range(25):
        add("emoji_zwj", "".join(rng.choice(ZWJ) for _ in range(rng.randint(2, 4))))
    for b in TONE_BASES:
        for t in TONES[:4]:
            add("emoji_tone", b + t)
    for f in FLAGS:
        add("flags", f)
    for _ in range(15):
        add("flags", "".join(rng.choice(FLAGS) for _ in range(rng.randint(2, 4))))
    for k in KEYCAPS:
        add("keycaps", k)
    for _ in range(8):
        add("keycaps", "".join(rng.choice(KEYCAPS) for _ in range(rng.randint(2, 4))))
    for c in COMBINING:
        add("combining", c)
    for _ in range(26):
        add("combining", "".join(rng.choice(COMBINING) for _ in range(rng.randint(2, 3))))
    for _ in range(30):
        zw = rng.choice(ZERO_WIDTH)
        w = rng.choice(ASCII_WORDS)
        cut = rng.randint(1, len(w) - 1)
        add("zero_width", w[:cut] + zw * rng.randint(1, 3) + w[cut:])
    for a in AMBIGUOUS:
        add("ambiguous", a)
    for _ in range(22):
        add("ambiguous", join_some(rng, [rng.choice(AMBIGUOUS) for _ in range(rng.randint(2, 4))]))
    for c in CONTROLS:
        add("control", "a" + c + "b")
    for _ in range(19):
        add("control", rng.choice(ASCII_WORDS) + rng.choice(CONTROLS) + rng.choice(ASCII_WORDS))
    pools = [HAN, HIRA, KATA, HANGUL, EMOJI, ZWJ, FLAGS, COMBINING, AMBIGUOUS, ASCII_WORDS]
    while len(items) < 500:
        parts = []
        for _ in range(rng.randint(3, 6)):
            p = rng.choice(pools)
            parts.append(rng.choice(p) if isinstance(p, list) else pick(rng, p, 1, 4))
        add("mixed", join_some(rng, parts))
    items = items[:500]
    for i, it in enumerate(items):
        it["id"] = f"w-{i:04d}"
    return items


def main():
    items = build()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w") as f:
        for it in items:
            f.write(json.dumps(it, ensure_ascii=False, sort_keys=True) + "\n")
    cats = {}
    for it in items:
        cats[it["category"]] = cats.get(it["category"], 0) + 1
    print(len(items), "strings", cats)


if __name__ == "__main__":
    main()
