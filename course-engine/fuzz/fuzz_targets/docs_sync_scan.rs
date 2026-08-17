//! Coverage-guided fuzz of the advertised-count scanner.
//!
//! Narrow boundary: `&[u8]` → `scan_document`. Strongest oracle available
//! without a reference impl: crash + span validity + identity rewrite.
//! Metamorphic: appending unmatched bytes must not drop existing hits
//! whose spans still sit inside the original prefix.
#![no_main]

use cdcp_cli::docs::{rewrite_identity, scan_document, scan_text};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 64_000 {
        return;
    }
    let hits = scan_document(data);
    let text = String::from_utf8_lossy(data);
    for h in &hits {
        assert!(h.start <= h.end, "inverted span");
        assert!(h.end <= text.len(), "span past end");
        assert!(
            text.is_char_boundary(h.start) && text.is_char_boundary(h.end),
            "span not on char boundary"
        );
        assert!(h.line >= 1, "line numbers are 1-based");
    }
    let identity = rewrite_identity(&text, &hits);
    assert_eq!(identity, text, "no-op rewrite must be identity");

    // MR: suffix of unmatched prose cannot drop prefix hits.
    if text.len() < 32_000 {
        let mut extended = text.into_owned();
        extended.push_str("\nLearn-15 Module 15 2026-08-15\n");
        let again = scan_text(&extended);
        assert!(
            again.len() >= hits.len(),
            "unmatched suffix dropped hits ({} -> {})",
            hits.len(),
            again.len()
        );
    }
});
