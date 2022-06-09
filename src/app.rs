use bevy::prelude::*;

pub static APP_NAME: &str = "vetovoima";
pub const PIXELS_PER_METER: f32 = 16.0;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InMenu,
    InitGame,
    LoadingLevel,
    InGame,
    GameOver,
    ObserveSimulation,
}

pub fn cursor_visible<const VISIBILITY: bool>(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();

    window.set_cursor_visibility(VISIBILITY);
}
