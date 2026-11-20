//! C ABI for `mathematics.c`: drop-in replacements with the original names and signatures, generated
//! by rust-anything from `rust-anything.json`. All of the crate's `unsafe` code lives in this file.

pub mod port;

/// `int64_t av_gcd(int64_t a, int64_t b)` (`mathematics.c:37`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_gcd(a: i64, b: i64) -> i64 {
    port::av_gcd(a, b)
}

/// `av_gcd` as `rust_anything_av_gcd`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_gcd(a: i64, b: i64) -> i64 {
    port::av_gcd(a, b)
}

/// `int64_t av_rescale_rnd(int64_t a, int64_t b, int64_t c, enum AVRounding rnd)` (`mathematics.c:58`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_rescale_rnd(a: i64, b: i64, c: i64, rnd: std::ffi::c_int) -> i64 {
    port::av_rescale_rnd(a, b, c, rnd)
}

/// `av_rescale_rnd` as `rust_anything_av_rescale_rnd`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_rescale_rnd(
    a: i64,
    b: i64,
    c: i64,
    rnd: std::ffi::c_int,
) -> i64 {
    port::av_rescale_rnd(a, b, c, rnd)
}

/// `int64_t av_rescale(int64_t a, int64_t b, int64_t c)` (`mathematics.c:129`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_rescale(a: i64, b: i64, c: i64) -> i64 {
    port::av_rescale(a, b, c)
}

/// `av_rescale` as `rust_anything_av_rescale`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_rescale(a: i64, b: i64, c: i64) -> i64 {
    port::av_rescale(a, b, c)
}

/// `int64_t av_compare_mod(uint64_t a, uint64_t b, uint64_t mod)` (`mathematics.c:160`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_compare_mod(a: u64, b: u64, r#mod: u64) -> i64 {
    port::av_compare_mod(a, b, r#mod)
}

/// `av_compare_mod` as `rust_anything_av_compare_mod`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_compare_mod(a: u64, b: u64, r#mod: u64) -> i64 {
    port::av_compare_mod(a, b, r#mod)
}

/// `double av_bessel_i0(double x)` (`mathematics.c:257`).
#[cfg(not(feature = "shadow"))]
// SAFETY: exported under the C name it replaces: link it instead of the original object.
#[unsafe(no_mangle)]
pub extern "C" fn av_bessel_i0(x: f64) -> f64 {
    port::av_bessel_i0(x)
}

/// `av_bessel_i0` as `rust_anything_av_bessel_i0`, for shadow builds beside the original.
// SAFETY: a name no C code defines.
#[unsafe(no_mangle)]
pub extern "C" fn rust_anything_av_bessel_i0(x: f64) -> f64 {
    port::av_bessel_i0(x)
}
