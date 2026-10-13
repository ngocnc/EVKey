//! Deterministic protocol-script generator for the OpenKey differential harness.
//!
//! `cargo run -p evkey-engine --example gen_corpus > crates/engine/tests/data/corpus.txt`
//!
//! Output is the text protocol read by `tools/difftest/harness.cpp`. The script is
//! a sequence of independent sections. Each starts with `# section N`, `INIT`, a
//! block of `SET` lines, `TEMP_OFF 0`, `SET_SPELL`, and macro definitions. A section
//! then contains realistic Vietnamese typing (Telex or VNI), random key fuzzing, or
//! both. Stats go to stderr so stdout stays a clean script.
//!
//! Protocol extension beyond the shared contract: `MACRO_CLEAR` removes every macro
//! added by `MACRO`, which keeps sections independent of each other.

use std::fmt::Write as _;
use std::io::Write as _;

const SEED: u64 = 0x2545_F491_4F6C_DD1D;
const TARGET_EVENTS: usize = 400_000;

// Windows VK codes, as defined in platforms/win32.h.
const K_BACKSPACE: u16 = 8; // KEY_DELETE = VK_BACK
const K_TAB: u16 = 9;
const K_ENTER: u16 = 13;
const K_ESC: u16 = 27;
const K_SPACE: u16 = 32;
const K_LSHIFT: u16 = 160;
const K_RSHIFT: u16 = 161;
const PUNCT: [u16; 11] = [
    188, // KEY_COMMA
    190, // KEY_DOT
    191, // KEY_SLASH
    186, // KEY_SEMICOLON
    192, // KEY_QUOTE
    222, // KEY_BACK_SLASH
    189, // KEY_MINUS
    187, // KEY_EQUALS
    220, // KEY_BACKQUOTE
    219, // KEY_LEFT_BRACKET
    221, // KEY_RIGHT_BRACKET
];
// VK_LEFT, UP, RIGHT, DOWN, HOME, END, INSERT, DELETE, PRIOR, NEXT (all break codes).
const NAV_KEYS: [u16; 10] = [37, 38, 39, 40, 36, 35, 45, 46, 33, 34];

const ONSETS: &[&str] = &[
    "", "", "", "b", "c", "ch", "d", "g", "gh", "gi", "h", "k", "kh", "l", "m", "n", "ng", "ngh",
    "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x", "f", "j", "w", "z",
];
const NUCLEI: &[&str] = &[
    "a", "e", "i", "o", "u", "y", "ai", "ao", "au", "ay", "eo", "ia", "ie", "iu", "oa", "oe", "oi",
    "ua", "ue", "ui", "uo", "uy", "ye", "oai", "uoi", "uye",
];
const CODAS: &[&str] = &["", "", "", "", "c", "ch", "m", "n", "ng", "nh", "p", "t"];
const TRIGGERS: &[&str] = &["btw", "ms", "vn", "dc", "hn"];
// Macro definitions loaded into every section (each kept with 85% chance).
const MACROS: &[(&str, &str)] = &[
    ("btw", "by the way"),
    ("ms", "millisecond"),
    ("vn", "Việt Nam"),
    ("dc", "được"),
    ("hn", "Hà Nội"),
];

/// xorshift64* PRNG: deterministic, std-only.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// Uniform in `[lo, hi)`.
    fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + self.below(hi - lo)
    }

    /// True with probability `per_mille / 1000`.
    fn chance(&mut self, per_mille: u64) -> bool {
        self.next_u64() % 1000 < per_mille
    }

    fn pick<T: Copy>(&mut self, xs: &[T]) -> T {
        xs[self.below(xs.len())]
    }
}

#[derive(Clone, Copy)]
enum Case {
    Lower,
    Title,
    Upper,
}

#[derive(Default)]
struct Stats {
    sections: usize,
    k: usize,
    m: usize,
    e: usize,
    config: usize,
}

impl Stats {
    fn events(&self) -> usize {
        self.k + self.m + self.e
    }
}

struct Gen {
    out: String,
    rng: Rng,
    stats: Stats,
    fuzz_keys: Vec<u16>,
    temp_off: bool,
}

impl Gen {
    fn new() -> Self {
        let mut fuzz_keys = Vec::new();
        for c in b'A'..=b'Z' {
            fuzz_keys.extend([u16::from(c); 3]); // letters weighted x3
        }
        fuzz_keys.extend((b'0'..=b'9').map(u16::from));
        fuzz_keys.extend(PUNCT);
        fuzz_keys.extend([K_SPACE, K_SPACE, K_ENTER, K_BACKSPACE, K_BACKSPACE, K_TAB, K_ESC]);
        fuzz_keys.extend([K_LSHIFT, K_RSHIFT]);
        fuzz_keys.extend(NAV_KEYS);
        Gen { out: String::with_capacity(8 << 20), rng: Rng(SEED), stats: Stats::default(), fuzz_keys, temp_off: false }
    }

