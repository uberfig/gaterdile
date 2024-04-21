#![recursion_limit = "2048"]
#![type_length_limit = "2048"]

// #[macro_use]
// extern crate diesel;
// #[macro_use]
// extern crate rocket_sync_db_pools;

pub mod db;
pub mod db_event_types;
pub mod db_types;
pub mod handlers;
pub mod schema;
pub mod transmission;
