//! math.rs — pluggable distance metrics for VCAL–core.
//!
//! * `Metric` trait abstracts over cosine / dot etc.
//! * Works on stable Rust 1.56.
//! * Optional AVX2 fast-path behind `--features simd` and
//!   `RUSTFLAGS="-C target-cpu=native"` on x86_64.

/// Blanket trait — all metrics must return **smaller = closer** distance.
pub trait Metric: Send + Sync + 'static {
    fn distance(&self, a: &[f32], b: &[f32]) -> f32;
}

/// ----------------------------------------------------------------------
/// Cosine distance  — returns  1 – cos(θ)  ∈ [0, 2]
/// Robust to unnormalised inputs; if either vector is zero → 1.0.
/// ----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub struct Cosine;

impl Metric for Cosine {
    #[inline]
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len());

        let (mut dot, mut na, mut nb) = (0.0_f32, 0.0_f32, 0.0_f32);
        let mut i = 0usize;

        // ---------------- SIMD fast-path (AVX2) ----------------
        #[cfg(all(feature = "simd", target_arch = "x86_64"))]
        unsafe {
            use std::arch::x86_64::*;
            if is_x86_feature_detected!("avx2") {
                while i + 8 <= a.len() {
                    let va = _mm256_loadu_ps(a.as_ptr().add(i));
                    let vb = _mm256_loadu_ps(b.as_ptr().add(i));
                    dot += _mm256_reduce_add_ps(_mm256_mul_ps(va, vb));
                    na  += _mm256_reduce_add_ps(_mm256_mul_ps(va, va));
                    nb  += _mm256_reduce_add_ps(_mm256_mul_ps(vb, vb));
                    i += 8;
                }
            }
        }

        // ---------------- scalar tail (portable) ---------------
        while i < a.len() {
            let x = a[i];
            let y = b[i];
            dot += x * y;
            na  += x * x;
            nb  += y * y;
            i += 1;
        }

        if na == 0.0 || nb == 0.0 {
            return 1.0; // degenerate
        }
        let denom = (na.sqrt() * nb.sqrt()).max(1e-12);
        let cos = (dot / denom).max(-1.0).min(1.0); // clamp for safety
        1.0 - cos
    }
}

/// AVX2 helper: horizontal sum of 8-lane register.
#[cfg(all(feature = "simd", target_arch = "x86_64"))]
#[inline]
unsafe fn _mm256_reduce_add_ps(v: std::arch::x86_64::__m256) -> f32 {
    use std::arch::x86_64::*;
    let hi = _mm256_extractf128_ps(v, 1);
    let lo = _mm256_castps256_ps128(v);
    let sum128 = _mm_add_ps(lo, hi);
    let hi64 = _mm_movehl_ps(sum128, sum128);
    let sum64 = _mm_add_ps(sum128, hi64);
    let shuf = _mm_movehdup_ps(sum64);
    let result = _mm_add_ss(sum64, shuf);
    _mm_cvtss_f32(result)
}

/// ----------------------------------------------------------------------
/// Dot-product distance — returns 1 – dot(a, b)
/// Interprets **smaller = closer**; best when inputs are unit-normalised.
/// ----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub struct Dot;

impl Metric for Dot {
    #[inline]
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len());

        let mut dot = 0.0_f32;
        let mut i = 0usize;

        #[cfg(all(feature = "simd", target_arch = "x86_64"))]
        unsafe {
            use std::arch::x86_64::*;
            if is_x86_feature_detected!("avx2") {
                while i + 8 <= a.len() {
                    let va = _mm256_loadu_ps(a.as_ptr().add(i));
                    let vb = _mm256_loadu_ps(b.as_ptr().add(i));
                    dot += _mm256_reduce_add_ps(_mm256_mul_ps(va, vb));
                    i += 8;
                }
            }
        }

        while i < a.len() {
            dot += a[i] * b[i];
            i += 1;
        }

        1.0 - dot
    }
}
