mod domain;
mod input;
mod output;
mod service;
pub mod postgres;
mod igdb;

pub use domain::*;
pub use input::*;
pub use output::*;
pub use service::*;
pub use postgres::PostgresAdapter;
pub use igdb::IgdbAdapter;