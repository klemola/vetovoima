use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioPlugin, AudioSource};

use crate::game::GameEvent;

pub struct SoundsPlugin;

#[derive(Component)]
struct Sounds {
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
            .add_system(react_to_game_event);
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let countdown_very_low: Handle<AudioSource> = asset_server.load("sounds/cd_very_low.ogg");
    let reach_goal: Handle<AudioSource> = asset_server.load("sounds/reach_goal.ogg");
    let game_over: Handle<AudioSource> = asset_server.load("sounds/cd_very_low.ogg");
    let gravity_quarter: Handle<AudioSource> = asset_server.load("sounds/g_quarter.ogg");
    let gravity_eight: Handle<AudioSource> = asset_server.load("sounds/g_eight.ogg");

    commands.insert_resource(Sounds {
        countdown_very_low,
        reach_goal,
        game_over,
        gravity_quarter,
        gravity_eight,
    })
}

fn react_to_game_event(
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
                audio.stop();
                audio.play_looped(sounds.gravity_quarter.clone());
            }
        };
    }
}
