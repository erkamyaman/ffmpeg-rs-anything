// A Rust port of FFmpeg n7.1 libavutil/sha.c, changed from the C original.
// Copyright (C) 2007 Michael Niedermayer
// Copyright (C) 2009 Konstantin Shishkov
// based on public domain SHA-1 code by Steve Reid and on BSD-licensed SHA-2 code by Aaron D. Gifford
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of `sha.c` (SHA-1, SHA-224, SHA-256), exported as `oneshot_sha`: the one-shot
//! wrapper around `av_sha_init`, `av_sha_update` and `av_sha_final` that the recordings call.

#![forbid(unsafe_code)]

use std::ffi::c_int;

const EINVAL: c_int = -22;

/// `K256` of `sha.c`.
const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn word(buffer: &[u8; 64], i: usize) -> u32 {
    u32::from_be_bytes([
        buffer[4 * i],
        buffer[4 * i + 1],
        buffer[4 * i + 2],
        buffer[4 * i + 3],
    ])
}

#[inline(always)]
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & (y ^ z)) ^ z
}

#[inline(always)]
fn parity(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

#[inline(always)]
fn maj(x: u32, y: u32, z: u32) -> u32 {
    ((x | y) & z) | (x & y)
}

fn sha1_transform(state: &mut [u32; 8], buffer: &[u8; 64]) {
    let mut block = [0u32; 16];
    let [mut a, mut b, mut c, mut d, mut e, ..] = *state;
    macro_rules! blk {
        ($i:expr) => {{
            block[$i & 15] = if $i < 16 {
                word(buffer, $i)
            } else {
                (block[($i - 3) & 15]
                    ^ block[($i - 8) & 15]
                    ^ block[($i - 14) & 15]
                    ^ block[$i & 15])
                    .rotate_left(1)
            };
            block[$i & 15]
        }};
    }
    macro_rules! r {
        ($v:ident, $w:ident, $x:ident, $y:ident, $z:ident, $i:expr, $f:ident, $k:expr) => {
            $z = $z
                .wrapping_add($f($w, $x, $y))
                .wrapping_add(blk!($i))
                .wrapping_add($k)
                .wrapping_add($v.rotate_left(5));
            $w = $w.rotate_left(30);
        };
    }
    macro_rules! five {
        ($i:expr, $f:ident, $k:expr) => {
            r!(a, b, c, d, e, $i, $f, $k);
            r!(e, a, b, c, d, $i + 1, $f, $k);
            r!(d, e, a, b, c, $i + 2, $f, $k);
            r!(c, d, e, a, b, $i + 3, $f, $k);
            r!(b, c, d, e, a, $i + 4, $f, $k);
        };
    }
    five!(0, ch, 0x5A82_7999);
    five!(5, ch, 0x5A82_7999);
    five!(10, ch, 0x5A82_7999);
    five!(15, ch, 0x5A82_7999);
    five!(20, parity, 0x6ED9_EBA1);
    five!(25, parity, 0x6ED9_EBA1);
    five!(30, parity, 0x6ED9_EBA1);
    five!(35, parity, 0x6ED9_EBA1);
    five!(40, maj, 0x8F1B_BCDC);
    five!(45, maj, 0x8F1B_BCDC);
    five!(50, maj, 0x8F1B_BCDC);
    five!(55, maj, 0x8F1B_BCDC);
    five!(60, parity, 0xCA62_C1D6);
    five!(65, parity, 0xCA62_C1D6);
    five!(70, parity, 0xCA62_C1D6);
    five!(75, parity, 0xCA62_C1D6);
    for (s, v) in state.iter_mut().zip([a, b, c, d, e]) {
        *s = s.wrapping_add(v);
    }
}

fn sha256_transform(state: &mut [u32; 8], buffer: &[u8; 64]) {
    let mut block = [0u32; 16];
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    macro_rules! blk {
        ($i:expr) => {{
            block[$i & 15] = if $i < 16 {
                word(buffer, $i)
            } else {
                let x = block[($i - 15) & 15];
                let y = block[($i - 2) & 15];
                block[$i & 15]
                    .wrapping_add(x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3))
                    .wrapping_add(y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10))
                    .wrapping_add(block[($i - 7) & 15])
            };
            block[$i & 15]
        }};
    }
    macro_rules! r {
        ($a:ident, $b:ident, $c:ident, $d:ident, $e:ident, $f:ident, $g:ident, $h:ident, $i:expr) => {
            let t1 = blk!($i)
                .wrapping_add($h)
                .wrapping_add($e.rotate_right(6) ^ $e.rotate_right(11) ^ $e.rotate_right(25))
                .wrapping_add(ch($e, $f, $g))
                .wrapping_add(K256[$i]);
            $d = $d.wrapping_add(t1);
            $h = t1
                .wrapping_add($a.rotate_right(2) ^ $a.rotate_right(13) ^ $a.rotate_right(22))
                .wrapping_add(maj($a, $b, $c));
        };
    }
    macro_rules! eight {
        ($i:expr) => {
            r!(a, b, c, d, e, f, g, h, $i);
            r!(h, a, b, c, d, e, f, g, $i + 1);
            r!(g, h, a, b, c, d, e, f, $i + 2);
            r!(f, g, h, a, b, c, d, e, $i + 3);
            r!(e, f, g, h, a, b, c, d, $i + 4);
            r!(d, e, f, g, h, a, b, c, $i + 5);
            r!(c, d, e, f, g, h, a, b, $i + 6);
            r!(b, c, d, e, f, g, h, a, $i + 7);
        };
    }
    eight!(0);
    eight!(8);
    eight!(16);
    eight!(24);
    eight!(32);
    eight!(40);
    eight!(48);
    eight!(56);
    for (s, v) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *s = s.wrapping_add(v);
    }
}

