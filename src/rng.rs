//! A small xorshift generator, seeded so a maze is the same everywhere.

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        // spread nearby seeds apart (splitmix64), never zero
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Rng((z ^ (z >> 31)).max(1))
    }

    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform in 0..n.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

/// A stable 64-bit hash (FNV-1a) for turning names into seeds.
pub fn hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}
