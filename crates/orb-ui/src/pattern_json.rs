//! A pattern as readable JSON, the text Edit > Copy Pattern puts on the system clipboard and
//! Edit > Paste Pattern reads back (task-287, docs/ui_spec.md section 16).
//!
//! The text carries the pattern's whole byte array as [`crate::link::Link::pattern`] holds it
//! (98 bytes for a 303, 194 for an 808 or a 909): the shuffle flag, the length and 16 steps.
//! Every byte has a field, so copy then paste gives back the same bytes. A value the readable
//! form names is written as its name (`true`, `"C#"`, `"flam"`); any other byte is written as
//! its number in the same field (an AC byte 1, a 303 tone 13, a 909 step 4, a length 0), and a
//! paste takes a number in place of any name. The one exception is the length: a copy writes
//! whatever byte the pattern holds, but a paste refuses a length outside 1 to 16.

use serde_json::{Map, Value};

/// The `"version"` this program writes and reads.
pub const VERSION: u64 = 1;

/// The notes of a 303 step's tone byte: 0 is C, 12 is the next C.
const NOTES: [&str; 13] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B", "C'",
];
/// The 808's step columns in the pattern's byte order (the accent first).
const KEYS_808: [&str; 12] = [
    "ac", "bd", "sd", "lt", "mt", "ht", "rs", "cp", "cb", "cy", "oh", "ch",
];
/// The 909's step columns in the pattern's byte order (the accent first).
const KEYS_909: [&str; 12] = [
    "ac", "bd", "sd", "lt", "mt", "ht", "rs", "cp", "ch", "oh", "cc", "rc",
];
/// The 303 step's fields, in the pattern's byte order after the tone.
const KEYS_303: [&str; 6] = ["note", "slide", "accent", "up", "down", "gate"];
/// The 909 step values 0..=3.
const LEVELS: [&str; 4] = ["off", "low", "high", "flam"];
/// The 303 gate values 0 and 1: the step byte is "rest", so 1 is a pause.
const GATES: [&str; 2] = ["note", "pause"];

/// A pattern read from clipboard text: its device ("303", "808" or "909") and its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pattern {
    pub device: &'static str,
    pub bytes: Vec<u8>,
}

/// Why a text is no pattern to paste.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// Not JSON, or JSON without `"openrb338": "pattern"`: the clipboard holds something else.
    NotPattern,
    /// It claims to be a pattern but a value is wrong; the text says which.
    Invalid(String),
}

/// The device of row `row` (0..=3).
pub fn device_of_row(row: usize) -> &'static str {
    ["303", "303", "808", "909"][row]
}

/// The bytes of one pattern of `device`: the shuffle flag, the length, 16 steps.
fn pattern_len(device: &str) -> usize {
    if device == "303" { 98 } else { 194 }
}

/// A byte as a JSON flag: `true` and `false` for 1 and 0, else its number.
fn bit(b: u8) -> String {
    match b {
        0 => "false".into(),
        1 => "true".into(),
        n => n.to_string(),
    }
}

/// A byte as one of `names` (quoted), else its number.
fn name(b: u8, names: &[&str]) -> String {
    match names.get(b as usize) {
        Some(n) => format!("\"{n}\""),
        None => b.to_string(),
    }
}

/// The JSON text of `bytes`, a whole pattern of row `row` (0..=3): one object, the 16 steps one
/// to a line.
pub fn to_json(row: usize, bytes: &[u8]) -> String {
    let device = device_of_row(row);
    assert_eq!(
        bytes.len(),
        pattern_len(device),
        "a whole pattern of row {row}"
    );
    let mut out = format!(
        "{{\n  \"openrb338\": \"pattern\",\n  \"version\": {VERSION},\n  \"device\": \
         \"{device}\",\n  \"length\": {},\n  \"shuffle\": {},\n  \"steps\": [\n",
        bytes[1],
        bit(bytes[0])
    );
    let slen = if row < 2 { 6 } else { 12 };
    for (i, s) in bytes[2..].chunks(slen).enumerate() {
        let fields: Vec<String> = match row {
            0 | 1 => vec![
                name(s[0], &NOTES),
                bit(s[1]),
                bit(s[2]),
                bit(s[3]),
                bit(s[4]),
                name(s[5], &GATES),
            ],
            2 => s.iter().map(|&b| bit(b)).collect(),
            _ => s
                .iter()
                .enumerate()
                // the accent column is on (2) or off (0); the drums name their four values
                .map(|(c, &b)| match (c, b) {
                    (0, 0) => "false".into(),
                    (0, 2) => "true".into(),
                    (0, _) => b.to_string(),
                    _ => name(b, &LEVELS),
                })
                .collect(),
        };
        let keys: &[&str] = match row {
            0 | 1 => &KEYS_303,
            2 => &KEYS_808,
            _ => &KEYS_909,
        };
        let line: Vec<String> = keys
            .iter()
            .zip(fields)
            .map(|(k, v)| format!("\"{k}\": {v}"))
            .collect();
        let comma = if i < 15 { "," } else { "" };
        out.push_str(&format!("    {{{}}}{comma}\n", line.join(", ")));
    }
    out.push_str("  ]\n}\n");
    out
}

