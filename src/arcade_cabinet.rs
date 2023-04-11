use bevy::{input::gamepad::GamepadConnection, input::gamepad::GamepadEvent, prelude::*};

pub struct RustArcadePlugin;
impl Plugin for RustArcadePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ArcadeInputEvent>()
            .add_system(input_events_system);
    }
}

// Inputs on the arcade machine
#[derive(Debug, Clone)]
pub enum ArcadeInput {
    JoyUp,
    JoyDown,
    JoyLeft,
    JoyRight,
    JoyButton,
    ButtonTop1,
    ButtonTop2,
    ButtonTop3,
    ButtonTop4,
    ButtonTop5,
    ButtonTop6,
    ButtonLeftSide,
    ButtonRightSide,
    ButtonFront1,
    ButtonFront2,
}

// Event for sending the input data
pub struct ArcadeInputEvent {
    pub gamepad: Gamepad,
    pub arcade_input: ArcadeInput,
    pub value: f32,
}

// Read gamepad inputs and convert to arcade inputs
fn input_events_system(
    mut gamepad_event: EventReader<GamepadEvent>,
    mut arcade_gamepad_event: EventWriter<ArcadeInputEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent::Connection(connection_event) => match &connection_event.connection {
                GamepadConnection::Connected(_) => {
                    info!("{:?} Connected", &connection_event.gamepad)
                }
                GamepadConnection::Disconnected => {
                    info!("{:?} Disconnected", &connection_event.gamepad)
                }
            },
            GamepadEvent::Button(button_event) => {
                let arcade_input = match button_event.button_type {
                    GamepadButtonType::DPadUp => Some(ArcadeInput::JoyUp),
                    GamepadButtonType::DPadDown => Some(ArcadeInput::JoyDown),
                    GamepadButtonType::DPadLeft => Some(ArcadeInput::JoyLeft),
                    GamepadButtonType::DPadRight => Some(ArcadeInput::JoyRight),
                    GamepadButtonType::South => Some(ArcadeInput::JoyButton),
                    GamepadButtonType::East => Some(ArcadeInput::ButtonTop1),
                    GamepadButtonType::West => Some(ArcadeInput::ButtonTop2),
                    GamepadButtonType::LeftThumb => Some(ArcadeInput::ButtonTop3),
                    GamepadButtonType::North => Some(ArcadeInput::ButtonTop4),
                    GamepadButtonType::LeftTrigger => Some(ArcadeInput::ButtonTop5),
                    GamepadButtonType::RightTrigger => Some(ArcadeInput::ButtonTop6),
                    GamepadButtonType::LeftTrigger2 => Some(ArcadeInput::ButtonLeftSide),
                    GamepadButtonType::RightTrigger2 => Some(ArcadeInput::ButtonRightSide),
                    GamepadButtonType::Select => Some(ArcadeInput::ButtonFront1),
                    GamepadButtonType::Start => Some(ArcadeInput::ButtonFront2),
                    _ => None,
                };

                if let Some(arcade_input) = arcade_input {
                    arcade_gamepad_event.send(ArcadeInputEvent {
                        gamepad: *&button_event.gamepad,
                        arcade_input,
                        value: button_event.value,
                    });
                }
            }

            _ => {}
        }
    }
}
