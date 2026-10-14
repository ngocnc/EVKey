//! Per-key latency of `Session::on_key` (the work the hook does between a key arriving and the edit being known).
//! `cargo run -p evkey-core --release --example bench_keys`
use std::time::Instant;

use evkey_core::session::{KeyInput, Mods, Session};

/// Telex text with tones, marks, a correction (backspace) and punctuation; letters map to VK = uppercase ASCII.
const TEXT: &str = "tieengs vieejt laf ngoon nguwx cuar nguwowfi vieejt nam, chungs toi ddang thuwr nghieejm booj gox mowis. ";

fn vk(c: char) -> u16 {
    match c {
        ' ' => 0x20,
        ',' => 0xBC,
        '.' => 0xBE,
        c => c.to_ascii_uppercase() as u16,
    }
}

fn main() {
    let mut s = Session::new(None);
    let keys: Vec<u16> = TEXT.chars().map(vk).collect();
    let mut ns: Vec<u64> = Vec::with_capacity(keys.len() * 2000);
    for round in 0..2100 {
        for &k in &keys {
            for down in [true, false] {
                let t = Instant::now();
                let r = s.on_key(KeyInput { vk: k, down, mods: Mods::default(), caps_lock: false });
                let d = t.elapsed().as_nanos() as u64;
                std::hint::black_box(r);
                if round >= 100 && down {
                    ns.push(d);
                }
            }
        }
    }
    ns.sort_unstable();
    let p = |q: f64| ns[((ns.len() - 1) as f64 * q) as usize];
    let mean = ns.iter().sum::<u64>() / ns.len() as u64;
    println!("keys={} mean={mean}ns p50={}ns p99={}ns p99.9={}ns max={}ns", ns.len(), p(0.5), p(0.99), p(0.999), ns[ns.len() - 1]);
}
