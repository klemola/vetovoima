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
    tick_slow: Handle<AudioSource>,
    tick_fast: Handle<AudioSource>,
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
        menu_music: asset_server.load("sounds/tick_slow.ogg"),
        new_game: asset_server.load("sounds/new_game.ogg"),
        countdown_very_low: asset_server.load("sounds/cd_very_low.ogg"),
        reach_goal: asset_server.load("sounds/reach_goal.ogg"),
        game_over: asset_server.load("sounds/cd_very_low.ogg"),
        tick_slow: asset_server.load("sounds/tick_slow.ogg"),
        tick_fast: asset_server.load("sounds/tick_fast.ogg"),
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
                audio.set_volume(1.0);
                audio.play_looped(sounds.menu_music.clone());
            }

            MenuEvent::BeginNewGame => {
                audio.stop();
                audio.set_volume(1.0);
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
                            audio.set_volume(0.6);
                            audio.play_looped(sounds.tick_fast.clone());
                        }

                        1..=5 => {
                            audio.stop();
                            audio.set_volume(1.0);
                            audio.play(sounds.countdown_very_low.clone());
                        }
                        _ => (),
                    }
                }
            }

            GameEvent::ReachGoal => {
                audio.stop();
                audio.set_volume(1.0);
                audio.play(sounds.reach_goal.clone());
            }

            GameEvent::GameOver => {
                audio.stop();
                audio.set_volume(1.0);
                audio.play(sounds.game_over.clone());
            }

            GameEvent::LevelStart => {
                audio.set_volume(0.6);
                audio.play_looped(sounds.tick_slow.clone());
            }
        };
    }
}
