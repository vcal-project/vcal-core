//! math.rs — pluggable distance metrics for VCAL-core.
//!
//! * `Metric` trait abstracts over cosine / dot etc.
//! * Works on stable Rust 1.56.
//! * Fully safe scalar implementation in v0.1.1.

pub trait Metric: Send + Sync + 'static {
    fn distance(&self, a: &[f32], b: &[f32]) -> f32;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Cosine;

impl Metric for Cosine {
    #[inline]
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len());

        let (mut dot, mut na, mut nb) = (0.0_f32, 0.0_f32, 0.0_f32);

        for i in 0..a.len() {
            let x = a[i];
            let y = b[i];
            dot += x * y;
            na += x * x;
            nb += y * y;
        }

        if na == 0.0 || nb == 0.0 {
            return 1.0;
        }

        const EPS: f32 = 1e-12;
        let denom = (na.sqrt() * nb.sqrt()).max(EPS);
        let cos = (dot / denom).clamp(-1.0, 1.0);
        1.0 - cos
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Dot;

impl Metric for Dot {
    #[inline]
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        debug_assert_eq!(a.len(), b.len());

        let mut dot = 0.0_f32;
        for i in 0..a.len() {
            dot += a[i] * b[i];
        }

        1.0 - dot
    }
}
