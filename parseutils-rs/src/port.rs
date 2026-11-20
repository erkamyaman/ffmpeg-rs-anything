// A Rust port of FFmpeg n7.1 libavutil/parseutils.c, changed from the C original.
// the FFmpeg developers
//
// This library is free software; you can redistribute it and/or modify it under the terms of
// the GNU Lesser General Public License as published by the Free Software Foundation; either
// version 2.1 of the License, or (at your option) any later version. It is distributed WITHOUT
// ANY WARRANTY; see ../LICENSE for the license text.

//! The Rust port of `av_parse_video_size` and `av_find_info_tag` from `parseutils.c`; `lib.rs`
//! exports each under its C name. `av_parse_time` stays in C: it reads the clock and the local
//! time zone, which a recording cannot pin down.

#![forbid(unsafe_code)]

use std::ffi::c_int;

const EINVAL: c_int = -22;

/// `video_size_abbrs` of `parseutils.c`.
const VIDEO_SIZE_ABBRS: &[(&[u8], c_int, c_int)] = &[
    (b"ntsc", 720, 480),
    (b"pal", 720, 576),
    (b"qntsc", 352, 240),
    (b"qpal", 352, 288),
    (b"sntsc", 640, 480),
    (b"spal", 768, 576),
    (b"film", 352, 240),
    (b"ntsc-film", 352, 240),
    (b"sqcif", 128, 96),
    (b"qcif", 176, 144),
    (b"cif", 352, 288),
    (b"4cif", 704, 576),
    (b"16cif", 1408, 1152),
    (b"qqvga", 160, 120),
    (b"qvga", 320, 240),
    (b"vga", 640, 480),
    (b"svga", 800, 600),
    (b"xga", 1024, 768),
    (b"uxga", 1600, 1200),
    (b"qxga", 2048, 1536),
    (b"sxga", 1280, 1024),
    (b"qsxga", 2560, 2048),
    (b"hsxga", 5120, 4096),
    (b"wvga", 852, 480),
    (b"wxga", 1366, 768),
    (b"wsxga", 1600, 1024),
    (b"wuxga", 1920, 1200),
    (b"woxga", 2560, 1600),
    (b"wqhd", 2560, 1440),
    (b"wqsxga", 3200, 2048),
    (b"wquxga", 3840, 2400),
    (b"whsxga", 6400, 4096),
    (b"whuxga", 7680, 4800),
    (b"cga", 320, 200),
    (b"ega", 640, 350),
    (b"hd480", 852, 480),
    (b"hd720", 1280, 720),
    (b"hd1080", 1920, 1080),
    (b"quadhd", 2560, 1440),
    (b"2k", 2048, 1080),
    (b"2kdci", 2048, 1080),
    (b"2kflat", 1998, 1080),
    (b"2kscope", 2048, 858),
    (b"4k", 4096, 2160),
    (b"4kdci", 4096, 2160),
    (b"4kflat", 3996, 2160),
    (b"4kscope", 4096, 1716),
    (b"nhd", 640, 360),
    (b"hqvga", 240, 160),
    (b"wqvga", 400, 240),
    (b"fwqvga", 432, 240),
    (b"hvga", 480, 320),
    (b"qhd", 960, 540),
    (b"uhd2160", 3840, 2160),
    (b"uhd4320", 7680, 4320),
];

fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// `strtol(s, &end, 10)` in the C locale: the value (clamped to `long` on overflow) and where
/// parsing stopped (0 when no digits were read).
fn strtol(s: &[u8]) -> (i64, usize) {
    let mut i = s
        .iter()
        .position(|&b| !matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .unwrap_or(s.len());
    let negative = at(s, i) == b'-';
    if matches!(at(s, i), b'+' | b'-') {
        i += 1;
    }
    let start = i;
    let limit: u64 = if negative { 1 << 63 } else { i64::MAX as u64 };
    let mut value: u64 = 0;
    let mut overflow = false;
    while at(s, i).is_ascii_digit() {
        let digit = u64::from(at(s, i) - b'0');
        match value.checked_mul(10).and_then(|v| v.checked_add(digit)) {
            Some(v) if v <= limit => value = v,
            _ => overflow = true,
        }
        i += 1;
    }
    if i == start {
        return (0, 0);
    }
    let value = match (overflow, negative) {
        (true, true) => i64::MIN,
        (true, false) => i64::MAX,
        (false, true) => (value as i64).wrapping_neg(),
        (false, false) => value as i64,
    };
    (value, i)
}

/// Port of `int av_parse_video_size(int *width_ptr, int *height_ptr, const char *str)` (`parseutils.c:150`).
pub fn av_parse_video_size(
    width_ptr: Option<&mut c_int>,
    height_ptr: Option<&mut c_int>,
    str: Option<&[u8]>,
) -> c_int {
    let s = str.unwrap_or_default();
    let (width, height) = match VIDEO_SIZE_ABBRS.iter().find(|(abbr, ..)| *abbr == s) {
        Some(&(_, w, h)) => (w, h),
        None => {
            let (w, mut p) = strtol(s);
            if at(s, p) != 0 {
                p += 1;
            }
            let (h, end) = strtol(&s[p.min(s.len())..]);
            let end = if end == 0 { p } else { p + end };
            if at(s, end) != 0 {
                return EINVAL;
            }
            (w as c_int, h as c_int)
        }
    };
    if width <= 0 || height <= 0 {
        return EINVAL;
    }
    if let Some(w) = width_ptr {
        *w = width;
    }
    if let Some(h) = height_ptr {
        *h = height;
    }
    0
}

/// Port of `int av_find_info_tag(char *arg, int arg_size, const char *tag1, const char *info)` (`parseutils.c:753`):
/// copies the value of `tag1` in a `?tag=value&...` string into `arg`.
pub fn av_find_info_tag(arg: &mut [u8], tag1: Option<&[u8]>, info: Option<&[u8]>) -> c_int {
    let (tag1, info) = (tag1.unwrap_or_default(), info.unwrap_or_default());
    let mut p = usize::from(at(info, 0) == b'?');
    loop {
        let start = p;
        while !matches!(at(info, p), 0 | b'=' | b'&') {
            p += 1;
        }
        let tag = &info[start..start + (p - start).min(127)];
        let mut q = 0;
        if at(info, p) == b'=' {
            p += 1;
            while !matches!(at(info, p), 0 | b'&') {
                if q + 1 < arg.len() {
                    arg[q] = if info[p] == b'+' { b' ' } else { info[p] };
                    q += 1;
                }
                p += 1;
            }
        }
        if let Some(end) = arg.get_mut(q) {
            *end = 0;
        }
        if tag == tag1 {
            return 1;
        }
        if at(info, p) != b'&' {
            return 0;
        }
        p += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        let (mut w, mut h) = (0, 0);
        assert_eq!(
            av_parse_video_size(Some(&mut w), Some(&mut h), Some(b"hd1080")),
            0
        );
        assert_eq!((w, h), (1920, 1080));
        assert_eq!(
            av_parse_video_size(Some(&mut w), Some(&mut h), Some(b" 640x 480")),
            0
        );
        assert_eq!((w, h), (640, 480));
        assert_eq!(
            av_parse_video_size(Some(&mut w), Some(&mut h), Some(b"640x480x")),
            EINVAL
        );
    }

    #[test]
    fn info_tags() {
        let mut arg = [0u8; 8];
        assert_eq!(
            av_find_info_tag(&mut arg, Some(b"b"), Some(b"?a=1&b=hello+world")),
            1
        );
        assert_eq!(&arg, b"hello w\0");
    }
}
