use bevy::prelude::*;

pub static APP_NAME: &str = "vetovoima";
pub const PIXELS_PER_METER: f32 = 12.0;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InMenu,
    LoadingLevel,
    InGame,
    GameOver,
}

#[derive(Component, Clone, Debug, Default)]
pub struct ButtonPress {
    pub select_pressed: bool,
    pub start_pressed: bool,
    pub main_control_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
}

pub enum VetovoimaColor {}

impl VetovoimaColor {
    pub const BLACKISH: Color = Color::hsl(0.0, 0.0, 0.0);
    pub const WHITEISH: Color = Color::hsl(25.0, 1.0, 0.9);
    pub const BLUEISH_LIGHT: Color = Color::hsl(220.0, 1.0, 0.66);
    pub const BLUEISH_DARK: Color = Color::hsl(220.0, 0.5, 0.2);
    pub const BLUEISH_MID: Color = Color::hsl(220.0, 0.5, 0.4);
    pub const REDDISH: Color = Color::hsl(10.0, 1.0, 0.66);
    pub const YELLOWISH: Color = Color::hsl(50.0, 1.0, 0.66);
    pub const GREENISH: Color = Color::hsl(150.0, 1.0, 0.66);
}

pub fn cursor_visible<const VISIBILITY: bool>(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();

    window.set_cursor_visibility(VISIBILITY);
}
