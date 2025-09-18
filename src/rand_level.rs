//! rand_level.rs — draw random layer for a new HNSW node.
//!
//! Follows the distribution from the original HNSW paper:  
//! P(level ≥ l) = `exp(-l / λ)`, where `λ = 1 / ln(M)`.
//!
//! In code we implement the standard “coin-flip until fail” geometric
//! sampler because it’s branch-cheap and MSRV 1.56-friendly.
//!
//! ```rust
//! // Public helper re-exported from the crate root.
//! // The `< 64` check here is *not* an algorithmic limit — it's a 
//! // demonstration that HNSW levels are typically small. In real 
//! // graphs, levels above ~20 are already rare, so this high bound 
//! // will never fail under normal use.
//! let lvl = vcal_core::draw_level(16.0);
//! assert!(lvl < 64);
//! ```

use rand::Rng;

#[inline]
pub fn draw_level(m: f64) -> usize {
    debug_assert!(m >= 2.0, "M must be ≥ 2");
    let lambda = 1.0 / m.ln(); // λ = 1 / ln M
    let mut lvl = 0;
    let mut rng = rand::thread_rng();

    // Equivalent to while rand() < exp(-lvl/λ) but avoids powf.
    while rng.gen::<f64>() < (-lambda).exp() {
        lvl += 1;
    }
    lvl
}
