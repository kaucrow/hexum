pub mod api;
pub mod features;
pub(crate) mod prelude;
mod init;
mod core;

#[cfg(test)]
mod tests;

pub use init::init;
pub use core::{
    Config,
    PlatformState,
    get_config,
    config::Environment,
    postgres,
};

pub(crate) use core::*;
pub(crate) use api::routes;