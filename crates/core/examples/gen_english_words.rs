//! Regenerates `crates/engine/data/english_words.txt`: common English words that Telex (with restore on) changes, kept
//! only when what Telex makes of them is (almost) never seen in Vietnamese text.
//!
//! Input: `<word> <count>` frequency lists (hermitdave/FrequencyWords, OpenSubtitles 2018, CC BY-SA 4.0).
//! `cargo run -p evkey-core --release --example gen_english_words -- en_50k.txt vi_50k.txt [--dump]`
//! Without `--dump` the list goes to stdout; `--dump` prints every candidate with both frequencies instead.

use std::collections::HashMap;

use evkey_core::typing::edit_for;
use evkey_engine::composer::Composer;
use evkey_engine::settings::Settings;

/// The Vietnamese syllable Telex makes must be rarer than this (per million words). The Vietnamese list comes from
/// subtitles, which under-count formal words (`thí` in `thí sinh`), so the bar is absolute and low, not a ratio.
const VI_MAX_PER_MILLION: f64 = 0.3;
/// Rarer English words are left out.
const MIN_EN_PER_MILLION: f64 = 2.0;

fn per_million(path: &str) -> HashMap<String, f64> {
    let text = std::fs::read_to_string(path).expect("frequency list");
    let pairs: Vec<(String, f64)> = text
        .lines()
        .filter_map(|l| {
            let (w, c) = l.split_once(' ')?;
            Some((w.to_lowercase(), c.trim().parse().ok()?))
        })
        .collect();
    let total: f64 = pairs.iter().map(|p| p.1).sum();
    let mut out = HashMap::new();
    for (w, c) in pairs {
        *out.entry(w).or_insert(0.0) += c / total * 1e6;
    }
    out
}

/// What Telex with default settings leaves in the field after typing `word` and a space.
fn telex(word: &str) -> String {
    let mut c = Composer::new(Settings::default().to_config(false));
    let mut field = String::new();
    for ch in word.chars().chain([' ']) {
        c.engine.key_event(false, ch.to_ascii_uppercase() as u16, 0, false);
        match edit_for(&c.engine.hook, c.engine.cfg.code_table) {
            None => field.push(ch),
            Some(edit) => {
                for _ in 0..edit.backspaces {
                    field.pop();
                }
                field.push_str(&edit.text);
                if edit.resend_key {
                    field.push(ch);
                }
            }
        }
    }
    field.trim_end().to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let en = per_million(&args[0]);
    let vi = per_million(&args[1]);
    let dump = args.iter().any(|a| a == "--dump");
    let mut found: Vec<(String, f64, String, f64)> = en
        .iter()
        .filter(|&(w, &f)| f >= MIN_EN_PER_MILLION && w.len() >= 2 && w.bytes().all(|b| b.is_ascii_lowercase()))
        .filter_map(|(w, &f)| {
            let out = telex(w);
            (out != *w).then(|| {
                let v = vi.get(&out.to_lowercase()).copied().unwrap_or(0.0);
                (w.clone(), f, out, v)
            })
        })
        .collect();
    if dump {
        found.sort_by(|a, b| b.1.total_cmp(&a.1));
        for (w, f, out, v) in &found {
            println!("{w}\t{f:.1}\t{out}\t{v:.2}\t{}", if *v < VI_MAX_PER_MILLION { "KEEP" } else { "" });
        }
        return;
    }
    let mut words: Vec<&str> = found.iter().filter(|c| c.3 < VI_MAX_PER_MILLION).map(|c| c.0.as_str()).collect();
    words.sort_unstable();
    for w in words {
        println!("{w}");
    }
}
