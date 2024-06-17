//! Algorithms to compute minimizers.

pub trait MinimizerAlgorithm {}

/// "Classic" minimizers.
pub struct Minimizer {}
impl MinimizerAlgorithm for Minimizer {}

/// Mod-minimizers, introduced in [The mod-minimizer: a simple and efficient sampling algorithm for long k-mers (Groot Koerkamp & Pibiri '24)](https://doi.org/10.1101/2024.05.25.595898).
pub struct ModMinimizer {}
impl MinimizerAlgorithm for ModMinimizer {}
