use bevy::prelude::*;
use bevy_kira_audio::{AudioApp, AudioChannel, AudioControl, AudioPlugin, AudioSource};
use std::marker::PhantomData;

use crate::{game::GameEvent, main_menu::MenuEvent};

pub struct SoundsPlugin;

#[derive(Resource)]
struct ChannelAudioState<T> {
    stopped: bool,
    paused: bool,
    loop_started: bool,
    volume: f64,
    _marker: PhantomData<T>,
}

impl<T> Default for ChannelAudioState<T> {
    fn default() -> Self {
        ChannelAudioState {
            volume: 1.0,
            stopped: true,
            loop_started: false,
            paused: false,
            _marker: PhantomData::<T>::default(),
        }
    }
}

#[derive(Resource, Component, Default, Clone)]
struct MainChannel;
#[derive(Resource, Component, Default, Clone)]
struct EffectChannel;
#[derive(Resource, Component, Default, Clone)]
struct TransitionChannel;

#[derive(Component, Resource)]
struct Sounds {
    menu_music: Handle<AudioSource>,
    new_game: Handle<AudioSource>,
    countdown_very_low: Handle<AudioSource>,
    reach_goal: Handle<AudioSource>,
    game_over: Handle<AudioSource>,
    tick_slow: Handle<AudioSource>,
    tick_fast: Handle<AudioSource>,
    bump_low: Handle<AudioSource>,
}

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_startup_system(audio_setup)
            .add_system(process_menu_events)
            .add_system(process_game_events)
            .add_audio_channel::<MainChannel>()
            .add_audio_channel::<EffectChannel>()
            .add_audio_channel::<TransitionChannel>();
    }
}

fn audio_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Sounds {
        menu_music: asset_server.load("sounds/tick_slow.ogg"),
        new_game: asset_server.load("sounds/new_game.ogg"),
        countdown_very_low: asset_server.load("sounds/cd_very_low.ogg"),
        reach_goal: asset_server.load("sounds/reach_goal.ogg"),
        game_over: asset_server.load("sounds/cd_very_low.ogg"),
        tick_slow: asset_server.load("sounds/tick_slow.ogg"),
        tick_fast: asset_server.load("sounds/tick_fast.ogg"),
        bump_low: asset_server.load("sounds/bump_low.ogg"),
    });

    commands.insert_resource(ChannelAudioState::<MainChannel>::default());
    commands.insert_resource(ChannelAudioState::<EffectChannel>::default());
    commands.insert_resource(ChannelAudioState::<TransitionChannel>::default());
}

fn process_menu_events(
    mut menu_event: EventReader<MenuEvent>,
    sounds: Res<Sounds>,
    main_channel: Res<AudioChannel<MainChannel>>,
) {
    for event in menu_event.iter() {
        match event {
            MenuEvent::EnterMenu => {
                main_channel.stop();
                main_channel.set_volume(1.0);
                main_channel.play(sounds.menu_music.clone()).looped();
            }

            MenuEvent::BeginNewGame => {
                main_channel.stop();
                main_channel.set_volume(1.0);
                main_channel.play(sounds.new_game.clone());
            }
        }
    }
}

fn process_game_events(
    mut game_event: EventReader<GameEvent>,
    sounds: Res<Sounds>,
    main_channel: Res<AudioChannel<MainChannel>>,
    effect_channel: Res<AudioChannel<EffectChannel>>,
) {
    for event in game_event.iter() {
        match event {
            GameEvent::CountdownTick(elapsed_secs) => {
                if *elapsed_secs > 0 {
                    match *elapsed_secs {
                        15 => {
                            main_channel.stop();
                            main_channel.set_volume(0.6);
                            main_channel.play(sounds.tick_fast.clone()).looped();
                        }

                        1..=5 => {
                            main_channel.stop();
                            main_channel.set_volume(0.75);
                            main_channel.play(sounds.countdown_very_low.clone());
                        }
                        _ => (),
                    }
                }
            }

            GameEvent::GoalReached => {
                main_channel.stop();
                main_channel.set_volume(1.0);
                main_channel.play(sounds.reach_goal.clone());
            }

            GameEvent::GameOver => {
                main_channel.stop();
                main_channel.set_volume(1.0);
                main_channel.play(sounds.game_over.clone());
            }

            GameEvent::LevelStarted => {
                main_channel.set_volume(0.6);
                main_channel.play(sounds.tick_slow.clone()).looped();
            }

            GameEvent::PlayerCollided(time_since_previous_collision, total_force_magnitude) => {
                let millis: f64 = time_since_previous_collision.as_secs_f64() * 1000.0;
                let playback_rate_increase = 1.0 - (millis / 200.0).min(1.0);
                let playback_rate = 1.0 + playback_rate_increase;

                let volume_coefficient = (total_force_magnitude / 750.0).min(1.0);
                let volume: f64 = 0.5 * volume_coefficient as f64;

                effect_channel.set_volume(volume);
                effect_channel.set_playback_rate(playback_rate);
                effect_channel.play(sounds.bump_low.clone());
            }
        };
    }
}
