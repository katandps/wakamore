use bevy::prelude::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Title,
    Playing,
    Result,
}
pub use wakamore_render::{ResultEntity, TitleEntity};

pub mod playing;
pub mod result;
pub mod title;

pub use playing::{cleanup_playing, update_playing_input};
pub use result::{cleanup_result, update_result_input};
pub use title::{cleanup_title, update_title_input};
