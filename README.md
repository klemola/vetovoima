# vetovoima

![vetovoima_screenshot_1](docs/screenshot_1.png)

## _vetovoima_ is an arcade game where you control the gravity!

The world is a hollow circle with a star in the center. You're the **Yellow Block** and your goal is to navigate through shifting debris to the **Tall Blue Block** before the time runs out. The challenge is to survive the chaos that ensues from changes to gravity.

You can **move forward**, **slow down** and change the **intensity and direction of gravity**:

➡️ **Right arrow**: move forward

⬅️ **Left arrow**: slow down

↕️ **Up/Down arrow**: control the gravity

## Play _vetovoima_

vetovoima is up on [itch.io](https://yourmagicisworking.itch.io/vetovoima)

### Gameplay preview

> The preview is WebP format, which might not be supported by some browsers. [Try Vimeo instead](https://vimeo.com/723716079)

![vetovoima_level](docs/vetovoima_level.webp)

## Technical info

vetovoima is built with Rust, using the [Bevy game engine](https://bevyengine.org).
The visuals are rendered using [bevy_prototype_lyon](https://crates.io/crates/bevy_prototype_lyon) and the gravity/physics simulation is powered by [bevy_rapier2d](https://crates.io/crates/bevy_rapier2d).

## How to build and run the game

You can run the game like any common Rust project with **cargo**

`cargo run`

To create an optimized version, use

`cargo build --release`

You'll find the executable under `./target/release/`.
You need to symlink or copy the `assets` folder to the same directory where the release build is executed from.