/// `v` as an error message shows it: at most 40 characters, then an ellipsis, so a huge value
/// never lands in the status line.
fn shown(v: impl std::fmt::Display) -> String {
    let s = v.to_string();
    match s.char_indices().nth(40) {
        Some((i, _)) => format!("{}...", &s[..i]),
        None => s,
    }
}

/// Reads a byte: a whole number 0..=255.
fn byte(v: &Value, what: &str) -> Result<u8, String> {
    v.as_u64()
        .and_then(|n| u8::try_from(n).ok())
        .ok_or_else(|| format!("{what} must be a whole number 0 to 255, not {}", shown(v)))
}

/// Reads a flag: `true`, `false` or a byte.
fn flag(v: &Value, what: &str) -> Result<u8, String> {
    match v {
        Value::Bool(b) => Ok(*b as u8),
        _ => byte(v, what).map_err(|_| {
            format!(
                "{what} must be true, false or a number 0 to 255, not {}",
                shown(v)
            )
        }),
    }
}

/// Reads one of `names` (its index) or a byte.
fn named(v: &Value, what: &str, names: &[&str]) -> Result<u8, String> {
    match v {
        Value::String(s) => names
            .iter()
            .position(|n| n == s)
            .map(|i| i as u8)
            .ok_or_else(|| format!("{what} \"{}\" is none of {}", shown(s), quoted(names))),
        _ => byte(v, what).map_err(|_| {
            format!(
                "{what} must be one of {} or a number 0 to 255, not {}",
                quoted(names),
                shown(v)
            )
        }),
    }
}

fn quoted(names: &[&str]) -> String {
    names
        .iter()
        .map(|n| format!("\"{n}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The object `v`, whose keys are exactly `keys`.
fn object<'a>(v: &'a Value, keys: &[&str], what: &str) -> Result<&'a Map<String, Value>, String> {
    let m = v
        .as_object()
        .ok_or_else(|| format!("{what} must be an object"))?;
    if let Some(k) = m.keys().find(|k| !keys.contains(&k.as_str())) {
        return Err(format!("{what} has the unknown key \"{}\"", shown(k)));
    }
    if let Some(k) = keys.iter().find(|k| !m.contains_key(**k)) {
        return Err(format!("{what} has no \"{k}\""));
    }
    Ok(m)
}

/// The pattern in `text`: the clipboard's text, or a refusal. Only the identifying key decides
/// that a text is no pattern; after it every value is checked.
pub fn parse(text: &str) -> Result<Pattern, Refused> {
    let Ok(root) = serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) else {
        return Err(Refused::NotPattern);
    };
    if root.get("openrb338").and_then(Value::as_str) != Some("pattern") {
        return Err(Refused::NotPattern);
    }
    parse_pattern(&root).map_err(Refused::Invalid)
}

fn parse_pattern(root_v: &Value) -> Result<Pattern, String> {
    match root_v.get("version") {
        Some(v) if v.as_u64() == Some(VERSION) => {}
        Some(v) => {
            return Err(format!(
                "version {} is not supported (this program reads version {VERSION})",
                shown(v)
            ));
        }
        None => return Err("it has no \"version\"".into()),
    }
    let device = match root_v.get("device").and_then(Value::as_str) {
        Some("303") => "303",
        Some("808") => "808",
        Some("909") => "909",
        _ => return Err("\"device\" must be \"303\", \"808\" or \"909\"".into()),
    };
    let top = [
        "openrb338",
        "version",
        "device",
        "length",
        "shuffle",
        "steps",
    ];
    let root = object(root_v, &top, "the pattern")?;
    let length = byte(&root["length"], "\"length\"")?;
    if !(1..=16).contains(&length) {
        return Err(format!("\"length\" must be 1 to 16, not {length}"));
    }
    let mut bytes = vec![flag(&root["shuffle"], "\"shuffle\"")?, length];
    let steps = root["steps"].as_array().ok_or("\"steps\" must be a list")?;
    if steps.len() != 16 {
        return Err(format!("\"steps\" must hold 16 steps, not {}", steps.len()));
    }
    for (i, s) in steps.iter().enumerate() {
        let at = format!("step {}", i + 1);
        match device {
            "303" => step_303(s, &at, &mut bytes)?,
            "808" => {
                let m = object(s, &KEYS_808, &at)?;
                for k in KEYS_808 {
                    bytes.push(flag(&m[k], &format!("{at} \"{k}\""))?);
                }
            }
            _ => {
                let m = object(s, &KEYS_909, &at)?;
                for k in KEYS_909 {
                    let what = format!("{at} \"{k}\"");
                    bytes.push(match (k, &m[k]) {
                        // the accent: false is 0 and true is 2, the byte the panel writes
                        ("ac", Value::Bool(b)) => 2 * *b as u8,
                        ("ac", v) => byte(v, &what).map_err(|_| {
                            format!(
                                "{what} must be true, false or a number 0 to 255, not {}",
                                shown(v)
                            )
                        })?,
                        (_, v) => named(v, &what, &LEVELS)?,
                    });
                }
            }
        }
    }
    debug_assert_eq!(bytes.len(), pattern_len(device));
    Ok(Pattern { device, bytes })
}

