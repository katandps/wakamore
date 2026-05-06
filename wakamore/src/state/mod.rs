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

pub use playing::{
    PlayingInputEvent, cleanup_playing, map_playing_input_events, update_playing_input,
};
pub use result::{ResultInputEvent, cleanup_result, map_result_input_events, update_result_input};
pub use title::{TitleInputEvent, cleanup_title, map_title_input_events, update_title_input};
