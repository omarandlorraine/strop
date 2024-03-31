//! Module containing miscellaneous search algorithms that are generic across instruction sets.
//! Also contains some static analysis passes.
mod bruteforce;
mod stochastic;

pub use bruteforce::BruteForceSearch;
pub use bruteforce::CompatibilitySearch;
pub use bruteforce::LengthLimitedSearch;
pub use bruteforce::LinkageSearch;
pub use bruteforce::SearchTrace;
pub use bruteforce::BruteForce;
pub use stochastic::StochasticSearch;
