pub struct PRNG(u128);

impl PRNG {
    pub const fn new(seed: u128) -> Self {
        Self(seed)
    }

    pub const fn next(&mut self) -> u64 {
        // Constants from https://en.wikipedia.org/wiki/Linear_congruential_generator#Parameters_in_common_use
        // self.0 = (self.0 * 6364136223846793005 + 1442695040888963407) & ((1 << 64) - 1);

        self.0 *= 6364136223846793005;
        self.0 += 1442695040888963407;
        self.0 &= (1 << 64) - 1;
        return self.0 as u64;
    }
}