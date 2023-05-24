use bevy::{prelude::*, window::PrimaryWindow};

pub static APP_NAME: &str = "vetovoima";
pub const PIXELS_PER_METER: f32 = 18.0;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Init,
    InMenu,
    LoadingLevel,
    InGame,
    GameOver,
}

#[derive(Component, Clone, Debug, Default, Resource)]
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

#[derive(Resource)]
pub struct UiConfig {
    pub font_filename: &'static str,
    pub font_size_screen_title: f32,
    pub font_size_body_small: f32,
    pub font_size_countdown: f32,
    pub font_size_countdown_large: f32,
    pub font_size_menu_item: f32,
    pub font_size_app_title: f32,
    pub scale_multiplier: f32,
}

impl Default for UiConfig {
    fn default() -> Self {
        // The default values are alright for 720p-ish screen
        UiConfig {
            font_filename: "VT323-Regular.ttf",
            font_size_screen_title: 100.0,
            font_size_body_small: 24.0,
            font_size_countdown: 36.0,
            font_size_countdown_large: 48.0,
            font_size_menu_item: 64.0,
            font_size_app_title: 102.0,
            scale_multiplier: 1.0,
        }
    }
}

impl UiConfig {
    pub fn scale(ui_scale: f32) -> Self {
        let defaults = UiConfig::default();

        // The closer the ui_scale gets to 1.0, the more UI is scaled up (ui_scale of 1.0 means ~4k resolution)
        let scale_multiplier = if ui_scale < 1.5 {
            2.0
        } else if ui_scale < 2.0 {
            1.4
        } else {
            1.0
        };

        UiConfig {
            font_filename: defaults.font_filename,
            font_size_screen_title: defaults.font_size_screen_title * scale_multiplier,
            font_size_body_small: defaults.font_size_body_small * scale_multiplier,
            font_size_countdown: defaults.font_size_countdown * scale_multiplier,
            font_size_countdown_large: defaults.font_size_countdown_large * scale_multiplier,
            font_size_menu_item: defaults.font_size_menu_item * scale_multiplier,
            font_size_app_title: defaults.font_size_app_title * scale_multiplier,
            scale_multiplier,
        }
    }
}

pub fn cursor_visible<const VISIBILITY: bool>(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = window.get_single_mut().unwrap();

    window.cursor.visible = VISIBILITY;
}
