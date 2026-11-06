//! Pure logic of DctrXD's JEI. Nothing here knows about the game, the disk or
//! the clock, so all of it can be tested with `cargo test`.

pub mod app;
pub mod categories;
pub mod config;
pub mod edit_box;
pub mod grid;
pub mod ingredient;
pub mod lang;
pub mod layer;
pub mod nav;
pub mod persist;
pub mod recipe;
pub mod scroll_math;
pub mod search;
