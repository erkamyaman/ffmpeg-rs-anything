// A Rust port of FFmpeg n7.1 libavutil/murmur3.c, changed from the C original.
// Copyright (C) 2013 Reimar Döffinger
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of `murmur3.c` (128-bit x64 MurmurHash3 as libavutil streams it), exported as
//! `oneshot_murmur3`: the one-shot wrapper around `av_murmur3_init_seeded`, `av_murmur3_update`
//! and `av_murmur3_final` that the recordings call.

#![forbid(unsafe_code)]

use std::ffi::c_int;

const C1: u64 = 0x87c3_7b91_1142_53d5;
const C2: u64 = 0x4cf5_ad43_2745_937f;

fn get_k1(src: &[u8; 16]) -> u64 {
    let k = u64::from_le_bytes([
        src[0], src[1], src[2], src[3], src[4], src[5], src[6], src[7],
    ]);
    k.wrapping_mul(C1).rotate_left(31).wrapping_mul(C2)
}

fn get_k2(src: &[u8; 16]) -> u64 {
    let k = u64::from_le_bytes([
        src[8], src[9], src[10], src[11], src[12], src[13], src[14], src[15],
    ]);
    k.wrapping_mul(C2).rotate_left(33).wrapping_mul(C1)
}

fn update_h1(k: u64, h1: u64, h2: u64) -> u64 {
    (k ^ h1)
        .rotate_left(27)
        .wrapping_add(h2)
        .wrapping_mul(5)
        .wrapping_add(0x52dc_e729)
}

fn update_h2(k: u64, h1: u64, h2: u64) -> u64 {
    (k ^ h2)
        .rotate_left(31)
        .wrapping_add(h1)
        .wrapping_mul(5)
        .wrapping_add(0x3849_5ab5)
}

fn fmix(k: u64) -> u64 {
    let k = (k ^ (k >> 33)).wrapping_mul(0xff51_afd7_ed55_8ccd);
    let k = (k ^ (k >> 33)).wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    k ^ (k >> 33)
}

/// `AVMurMur3`.
struct Murmur3 {
    h1: u64,
    h2: u64,
    state: [u8; 16],
    state_pos: usize,
    len: u64,
}

impl Murmur3 {
    fn new(seed: u64) -> Self {
        Self {
            h1: seed,
            h2: seed,
            state: [0; 16],
            state_pos: 0,
            len: 0,
        }
    }

    fn mix(&mut self, block: &[u8; 16]) {
        let (k1, k2) = (get_k1(block), get_k2(block));
        self.h1 = update_h1(k1, self.h1, self.h2);
        self.h2 = update_h2(k2, self.h1, self.h2);
    }

    fn update(&mut self, mut src: &[u8]) {
        if src.is_empty() {
            return;
        }
        self.len = self.len.wrapping_add(src.len() as u64);
        if self.state_pos > 0 {
            let take = src.len().min(16 - self.state_pos);
            self.state[self.state_pos..self.state_pos + take].copy_from_slice(&src[..take]);
            self.state_pos += take;
            src = &src[take..];
            if take > 0 && src.is_empty() {
                return;
            }
            self.state_pos = 0;
            let state = self.state;
            self.mix(&state);
        }
        let (blocks, rest) = src.as_chunks::<16>();
        for block in blocks {
            self.mix(block);
        }
        if !rest.is_empty() {
            self.state[..rest.len()].copy_from_slice(rest);
            self.state_pos = rest.len();
        }
    }

    fn finish(mut self) -> [u8; 16] {
        self.state[self.state_pos..].fill(0);
        let mut h1 = self.h1 ^ get_k1(&self.state) ^ self.len;
        let mut h2 = self.h2 ^ get_k2(&self.state) ^ self.len;
        h1 = h1.wrapping_add(h2);
        h2 = h2.wrapping_add(h1);
        h1 = fmix(h1);
        h2 = fmix(h2);
        h1 = h1.wrapping_add(h2);
        h2 = h2.wrapping_add(h1);
        let mut out = [0u8; 16];
        out[..8].copy_from_slice(&h1.to_le_bytes());
        out[8..].copy_from_slice(&h2.to_le_bytes());
        out
    }
}

/// Port of `int oneshot_murmur3(uint8_t *dst, int dst_size, uint64_t seed, const uint8_t *src, size_t len, int chunk)` (`oneshot_murmur3.c:11`).
pub fn oneshot_murmur3(dst: &mut [u8], seed: u64, src: &[u8], chunk: c_int) -> c_int {
    if dst.len() < 16 {
        return -1;
    }
    let mut ctx = Murmur3::new(seed);
    match usize::try_from(chunk) {
        Ok(n) if n > 0 => src.chunks(n).for_each(|piece| ctx.update(piece)),
        _ => ctx.update(src),
    }
    dst[..16].copy_from_slice(&ctx.finish());
    16
}
