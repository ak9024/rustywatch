//! Configuration schema and `.env` support.
//!
//! [`crate::Config`] and [`crate::Workspace`] are re-exported at the crate
//! root — prefer `rustywatch::Config` over the module path.

pub mod env_loader;
pub mod schema;
