pub mod dtos;
pub mod search;
pub mod get;
pub mod popular;

pub use search::search;
pub use get::get_game;
pub use popular::popular_games;