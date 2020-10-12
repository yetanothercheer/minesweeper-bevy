mod minesweeper;
mod cursor_move;

use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use crate::minesweeper::MinesweeperPlugin;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            width: 960,
            height: 540,
            title: String::from("Bevy Demo"),
            vsync: true,
            resizable: true,
            ..Default::default()
        })
        .add_resource(Msaa { samples: 1 })
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_default_plugins()
        .add_startup_system(setup.system())
        .add_plugin(MinesweeperPlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(UiCameraComponents::default()).spawn(Camera2dComponents::default());
}
