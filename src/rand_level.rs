//! rand_level.rs — draw random layer for a new HNSW node.
//!
//! Follows the distribution from the original HNSW paper:
//! P(level ≥ l) = exp(-l / λ), where λ = 1 / ln(M).
//!
//! This implies a per-level promotion probability of:
//! p = exp(-1 / λ) = 1 / M.
//!
//! In code we implement this as a simple geometric sampler:
//! start at level 0 and keep promoting while rand() < 1/M.
//! This is branch-cheap and MSRV 1.56-friendly.
//!
//! ```rust
//! let lvl = vcal_core::draw_level(16);
//! assert!(lvl < 64);
//! ```

use rand::Rng;

#[inline]
pub fn draw_level(m: usize) -> usize {
    debug_assert!(m >= 2, "M must be ≥ 2");

    let p = 1.0 / m as f64;
    let mut lvl = 0usize;
    let mut rng = rand::thread_rng();

    while rng.gen::<f64>() < p {
        lvl += 1;
    }

    lvl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_level_is_usually_small_for_m16() {
        for _ in 0..10_000 {
            let lvl = draw_level(16);
            assert!(lvl < 16);
        }
    }

    #[test]
    fn average_level_drops_as_m_increases() {
        let n = 20_000;

        let avg = |m: usize| -> f64 {
            let mut sum = 0usize;
            for _ in 0..n {
                sum += draw_level(m);
            }
            sum as f64 / n as f64
        };

        let avg_m4 = avg(4);
        let avg_m16 = avg(16);
        let avg_m32 = avg(32);

        assert!(avg_m4 > avg_m16);
        assert!(avg_m16 > avg_m32);
    }

    #[test]
    fn mean_level_is_roughly_geometric() {
        let n = 50_000;
        let m = 16usize;
        let expected = 1.0 / (m as f64 - 1.0);

        let mut sum = 0usize;
        for _ in 0..n {
            sum += draw_level(m);
        }
        let observed = sum as f64 / n as f64;

        assert!((observed - expected).abs() < 0.03);
    }
}
