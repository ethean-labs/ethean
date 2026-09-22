//! Keccak-f[1600] and the SHAKE128 extendable-output function (FIPS 202).
//!
//! Used by the XMSS PRF. Kept dependency-free so the whole signature path is
//! auditable inside this crate.

const ROUNDS: usize = 24;

const RC: [u64; ROUNDS] = [
    0x0000000000000001,
    0x0000000000008082,
    0x800000000000808a,
    0x8000000080008000,
    0x000000000000808b,
    0x0000000080000001,
    0x8000000080008081,
    0x8000000000008009,
    0x000000000000008a,
    0x0000000000000088,
    0x0000000080008009,
    0x000000008000000a,
    0x000000008000808b,
    0x800000000000008b,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x000000000000800a,
    0x800000008000000a,
    0x8000000080008081,
    0x8000000000008080,
    0x0000000080000001,
    0x8000000080008008,
];

const ROTATION: [u32; 25] = [
    0, 1, 62, 28, 27, 36, 44, 6, 55, 20, 3, 10, 43, 25, 39, 41, 45, 15, 21, 8, 18, 2, 61, 56, 14,
];

/// Keccak-f[1600] permutation on a 25-lane state (lane index `x + 5 * y`).
pub fn keccak_f1600(state: &mut [u64; 25]) {
    for &rc in RC.iter() {
        // Theta.
        let mut c = [0u64; 5];
        for x in 0..5 {
            c[x] = state[x] ^ state[x + 5] ^ state[x + 10] ^ state[x + 15] ^ state[x + 20];
        }
        for x in 0..5 {
            let d = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
            for y in 0..5 {
                state[x + 5 * y] ^= d;
            }
        }
        // Rho and pi.
        let mut b = [0u64; 25];
        for x in 0..5 {
            for y in 0..5 {
                let idx = x + 5 * y;
                let nx = y;
                let ny = (2 * x + 3 * y) % 5;
                b[nx + 5 * ny] = state[idx].rotate_left(ROTATION[idx]);
            }
        }
        // Chi.
        for y in 0..5 {
            for x in 0..5 {
                state[x + 5 * y] =
                    b[x + 5 * y] ^ (!b[(x + 1) % 5 + 5 * y] & b[(x + 2) % 5 + 5 * y]);
            }
        }
        // Iota.
        state[0] ^= rc;
    }
}

/// SHAKE128 XOF: 168-byte rate, domain suffix `0x1F`.
pub struct Shake128 {
    state: [u64; 25],
    buffer: [u8; Shake128::RATE],
    buffered: usize,
    squeezing: bool,
    squeeze_pos: usize,
}

impl Default for Shake128 {
    fn default() -> Self {
        Self::new()
    }
}

impl Shake128 {
    pub const RATE: usize = 168;

    pub fn new() -> Self {
        Self {
            state: [0u64; 25],
            buffer: [0u8; Self::RATE],
            buffered: 0,
            squeezing: false,
            squeeze_pos: 0,
        }
    }

    fn absorb_block(&mut self) {
        for (lane, chunk) in self.state.iter_mut().zip(self.buffer.as_chunks::<8>().0) {
            *lane ^= u64::from_le_bytes(*chunk);
        }
        keccak_f1600(&mut self.state);
        self.buffered = 0;
    }

    /// Absorb input bytes. Must not be called after squeezing started.
    pub fn update(&mut self, mut data: &[u8]) {
        assert!(!self.squeezing, "SHAKE128: update after squeeze");
        while !data.is_empty() {
            let take = (Self::RATE - self.buffered).min(data.len());
            self.buffer[self.buffered..self.buffered + take].copy_from_slice(&data[..take]);
            self.buffered += take;
            data = &data[take..];
            if self.buffered == Self::RATE {
                self.absorb_block();
            }
        }
    }

    fn finalize(&mut self) {
        self.buffer[self.buffered..].fill(0);
        self.buffer[self.buffered] ^= 0x1f;
        self.buffer[Self::RATE - 1] ^= 0x80;
        self.absorb_block();
        self.squeezing = true;
        self.squeeze_pos = 0;
    }

    fn current_block(&self, out: &mut [u8; Shake128::RATE]) {
        for (chunk, lane) in out.as_chunks_mut::<8>().0.iter_mut().zip(self.state.iter()) {
            chunk.copy_from_slice(&lane.to_le_bytes());
        }
    }

    /// Squeeze output bytes; may be called repeatedly.
    pub fn squeeze(&mut self, out: &mut [u8]) {
        if !self.squeezing {
            self.finalize();
        }
        let mut block = [0u8; Self::RATE];
        let mut written = 0;
        while written < out.len() {
            if self.squeeze_pos == Self::RATE {
                keccak_f1600(&mut self.state);
                self.squeeze_pos = 0;
            }
            self.current_block(&mut block);
            let take = (Self::RATE - self.squeeze_pos).min(out.len() - written);
            out[written..written + take]
                .copy_from_slice(&block[self.squeeze_pos..self.squeeze_pos + take]);
            self.squeeze_pos += take;
            written += take;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn shake128_empty_kat() {
        let mut x = Shake128::new();
        let mut out = [0u8; 32];
        x.squeeze(&mut out);
        assert_eq!(
            hex(&out),
            "7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26"
        );
    }

    #[test]
    fn shake128_abc_kat_and_incremental_squeeze() {
        let mut x = Shake128::new();
        x.update(b"abc");
        let mut out = [0u8; 200];
        x.squeeze(&mut out);
        assert_eq!(
            hex(&out[..32]),
            "5881092dd818bf5cf8a3ddb793fbcba74097d5c526a6d35f97b83351940f2cc8"
        );
        let mut y = Shake128::new();
        y.update(b"a");
        y.update(b"bc");
        let mut first = [0u8; 100];
        let mut second = [0u8; 100];
        y.squeeze(&mut first);
        y.squeeze(&mut second);
        assert_eq!(&out[..100], &first[..]);
        assert_eq!(&out[100..], &second[..]);
    }

    #[test]
    fn shake128_multi_block_input() {
        let data = vec![0x61u8; 500];
        let mut x = Shake128::new();
        x.update(&data);
        let mut out = [0u8; 16];
        x.squeeze(&mut out);
        let mut y = Shake128::new();
        for chunk in data.chunks(7) {
            y.update(chunk);
        }
        let mut out2 = [0u8; 16];
        y.squeeze(&mut out2);
        assert_eq!(out, out2);
    }
}
