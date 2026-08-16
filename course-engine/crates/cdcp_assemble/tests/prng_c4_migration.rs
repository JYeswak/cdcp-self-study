//! C4 migration receipt: v1 (`StdRng` / rand 0.8.7) emitted the same seed-42
//! stream as v2 (`ChaCha12Rng` / rand_chacha 0.3.1). Lives outside `src/lib.rs`
//! so the doc-facts needle `StdRng::seed_from_u64` in the shipped module stays
//! absent — the product path does not name `StdRng`.
//!
//! If this test goes RED, either rand 0.8.7's StdRng changed (the exact
//! instability C4 exists to escape) or the pin in `prng_stream_seed42_is_pinned`
//! was edited without updating this receipt.

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

const SEED42_FIRST_8_U64: [u64; 8] = [
    0x86cc_7763_2227_24a2,
    0x8af0_0a13_3fad_517d,
    0xa2ef_6071_de51_34d1,
    0x67e9_2d78_fd76_30b2,
    0x08ca_b0df_f811_9fea,
    0x6a3a_9ca3_9e0f_81a8,
    0xbcc7_d8e8_5908_78fb,
    0xd968_8d9b_2f8e_b737,
];

fn first8<R: Rng>(mut rng: R) -> [u64; 8] {
    std::array::from_fn(|_| rng.gen::<u64>())
}

#[test]
fn stdrng_rand_087_matches_named_chacha12_at_seed_42() {
    let via_std = first8(StdRng::seed_from_u64(42));
    let via_named = first8(ChaCha12Rng::seed_from_u64(42));
    assert_eq!(
        via_named, SEED42_FIRST_8_U64,
        "named ChaCha12 stream drifted from the C4 pin"
    );
    assert_eq!(
        via_std, via_named,
        "v1 StdRng/rand-0.8.7 stream must equal v2 ChaCha12Rng — item_ids must not move"
    );
}
