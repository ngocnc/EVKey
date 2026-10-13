//! Differential test: replay `tests/data/corpus.txt` through the Rust engine and compare each
//! output line with `tests/data/golden.txt`, produced by the original C++ engine
//! (see `tools/difftest/gen_golden.bat`).

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use evkey_engine::engine::{Config, Engine, V_REPLACE_MACRO};

fn data(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data").join(name)
}

fn set(cfg: &mut Config, name: &str, v: i32) {
    match name {
        "vInputType" => cfg.input_type = v,
        "vFreeMark" => cfg.free_mark = v,
        "vCodeTable" => cfg.code_table = v,
        "vCheckSpelling" => cfg.check_spelling = v,
        "vUseModernOrthography" => cfg.use_modern_orthography = v,
        "vQuickTelex" => cfg.quick_telex = v,
        "vRestoreIfWrongSpelling" => cfg.restore_if_wrong_spelling = v,
        "vUseMacro" => cfg.use_macro = v,
        "vAutoCapsMacro" => cfg.auto_caps_macro = v,
        "vUpperCaseFirstChar" => cfg.upper_case_first_char = v,
        "vAllowConsonantZFWJ" => cfg.allow_consonant_zfwj = v,
        "vQuickStartConsonant" => cfg.quick_start_consonant = v,
        "vQuickEndConsonant" => cfg.quick_end_consonant = v,
        // read by the C++ harness but not by the engine core
        "vLanguage" | "vUseMacroInEnglishMode" | "vTempOffSpelling" | "vFixRecommendBrowser" => {}
        other => panic!("unknown SET {other}"),
    }
}

fn format_state(e: &Engine) -> String {
    let h = &e.hook;
    let mut s = format!("{} {} {} {} |", h.code, h.backspace_count, h.new_char_count, h.ext_code);
    for d in &h.char_data[..(h.new_char_count as usize).min(h.char_data.len())] {
        write!(s, " {d:x}").unwrap();
    }
    s.push_str(" |");
    if h.code == V_REPLACE_MACRO {
        for m in &h.macro_data {
            write!(s, " {m:x}").unwrap();
        }
    }
    s
}

#[test]
fn matches_cpp_golden() {
    let corpus = fs::read_to_string(data("corpus.txt")).expect("run tools/difftest/gen_golden.bat");
    let golden = fs::read_to_string(data("golden.txt")).expect("run tools/difftest/gen_golden.bat");
    let mut golden = golden.lines();

    let mut e = Engine::new();
    let (mut section, mut events, mut failures) = (String::new(), 0usize, 0usize);
    let mut macro_texts: Vec<String> = Vec::new();

    for (lineno, line) in corpus.lines().enumerate() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("# section") {
            section = rest.trim().to_string();
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
        let nums = |r: &str| -> Vec<i32> { r.split_whitespace().map(|t| t.parse().unwrap()).collect() };
        match cmd {
            "SET" => {
                let (name, v) = rest.split_once(' ').unwrap();
                set(&mut e.cfg, name, v.trim().parse().unwrap());
            }
            "MACRO" => {
                let (text, content) = rest.split_once(' ').unwrap();
                let table = e.cfg.code_table as usize;
                e.macros.add(text, content, table);
                macro_texts.push(text.to_string());
            }
            "MACRO_CLEAR" => {
                let table = e.cfg.code_table as usize;
                for t in macro_texts.drain(..) {
                    e.macros.delete(&t, table);
                }
            }
            "INIT" => e.init(),
            "NEWSESSION" => e.start_new_session(),
            "TOGGLE_SPELL" => e.temp_off_spell_checking(),
            "SET_SPELL" => e.set_check_spelling(),
            "TEMP_OFF" => e.temp_off_engine(rest.trim() == "1"),
            "K" | "M" | "E" => {
                match cmd {
                    "K" => {
                        let n = nums(rest);
                        e.key_event(false, n[0] as u16, n[1] as u8, n[2] == 1);
                    }
                    "M" => e.key_event(true, 0, 0, false),
                    _ => {
                        let n = nums(rest);
                        e.english_mode(false, n[0] as u16, n[1] != 0, n[2] == 1);
                    }
                }
                events += 1;
                let want = golden.next().unwrap_or_else(|| panic!("golden ended at event {events}"));
                let got = format_state(&e);
                if got != want.trim_end() {
                    failures += 1;
                    if failures <= 10 {
                        eprintln!(
                            "MISMATCH section {section} corpus line {} (event {events}): `{line}`\n  want: {want}\n  got:  {got}",
                            lineno + 1
                        );
                    }
                }
            }
            other => panic!("unknown command {other}"),
        }
    }
    assert!(golden.next().is_none(), "golden has extra lines");
    assert_eq!(failures, 0, "{failures} of {events} events differ from the C++ engine");
}
