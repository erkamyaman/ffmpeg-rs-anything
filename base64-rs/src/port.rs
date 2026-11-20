// A Rust port of FFmpeg n7.1 libavutil/base64.c, changed from the C original.
// Copyright (c) 2006 Ryan Martell
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of `base64.c`: safe Rust; `lib.rs` exports each function under its C name.

#![forbid(unsafe_code)]

const AVERROR_INVALIDDATA: i32 = -0x4144_4E49;

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `map2` of `base64.c`: the sextet of each character, `0xfe` for the characters that end the
/// input (NUL and `=`), `0xff` for invalid ones.
const MAP: [u8; 256] = {
    let mut map = [0xffu8; 256];
    let mut i = 0;
    while i < 64 {
        map[B64[i] as usize] = i as u8;
        i += 1;
    }
    map[0] = 0xfe;
    map[b'=' as usize] = 0xfe;
    map
};

/// Where decoding stopped: after this many sextets of the current group.
enum Stop {
    Group(usize),
}

/// Port of `int av_base64_decode(uint8_t *out, const char *in_str, int out_size)` (`base64.c:81`).
///
/// `out`: the output buffer (`NULL` and an empty buffer behave alike: only the syntax is checked).
pub fn av_base64_decode(out: &mut [u8], in_str: Option<&[u8]>) -> i32 {
    let input = in_str.unwrap_or_default();
    let sextet = |i: usize| MAP[usize::from(input.get(i).copied().unwrap_or(0))];
    let end = out.len();
    let mut dst = 0usize;
    let mut pos = 0usize;
    let mut bits = 0xffu32;
    let mut v = 0u32;
    let step = |k: usize, pos: usize, v: &mut u32, bits: &mut u32| -> Result<(), Stop> {
        *bits = u32::from(sextet(pos + k));
        if *bits & 0x80 != 0 {
            return Err(Stop::Group(k));
        }
        *v = if k > 0 { (*v << 6) + *bits } else { *bits };
        Ok(())
    };
    while end - dst > 3
        && let Some(&[c0, c1, c2, c3]) = input.get(pos..pos + 4)
    {
        let m = [
            MAP[usize::from(c0)],
            MAP[usize::from(c1)],
            MAP[usize::from(c2)],
            MAP[usize::from(c3)],
        ];
        if (m[0] | m[1] | m[2] | m[3]) & 0x80 != 0 {
            break;
        }
        let v = m.iter().fold(0u32, |v, &x| (v << 6) | u32::from(x));
        out[dst..dst + 4].copy_from_slice(&(v << 8).to_be_bytes());
        dst += 3;
        pos += 4;
    }
    let stop = 'decode: {
        while end - dst > 3 {
            for k in 0..4 {
                if let Err(s) = step(k, pos, &mut v, &mut bits) {
                    break 'decode s;
                }
            }
            let word = (v << 8).to_be_bytes();
            out[dst..dst + 4].copy_from_slice(&word);
            dst += 3;
            pos += 4;
        }
        if end > dst {
            for k in 0..4 {
                if let Err(s) = step(k, pos, &mut v, &mut bits) {
                    break 'decode s;
                }
            }
            out[dst] = (v >> 16) as u8;
            dst += 1;
            if end > dst {
                out[dst] = (v >> 8) as u8;
                dst += 1;
            }
            if end > dst {
                out[dst] = v as u8;
                dst += 1;
            }
            pos += 4;
        }
        loop {
            if let Err(s) = step(0, pos, &mut v, &mut bits) {
                break 'decode s;
            }
            pos += 1;
        }
    };
    let Stop::Group(k) = stop;
    if k == 3 {
        if end > dst {
            out[dst] = (v >> 10) as u8;
            dst += 1;
        }
        v <<= 2;
    }
    if k >= 2 && end > dst {
        out[dst] = (v >> 4) as u8;
        dst += 1;
    }
    if bits & 1 != 0 {
        AVERROR_INVALIDDATA
    } else {
        i32::try_from(dst).unwrap_or(i32::MAX)
    }
}

/// Port of `char *av_base64_encode(char *out, int out_size, const uint8_t *in, int in_size)` (`base64.c:144`).
///
/// Returns `Some(0)` (the start of `out`) once encoded, `None` when `out` is too small.
pub fn av_base64_encode(out: &mut [u8], r#in: &[u8]) -> Option<usize> {
    let in_size = r#in.len();
    if in_size >= (u32::MAX / 4) as usize || out.len() < in_size.div_ceil(3) * 4 + 1 {
        return None;
    }
    let groups = in_size.saturating_sub(3).div_ceil(3);
    for (o, i) in out[..4 * groups]
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(r#in.windows(4).step_by(3))
    {
        let bits = u32::from_be_bytes([i[0], i[1], i[2], i[3]]);
        *o = [26, 20, 14, 8].map(|shift| B64[((bits >> shift) & 0x3f) as usize]);
    }
    let mut dst = 4 * groups;
    let rest = &r#in[3 * groups..];
    let mut bits = 0u32;
    let mut shift = 0i32;
    for &b in rest {
        bits = (bits << 8) + u32::from(b);
        shift += 8;
    }
    while shift > 0 {
        out[dst] = B64[((bits << 6 >> shift) & 0x3f) as usize];
        dst += 1;
        shift -= 6;
    }
    while dst & 3 != 0 {
        out[dst] = b'=';
        dst += 1;
    }
    out[dst] = 0;
    Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fate_vectors() {
        for (data, encoded) in [
            ("", ""),
            ("1", "MQ=="),
            ("22", "MjI="),
            ("333", "MzMz"),
            ("4444", "NDQ0NA=="),
            ("55555", "NTU1NTU="),
            ("666666", "NjY2NjY2"),
            ("abc:def", "YWJjOmRlZg=="),
        ] {
            let mut out = vec![0u8; encoded.len() + 1];
            assert_eq!(av_base64_encode(&mut out, data.as_bytes()), Some(0));
            assert_eq!(&out[..encoded.len()], encoded.as_bytes());
            let mut back = vec![0u8; data.len()];
            let n = av_base64_decode(&mut back, Some(encoded.as_bytes()));
            assert_eq!(n, data.len() as i32);
            assert_eq!(back, data.as_bytes());
        }
        assert_eq!(av_base64_decode(&mut [], Some(b"!")), AVERROR_INVALIDDATA);
    }
}