    fn line(&mut self, s: &str) {
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn key(&mut self, code: u16, caps: u8, ctrl: bool) {
        let _ = writeln!(self.out, "K {code} {caps} {}", u8::from(ctrl));
        self.stats.k += 1;
    }

    fn mouse(&mut self) {
        self.line("M");
        self.stats.m += 1;
    }

    fn english_event(&mut self) {
        let i = self.rng.below(self.fuzz_keys.len());
        let code = self.fuzz_keys[i];
        let caps = self.rng.below(3);
        let ctrl = u8::from(self.rng.chance(50));
        let _ = writeln!(self.out, "E {code} {caps} {ctrl}");
        self.stats.e += 1;
    }

    fn caps_roll(&mut self) -> u8 {
        match self.rng.below(100) {
            0..80 => 0,
            80..90 => 1,
            _ => 2,
        }
    }

    fn type_word(&mut self, word: &[char], case: Case) {
        for (i, ch) in word.iter().enumerate() {
            let is_letter = ch.is_ascii_alphabetic();
            let stray = self.rng.chance(20); // ~2% stray shift
            let caps = match case {
                Case::Upper if is_letter => 2,
                Case::Title if i == 0 && is_letter => 1,
                _ if stray => 1,
                _ => 0,
            };
            let ctrl = self.rng.chance(5);
            self.key(ch.to_ascii_uppercase() as u16, caps, ctrl);
            if self.rng.chance(15) {
                self.key(K_BACKSPACE, 0, false);
            }
        }
    }

    fn separator(&mut self) {
        match self.rng.below(100) {
            0..80 => self.key(K_SPACE, 0, false),
            80..86 => {
                let p = self.rng.pick(&PUNCT[..]);
                self.key(p, 0, false);
                self.key(K_SPACE, 0, false);
            }
            86..90 => self.key(K_ENTER, 0, false),
            90..94 => {
                let p = self.rng.pick(&PUNCT[..]);
                self.key(p, 0, false);
            }
            94..97 => {
                let n = self.rng.pick(&NAV_KEYS[..]);
                self.key(n, 0, false);
            }
            _ => self.key(K_TAB, 0, false),
        }
    }

    /// Occasional mouse clicks, English-mode events, and engine toggles.
    fn side_events(&mut self) {
        if self.rng.chance(3) {
            self.mouse();
        }
        if self.rng.chance(3) {
            self.english_event();
        }
        if self.rng.chance(2) {
            let cmd = self.rng.pick(&["TOGGLE_SPELL", "SET_SPELL", "NEWSESSION"][..]);
            self.line(cmd);
        }
        if self.rng.chance(2) {
            self.temp_off = !self.temp_off;
            let _ = writeln!(self.out, "TEMP_OFF {}", u8::from(self.temp_off));
        }
    }

    /// Realistic Vietnamese typing, plus macro triggers and junk words.
    fn typing(&mut self, vni: bool, budget: usize) {
        let start = self.stats.events();
        while self.stats.events() - start < budget {
            let r = self.rng.below(100);
            let word: Vec<char> = if r < 72 {
                build_syllable(&mut self.rng, vni)
            } else if r < 80 {
                let t = self.rng.pick(TRIGGERS);
                t.chars().collect()
            } else if r < 86 {
                let n = self.rng.range(1, 7);
                (0..n).map(|_| char::from(b'a' + self.rng.below(26) as u8)).collect()
            } else {
                build_syllable(&mut self.rng, vni)
            };
            let case = match self.rng.below(100) {
                0..75 => Case::Lower,
                75..92 => Case::Title,
                _ => Case::Upper,
            };
            self.type_word(&word, case);
            self.separator();
            self.side_events();
        }
    }

    /// Pure random key stream over the engine-relevant key set.
    fn fuzz(&mut self, budget: usize) {
        let start = self.stats.events();
        while self.stats.events() - start < budget {
            let r = self.rng.below(1000);
            if r < 8 {
                self.mouse();
            } else if r < 20 {
                self.english_event();
            } else if r < 28 {
                let code = self.rng.below(256) as u16;
                let caps = self.caps_roll();
                self.key(code, caps, false);
            } else {
                let i = self.rng.below(self.fuzz_keys.len());
                let code = self.fuzz_keys[i];
                let caps = self.caps_roll();
                let ctrl = self.rng.chance(30);
                self.key(code, caps, ctrl);
            }
        }
    }

    fn settings(&mut self) -> bool {
        let input_type = self.rng.below(4);
        let lines: Vec<(&str, usize)> = vec![
            ("vLanguage", usize::from(self.rng.chance(950))),
            ("vInputType", input_type),
            ("vFreeMark", usize::from(self.rng.chance(500))),
            ("vCodeTable", self.rng.below(5)),
            ("vCheckSpelling", usize::from(self.rng.chance(800))),
            ("vUseModernOrthography", usize::from(self.rng.chance(500))),
            ("vQuickTelex", usize::from(self.rng.chance(400))),
            ("vRestoreIfWrongSpelling", usize::from(self.rng.chance(750))),
            ("vFixRecommendBrowser", usize::from(self.rng.chance(500))),
            ("vUseMacro", usize::from(self.rng.chance(600))),
            ("vUseMacroInEnglishMode", usize::from(self.rng.chance(300))),
            ("vAutoCapsMacro", usize::from(self.rng.chance(400))),
            ("vUpperCaseFirstChar", usize::from(self.rng.chance(300))),
            ("vTempOffSpelling", usize::from(self.rng.chance(300))),
            ("vAllowConsonantZFWJ", usize::from(self.rng.chance(300))),
            ("vQuickStartConsonant", usize::from(self.rng.chance(300))),
            ("vQuickEndConsonant", usize::from(self.rng.chance(300))),
        ];
        for (name, v) in lines {
            let _ = writeln!(self.out, "SET {name} {v}");
            self.stats.config += 1;
        }
        input_type == 1
    }

    fn macros(&mut self) {
        self.line("MACRO_CLEAR");
        for (text, content) in MACROS {
            if self.rng.chance(850) {
                let _ = writeln!(self.out, "MACRO {text} {content}");
            }
        }
    }

    fn section(&mut self, index: usize, budget: usize) {
        self.stats.sections += 1;
        let _ = writeln!(self.out, "# section {index}");
        self.line("INIT");
        let vni = self.settings();
        self.temp_off = false;
        self.line("TEMP_OFF 0");
        self.line("SET_SPELL");
        self.macros();

        match self.rng.below(100) {
            0..55 => self.typing(vni, budget),
            55..80 => self.fuzz(budget),
            _ => {
                let end = self.stats.events() + budget;
                while self.stats.events() < end {
                    self.typing(vni, 50);
                    self.fuzz(15);
                }
            }
        }
    }
}

fn tone_key(tone: usize, vni: bool) -> char {
    if vni {
        char::from(b'0' + tone as u8)
    } else {
        ['s', 'f', 'r', 'x', 'j'][tone - 1]
    }
}

/// Diacritic keystroke suffix for a vowel letter (Telex or VNI), if it has one.
fn diacritic(rng: &mut Rng, ch: char, vni: bool) -> Option<&'static str> {
    let opts: &[&str] = match (ch, vni) {
        ('a', false) => &["a", "w"], // â, ă
        ('e', false) => &["e"],      // ê
        ('o', false) => &["o", "w"], // ô, ơ
        ('u', false) => &["w"],      // ư
        ('a', true) => &["6", "8"],  // â, ă
        ('e', true) => &["6"],       // ê
        ('o', true) => &["6", "7"],  // ô, ơ
        ('u', true) => &["7"],       // ư
        _ => return None,
    };
    Some(rng.pick(opts))
}