/// A 303 step: the tone, then the five flag bytes, in the pattern's order.
fn step_303(s: &Value, at: &str, bytes: &mut Vec<u8>) -> Result<(), String> {
    let m = object(s, &KEYS_303, at)?;
    let key = |k: &str| format!("{at} \"{k}\"");
    bytes.push(named(&m["note"], &key("note"), &NOTES)?);
    for k in ["slide", "accent", "up", "down"] {
        bytes.push(flag(&m[k], &key(k))?);
    }
    bytes.push(named(&m["gate"], &key("gate"), &GATES)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A xorshift stream of bytes, the same on every run.
    fn bytes(n: usize, seed: u64, max: u64) -> Vec<u8> {
        let mut x = seed;
        (0..n)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                (x % max) as u8
            })
            .collect()
    }

    #[test]
    fn every_byte_of_every_device_round_trips() {
        for row in [0, 2, 3] {
            let len = pattern_len(device_of_row(row));
            // all 256 values at every position, then mostly small values
            for v in 0..=255u8 {
                for pos in 0..len {
                    let mut b = vec![0u8; len];
                    b[1] = 16;
                    b[pos] = v;
                    if pos == 1 && !(1..=16).contains(&v) {
                        // a length outside 1 to 16 is written but refused
                        let text = to_json(row, &b);
                        assert!(text.contains(&format!("\"length\": {v},")));
                        assert!(matches!(parse(&text), Err(Refused::Invalid(_))));
                        continue;
                    }
                    let p = parse(&to_json(row, &b)).unwrap();
                    assert_eq!(
                        (p.device, p.bytes),
                        (device_of_row(row), b),
                        "{row} {pos} {v}"
                    );
                }
            }
            for seed in 1..400u64 {
                let mut b = bytes(len, seed, if seed % 2 == 0 { 256 } else { 5 });
                b[1] = 1 + b[1] % 16;
                assert_eq!(parse(&to_json(row, &b)).unwrap().bytes, b);
            }
        }
    }

    #[test]
    fn names_are_written_where_the_form_has_them() {
        let mut b = vec![0u8; 98];
        b[1] = 16;
        b[2..8].copy_from_slice(&[12, 1, 1, 0, 1, 1]);
        let t = to_json(0, &b);
        assert!(t.contains("\"device\": \"303\""), "{t}");
        assert!(t.contains(
            "{\"note\": \"C'\", \"slide\": true, \"accent\": true, \"up\": false, \"down\": true, \
             \"gate\": \"pause\"}"
        ));
        let mut b = vec![0u8; 194];
        b[1] = 16;
        b[2..14].copy_from_slice(&[2, 1, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0]);
        let t = to_json(3, &b);
        assert!(t.contains(
            "{\"ac\": true, \"bd\": \"low\", \"sd\": \"high\", \"lt\": \"flam\", \"mt\": 4"
        ));
    }

    #[test]
    fn a_text_that_is_no_pattern_is_not_one() {
        for t in [
            "",
            "hello",
            "[1]",
            "{}",
            "{\"openrb338\": \"song\"}",
            "{\"a\": 1",
            "42",
        ] {
            assert_eq!(parse(t), Err(Refused::NotPattern), "{t}");
        }
    }

    fn invalid(text: &str) -> String {
        match parse(text) {
            Err(Refused::Invalid(r)) => r,
            other => panic!("{other:?}"),
        }
    }

    /// An 808 pattern text with `"length": 16` replaced by `field`.
    fn with_length(field: &str) -> String {
        let mut b = bytes(194, 3, 2);
        b[1] = 16;
        to_json(2, &b).replace("\"length\": 16", &format!("\"length\": {field}"))
    }

    #[test]
    fn numbers_that_are_no_byte_are_refused() {
        for n in [
            "-1",
            "2.5",
            "1e1",
            "16.0",
            "18446744073709551616",
            "256",
            "\"16\"",
            "null",
        ] {
            let r = invalid(&with_length(n));
            assert!(r.contains("\"length\""), "{n}: {r}");
        }
        // in a step too
        let good = with_length("16");
        let r = invalid(&good.replacen("\"bd\": ", "\"bd\": -3, \"x\": ", 1));
        assert!(r.contains("step 1"), "{r}");
        let neg = good.replacen("\"sd\": false", "\"sd\": -3", 1);
        let neg = neg.replacen("\"sd\": true", "\"sd\": -3", 1);
        let r = invalid(&neg);
        assert!(r.contains("\"sd\"") && r.contains("-3"), "{r}");
    }

    #[test]
    fn a_top_level_key_too_many_or_too_few_is_refused() {
        let good = with_length("16");
        let r = invalid(&good.replace("\"shuffle\"", "\"shuffled\""));
        assert!(r.contains("unknown key \"shuffled\""), "{r}");
        let r = invalid(&good.replacen("{\n", "{\n  \"extra\": 1,\n", 1));
        assert!(r.contains("unknown key \"extra\""), "{r}");
        let r = invalid(
            &good
                .replace("  \"shuffle\": false,\n", "")
                .replace("  \"shuffle\": true,\n", ""),
        );
        assert!(r.contains("has no \"shuffle\""), "{r}");
        let r = invalid(&good.replace("  \"length\": 16,\n", ""));
        assert!(r.contains("has no \"length\""), "{r}");
    }

    #[test]
    fn a_huge_offending_value_is_cut_in_the_message() {
        let long = "x".repeat(10_000);
        let good = with_length("16");
        for text in [
            good.replacen("\"bd\": true", &format!("\"bd\": \"{long}\""), 1)
                .replacen("\"bd\": false", &format!("\"bd\": \"{long}\""), 1),
            good.replace("\"length\"", &format!("\"{long}\"")),
            good.replace("\"version\": 1", &format!("\"version\": \"{long}\"")),
            good.replace("\"length\": 16", &format!("\"length\": \"{long}\"")),
        ] {
            let r = invalid(&text);
            assert!(
                r.len() < 200,
                "{} chars: {}",
                r.len(),
                &r[..80.min(r.len())]
            );
            assert!(r.contains("..."), "{r}");
        }
        // a short value is shown whole
        assert_eq!(shown("short"), "short");
        assert_eq!(shown("y".repeat(40)).len(), 40);
        assert_eq!(shown("y".repeat(41)), format!("{}...", "y".repeat(40)));
    }

    #[test]
    fn a_key_written_twice_takes_the_last() {
        let good = with_length("16");
        let twice = good.replacen("\"length\": 16,", "\"length\": 99, \"length\": 7,", 1);
        assert_eq!(parse(&twice).unwrap().bytes[1], 7);
    }

    #[test]
    fn a_wrong_value_is_refused_with_its_reason() {
        let mut b = bytes(194, 7, 2);
        b[1] = 16;
        let good = to_json(2, &b);
        assert!(parse(&good).is_ok());
        let r = invalid(&good.replace("\"version\": 1", "\"version\": 2"));
        assert!(r.contains("version 2 is not supported"), "{r}");
        let r = invalid(&good.replace("\"808\"", "\"707\""));
        assert!(r.contains("\"device\""), "{r}");
        let r = invalid(&good.replace("\"length\": ", "\"len\": "));
        assert!(r.contains("\"len\""), "{r}");
        let r = invalid(&good.replace("\"ac\": ", "\"ax\": "));
        assert!(r.contains("step 1"), "{r}");
        let r = invalid(
            &good
                .replacen("\"bd\": true", "\"bd\": \"yes\"", 1)
                .replacen("\"bd\": false", "\"bd\": \"yes\"", 1),
        );
        assert!(r.contains("\"bd\""), "{r}");
        let r = invalid(&good.replacen("\"cy\": true", "\"cy\": 256", 1).replacen(
            "\"cy\": false",
            "\"cy\": 256",
            1,
        ));
        assert!(r.contains("\"cy\""), "{r}");
        let r = invalid(concat!(
            "{\"openrb338\": \"pattern\", \"version\": 1, \"device\": \"808\", ",
            "\"length\": 16, \"shuffle\": false, \"steps\": []}"
        ));
        assert!(r.contains("16 steps"), "{r}");
    }
}
