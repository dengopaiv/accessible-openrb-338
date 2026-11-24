//! The template of a new song (task-288): four sample patterns in A1, the lead in the first
//! 303, the bass in the second, and one each in the 808 and the 909. Every other pattern stays
//! empty, and knobs and settings stay as a new song has them.
//!
//! The patterns are `assets/patterns/*.json` in the Edit > Copy Pattern format
//! ([`crate::pattern_json`]), embedded and parsed when a new song is made. This is a UI
//! template over the engine's empty song (`rbs::Song::empty`, the CLI and the corpus are not
//! changed); a song opened from a file does not get it.

use orb_engine::frame::Engine;
use orb_engine::song::Device;

use crate::pattern_json;

/// The four files, by row: the first 303, the second 303, the 808, the 909.
pub const FILES: [&str; 4] = [
    include_str!("../../../assets/patterns/303-lead.json"),
    include_str!("../../../assets/patterns/303-bass.json"),
    include_str!("../../../assets/patterns/808.json"),
    include_str!("../../../assets/patterns/909.json"),
];

/// The devices of the four rows.
const ROWS: [Device; 4] = [
    Device::Tb303_1,
    Device::Tb303_2,
    Device::Tr808,
    Device::Tr909,
];

/// The bytes of A1 of each row, as the engine holds a pattern (98 for a 303, 194 for a drum
/// machine).
pub fn a1() -> [Vec<u8>; 4] {
    std::array::from_fn(|r| {
        let p = pattern_json::parse(FILES[r]).expect("the template's patterns parse");
        assert_eq!(
            p.device,
            pattern_json::device_of_row(r),
            "the template's row {r}"
        );
        p.bytes
    })
}

/// Puts the template into A1 of each row of `e`, an engine of a new empty song.
pub fn apply(e: &mut Engine) {
    for (d, bytes) in ROWS.into_iter().zip(a1()) {
        e.pattern(d, 0, &bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The files are the ones the task gave: bytes pinned by SHA-256.
    #[test]
    fn files_are_verbatim() {
        use sha2::{Digest, Sha256};
        let want = [
            "43f5915b465e5ed5f9c8f5763f0e1a8f5a0ea442d755f7d3dd6dc00e95c42cb4",
            "5ad14e134348b9d1e37327990ca61bf7dffce8ea695269b540ae5b1c4d7a5ce8",
            "8cee68c8aea7443559a3f774f80a3114268644ed4180096e108e8756802bcd2f",
            "0c63e002b4043e81f3992e235e668de3d81fd3e47a25e371dd777a2b31c04fc6",
        ];
        for (f, w) in FILES.iter().zip(want) {
            let h: String = Sha256::digest(f.as_bytes())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            assert_eq!(h, w);
        }
    }

    #[test]
    fn each_file_parses_for_its_row() {
        let a = a1();
        for (r, bytes) in a.iter().enumerate() {
            let n = if r < 2 { 98 } else { 194 };
            assert_eq!(bytes.len(), n, "row {r}");
            assert_eq!(bytes[0], 0, "shuffle off");
            assert_eq!(bytes[1], 16, "length 16");
        }
    }
}
