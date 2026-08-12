//! Crash-only fuzz: ChoiceLetter::parse must never panic on arbitrary UTF-8.
#![no_main]

use cdcp_core::ChoiceLetter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let s = String::from_utf8_lossy(data);
    let _ = ChoiceLetter::parse(&s);
    // Round-trip: known letters always succeed and match.
    for letter in ["A", "B", "C", "D", "a", "b", "c", "d"] {
        let parsed = ChoiceLetter::parse(letter).expect("known letter");
        assert_eq!(parsed.as_str(), letter.to_uppercase());
    }
});
