use bevy::prelude::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    Playing,
    Result,
}

#[derive(Component)]
pub struct TitleEntity;

#[derive(Component)]
pub struct ResultEntity;

pub mod playing;
pub mod result;
pub mod title;

pub use playing::{cleanup_playing, update_playing_input};
pub use result::{cleanup_result, setup_result, update_result_input};
pub use title::{cleanup_title, setup_title, update_title_input};
