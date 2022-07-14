use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};

use crate::{game::GameEvent, main_menu::MenuEvent};

pub struct SoundsPlugin;

#[derive(Component)]
struct Sounds {
    menu_music: Handle<AudioSource>,
    new_game: Handle<AudioSource>,
    countdown_very_low: Handle<AudioSource>,
    reach_goal: Handle<AudioSource>,
    game_over: Handle<AudioSource>,
    gravity_quarter: Handle<AudioSource>,
    gravity_eight: Handle<AudioSource>,
}

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_startup_system(load_assets)
            .add_system(process_menu_events)
            .add_system(process_game_events);
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Sounds {
        menu_music: asset_server.load("sounds/g_quarter.ogg"),
        new_game: asset_server.load("sounds/reach_goal.ogg"),
        countdown_very_low: asset_server.load("sounds/cd_very_low.ogg"),
        reach_goal: asset_server.load("sounds/reach_goal.ogg"),
        game_over: asset_server.load("sounds/cd_very_low.ogg"),
        gravity_quarter: asset_server.load("sounds/g_quarter.ogg"),
        gravity_eight: asset_server.load("sounds/g_eight.ogg"),
    })
}

fn process_menu_events(
    mut menu_event: EventReader<MenuEvent>,
    sounds: Res<Sounds>,
    audio: Res<Audio>,
) {
    for event in menu_event.iter() {
        match event {
            MenuEvent::EnterMenu => {
                audio.stop();
                audio.play_looped(sounds.menu_music.clone());
            }

            MenuEvent::BeginNewGame => {
                audio.stop();
                audio.play(sounds.new_game.clone());
            }
        }
    }
}

fn process_game_events(
    mut game_event: EventReader<GameEvent>,
    sounds: Res<Sounds>,
    audio: Res<Audio>,
) {
    for event in game_event.iter() {
        match event {
            GameEvent::CountdownTick(elapsed_secs) => {
                if *elapsed_secs > 0 {
                    match *elapsed_secs {
                        20 => {
                            audio.stop();
                            audio.play_looped(sounds.gravity_eight.clone());
                        }

                        1..=5 => {
                            audio.stop();
                            audio.play(sounds.countdown_very_low.clone());
                        }
                        _ => (),
                    }
                }
            }

            GameEvent::ReachGoal => {
                audio.stop();
                audio.play(sounds.reach_goal.clone());
            }

            GameEvent::GameOver => {
                audio.stop();
                audio.play(sounds.game_over.clone());
            }

            GameEvent::LevelStart => {
                audio.play_looped(sounds.gravity_quarter.clone());
            }
        };
    }
}
