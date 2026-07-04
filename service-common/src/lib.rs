// service-common/src/lib.rs
//! Shared code between `bet_api` and `trading-service`: starting/stopping the server,
//! reading the configuration from the env, and exposing the OpenAPI documentation.
//!
//! Any logic specific to a single service (routing, domain, data access)
//! remains in the crate of that service.

mod docs;
mod env;
mod shutdown;

pub use docs::docs_router;
pub use env::parse_u32_env;
pub use shutdown::shutdown_signal;