/// A block transform: `sha1_transform` or `sha256_transform`.
type Transform = fn(&mut [u32; 8], &[u8; 64]);

/// `AVSHA`: digest length in words, byte count, pending block, state and transform.
struct Sha {
    digest_len: usize,
    count: u64,
    buffer: [u8; 64],
    state: [u32; 8],
    transform: Transform,
}

impl Sha {
    fn new(bits: c_int) -> Option<Self> {
        let (state, transform): ([u32; 8], Transform) = match bits {
            160 => (
                [
                    0x6745_2301,
                    0xEFCD_AB89,
                    0x98BA_DCFE,
                    0x1032_5476,
                    0xC3D2_E1F0,
                    0,
                    0,
                    0,
                ],
                sha1_transform,
            ),
            224 => (
                [
                    0xC105_9ED8,
                    0x367C_D507,
                    0x3070_DD17,
                    0xF70E_5939,
                    0xFFC0_0B31,
                    0x6858_1511,
                    0x64F9_8FA7,
                    0xBEFA_4FA4,
                ],
                sha256_transform,
            ),
            256 => (
                [
                    0x6A09_E667,
                    0xBB67_AE85,
                    0x3C6E_F372,
                    0xA54F_F53A,
                    0x510E_527F,
                    0x9B05_688C,
                    0x1F83_D9AB,
                    0x5BE0_CD19,
                ],
                sha256_transform,
            ),
            _ => return None,
        };
        Some(Self {
            digest_len: (bits >> 5) as usize,
            count: 0,
            buffer: [0; 64],
            state,
            transform,
        })
    }

    fn update(&mut self, mut data: &[u8]) {
        let mut j = (self.count & 63) as usize;
        self.count = self.count.wrapping_add(data.len() as u64);
        if data.len() >= 64 - j {
            let (head, rest) = data.split_at(64 - j);
            self.buffer[j..].copy_from_slice(head);
            let buffer = self.buffer;
            (self.transform)(&mut self.state, &buffer);
            let (blocks, tail) = rest.as_chunks::<64>();
            for block in blocks {
                (self.transform)(&mut self.state, block);
            }
            data = tail;
            j = 0;
        }
        self.buffer[j..j + data.len()].copy_from_slice(data);
    }

    fn finish(mut self, digest: &mut [u8]) {
        let count = (self.count << 3).to_be_bytes();
        self.update(&[0x80]);
        while self.count & 63 != 56 {
            self.update(&[0]);
        }
        self.update(&count);
        for (i, out) in digest
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .take(self.digest_len)
            .enumerate()
        {
            out.copy_from_slice(&self.state[i].to_be_bytes());
        }
    }
}

/// Port of `int oneshot_sha(uint8_t *dst, int dst_size, int bits, const uint8_t *src, size_t len, int chunk)` (`oneshot_sha.c:11`):
/// the SHA-`bits` digest of `src` (160, 224 or 256), fed `chunk` bytes at a time.
pub fn oneshot_sha(dst: &mut [u8], bits: c_int, src: &[u8], chunk: c_int) -> c_int {
    if dst.len() < 32 {
        return -1;
    }
    let Some(mut ctx) = Sha::new(bits) else {
        return EINVAL;
    };
    match usize::try_from(chunk) {
        Ok(n) if n > 0 => src.chunks(n).for_each(|piece| ctx.update(piece)),
        _ => ctx.update(src),
    }
    ctx.finish(dst);
    bits / 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fips_vectors() {
        let mut out = [0u8; 32];
        assert_eq!(oneshot_sha(&mut out, 160, b"abc", 0), 20);
        assert_eq!(out[..4], [0xa9, 0x99, 0x3e, 0x36]);
        assert_eq!(oneshot_sha(&mut out, 256, b"abc", 1), 32);
        assert_eq!(out[..4], [0xba, 0x78, 0x16, 0xbf]);
        assert_eq!(oneshot_sha(&mut out, 224, b"abc", 0), 28);
        assert_eq!(out[..4], [0x23, 0x09, 0x7d, 0x22]);
        assert_eq!(oneshot_sha(&mut out, 512, b"abc", 0), EINVAL);
    }
}
