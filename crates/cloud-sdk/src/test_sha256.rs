use crate::retry::{DigestAlgorithm, FingerprintHasher};

pub(crate) struct Sha256;

impl FingerprintHasher for Sha256 {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        let Some(digest) = sha256(input) else {
            return Ok(0);
        };
        output.copy_from_slice(&digest);
        Ok(digest.len())
    }
}

pub(crate) fn sha256(input: &[u8]) -> Option<[u8; 32]> {
    const INITIAL: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const ROUND: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let bit_len = u64::try_from(input.len()).ok()?.checked_mul(8)?;
    let mut state = INITIAL;
    let (blocks, remaining) = input.as_chunks::<64>();
    for block in blocks {
        compress(&mut state, block, &ROUND)?;
    }
    let mut tail = [0_u8; 128];
    tail.get_mut(..remaining.len())?.copy_from_slice(remaining);
    *tail.get_mut(remaining.len())? = 0x80;
    let end = if remaining.len() < 56 { 64_usize } else { 128 };
    let length_start = end.checked_sub(8)?;
    tail.get_mut(length_start..end)?
        .copy_from_slice(&bit_len.to_be_bytes());
    let (tail_blocks, tail_remainder) = tail.get(..end)?.as_chunks::<64>();
    if !tail_remainder.is_empty() {
        return None;
    }
    for block in tail_blocks {
        compress(&mut state, block, &ROUND)?;
    }
    let mut output = [0_u8; 32];
    let (targets, output_remainder) = output.as_chunks_mut::<4>();
    if !output_remainder.is_empty() {
        return None;
    }
    for (word, target) in state.iter().zip(targets) {
        target.copy_from_slice(&word.to_be_bytes());
    }
    Some(output)
}

fn compress(state: &mut [u32; 8], block: &[u8], round: &[u32; 64]) -> Option<()> {
    let mut words = [0_u32; 64];
    let (encoded_words, remainder) = block.as_chunks::<4>();
    if !remainder.is_empty() {
        return None;
    }
    for (index, bytes) in encoded_words.iter().enumerate() {
        *words.get_mut(index)? = u32::from_be_bytes(*bytes);
    }
    for index in 16_usize..64 {
        let a = *words.get(index.checked_sub(15)?)?;
        let b = *words.get(index.checked_sub(2)?)?;
        let s0 = a.rotate_right(7) ^ a.rotate_right(18) ^ (a >> 3);
        let s1 = b.rotate_right(17) ^ b.rotate_right(19) ^ (b >> 10);
        let value = words
            .get(index.checked_sub(16)?)?
            .wrapping_add(s0)
            .wrapping_add(*words.get(index.checked_sub(7)?)?)
            .wrapping_add(s1);
        *words.get_mut(index)? = value;
    }
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for (word, constant) in words.iter().zip(round) {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choice = (e & f) ^ (!e & g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(choice)
            .wrapping_add(*constant)
            .wrapping_add(*word);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(majority);
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }
    for (target, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *target = target.wrapping_add(value);
    }
    Some(())
}
