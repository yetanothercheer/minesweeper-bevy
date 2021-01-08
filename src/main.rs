mod minesweeper;
mod cursor_move;

use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use crate::minesweeper::MinesweeperPlugin;

fn main() {
    App::build()
        .add_resource(WindowDescriptor {
            width: 960.0,
            height: 540.0,
            title: "minesweeper-bevy".to_string(),
            ..Default::default()
        })
        .add_resource(Msaa { samples: 1 })
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_plugin(MinesweeperPlugin)
        .run();
}

fn setup(commands: &mut Commands) {
    commands.spawn(CameraUiBundle::default()).spawn(Camera2dBundle::default());
}