/// One syllable as raw keystrokes (lowercase letters/digits). Tone key placement varies:
/// right after the first vowel, after the nucleus, or at the end. Occasionally adds a
/// stray removal key or a random wrong key so restore paths get exercised.
fn build_syllable(rng: &mut Rng, vni: bool) -> Vec<char> {
    let mut s: Vec<char> = Vec::new();
    let onset = rng.pick(ONSETS);
    for ch in onset.chars() {
        if ch == 'd' && rng.chance(150) {
            // đ: Telex "dd", VNI "d9"
            s.push('d');
            s.push(if vni { '9' } else { 'd' });
        } else {
            s.push(ch);
        }
    }

    let tone = if rng.chance(600) { rng.range(1, 6) } else { 0 };
    let tone_char = (tone > 0).then(|| tone_key(tone, vni));
    let slot = match rng.below(100) {
        0..40 => 0,  // after first vowel
        40..65 => 1, // after nucleus
        _ => 2,      // end of syllable
    };

    let nucleus = rng.pick(NUCLEI);
    for (i, ch) in nucleus.chars().enumerate() {
        s.push(ch);
        if rng.chance(250)
            && let Some(suffix) = diacritic(rng, ch, vni)
        {
            s.extend(suffix.chars());
        }
        if i == 0 && slot == 0 {
            s.extend(tone_char);
        }
    }
    if slot == 1 {
        s.extend(tone_char);
    }

    s.extend(rng.pick(CODAS).chars());
    if slot == 2 {
        s.extend(tone_char);
    }

    if rng.chance(20) {
        s.push(if vni { '0' } else { 'z' }); // tone removal
    }
    if rng.chance(20) {
        let wrong = rng.pick(&['a', 'e', 'o', 'w', 's', 'f', 'd', 'j', '6', '8', '1'][..]);
        let pos = rng.below(s.len() + 1);
        s.insert(pos, wrong);
    }
    s
}

fn main() {
    let mut g = Gen::new();
    let mut index = 0;
    while g.stats.events() < TARGET_EVENTS {
        let remaining = TARGET_EVENTS - g.stats.events();
        let budget = g.rng.range(200, 1800).min(remaining);
        g.section(index, budget);
        index += 1;
    }

    let mut stdout = std::io::stdout().lock();
    stdout.write_all(g.out.as_bytes()).expect("failed to write corpus");
    stdout.flush().expect("failed to flush corpus");

    let s = &g.stats;
    eprintln!(
        "gen_corpus: sections={} config_lines={} K={} M={} E={} events={} bytes={}",
        s.sections,
        s.config,
        s.k,
        s.m,
        s.e,
        s.events(),
        g.out.len()
    );
}
