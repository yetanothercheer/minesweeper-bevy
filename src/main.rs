mod life_saver;
mod minesweeper;

use std::collections::HashMap;

use bevy::render::pass::ClearColor;
use bevy::{prelude::*};
use life_saver::*;
use minesweeper::*;

fn main() {
    env_logger::init();

    App::build()
        .add_resource(WindowDescriptor {
            width: 960.0,
            height: 540.0,
            title: "minesweeper-bevy".into(),
            resizable: false,
            ..Default::default()
        })
        .add_resource(Msaa { samples: 1 })
        .add_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup.system())
        .add_system(prologue.system())
        .add_system(minesweeper.system())
        .run()
}

fn startup(commands: &mut Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .spawn(CameraUiBundle::default());
}

struct Prologue;

fn prologue(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,

    mut frame_count: Local<usize>,
    mut query: Query<&mut Text>,
    query_prologue: Query<Entity, With<Prologue>>,
) {
    *frame_count += 1;

    if *frame_count == 1 {
        let font = asset_server.load("fonts/IBMPlexMono-Light.ttf");
        commands
            .spawn(SimpleText(
                "Minesweeper !",
                60,
                Color::WHITE,
                font,
                Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Px(540.0 / 2.0 - 50.0),
                        left: Val::Px(960.0 / 2.0 - 160.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ))
            .with(Prologue);
    }

    if *frame_count < 150 {
        for mut s in query.iter_mut() {
            s.style.color = Color::rgba(1.0, 1.0, 1.0, (*frame_count as f32).abs() / 150.0);
        }
    }

    if *frame_count == 150 {
        for s in query_prologue.iter() {
            commands.despawn(s);
        }
        let scale_factor = 35.0;

        for i in 0..10 {
            for j in 0..10 {
                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            size: Vec2::new(30.0, 30.0),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            i as f32 * scale_factor - 4.5 * scale_factor,
                            j as f32 * scale_factor - 4.5 * scale_factor,
                            0.0,
                        )),
                        ..Default::default()
                    })
                    .with(Mine);
            }
        }
    }
}

struct Mine;

impl FromResources for Mines {
    fn from_resources(_: &Resources) -> Self {
        Mines::new(10, 10)
    }
}

fn minesweeper(
    mut materials: ResMut<Assets<ColorMaterial>>,

    time: Res<Time>,
    mut colors: Local<HashMap<usize, (f64, f64, Color, Color)>>,

    // Click
    mouse_button_input: Res<Input<MouseButton>>,
    // Mouse position
    cursor_moved_events: Res<Events<CursorMoved>>,
    mut cursor_moved_event_reader: Local<EventReader<CursorMoved>>,
    mut cursor_position: Local<Vec2>,
    // Game state
    mut mines: Local<Mines>,
    mut ongoing: Local<bool>,
    // Sprite
    mut query: Query<&mut Handle<ColorMaterial>, With<Mine>>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        let x = (cursor_position.x - 960.0 / 2.0 + 35.0 * 5.0) / 35.0;
        let y = (cursor_position.y - 540.0 / 2.0 + 35.0 * 5.0) / 35.0;

        if x <= 0.0 || y <= 0.0 || x > 10.0 || y > 10.0 {
            *mines = Mines::new(10, 10);
            *ongoing = false;
            for mut material in query.iter_mut() {
                *material = materials.add(Color::WHITE.into());
                colors.clear();
            }
        }
    }

    match mines.status() {
        Status::GameOver | Status::Win => {
            match cursor_moved_event_reader.latest(&cursor_moved_events) {
                Some(e) => {
                    *cursor_position = e.position;
                }
                None => {}
            }
            return;
        }
        _ => {}
    }

    if mouse_button_input.just_released(MouseButton::Left) {
        let x = (cursor_position.x - 960.0 / 2.0 + 35.0 * 5.0) / 35.0;
        let y = (cursor_position.y - 540.0 / 2.0 + 35.0 * 5.0) / 35.0;

        if !*ongoing {
            *ongoing = true;
            mines.generate(20, (y.ceil() as usize, x.ceil() as usize));
        }

        mines.reveal(y.ceil() as usize, x.ceil() as usize);
    }

    match cursor_moved_event_reader.latest(&cursor_moved_events) {
        Some(e) => {
            *cursor_position = e.position;
        }
        None => {}
    }
    let mut index = 0;
    for mut material in query.iter_mut() {
        let state = mines.state.get_mut(index).unwrap();

        let x = index % 10;
        let y = index / 10;

        if state.bomb && state.reveal && !colors.contains_key(&index) {
            colors.insert(
                index,
                (
                    time.seconds_since_startup(),
                    time.seconds_since_startup(),
                    Color::rgb(1.0, 1.0, 1.0),
                    Color::rgb(1.0, 0.4, 0.5),
                ),
            );
        }

        if !state.bomb && state.reveal && !colors.contains_key(&index) {
            colors.insert(
                index,
                (
                    time.seconds_since_startup(),
                    time.seconds_since_startup() + 0.15,
                    Color::rgb(1.0, 1.0, 1.0),
                    Color::rgb(0.3, 1.0 - state.surrounds as f32 / 8.0, 0.6),
                ),
            );
        }

        if colors.contains_key(&index) {
            let (a, b, c, d) = colors.get(&index).unwrap();
            let p = if time.seconds_since_startup() >= *b {
                1.0
            } else {
                ((time.seconds_since_startup() - a) / (b - a)) as f32
            };
            *material = materials.add(
                Color::rgb(
                    p * d.r() + (1.0 - p) * c.r(),
                    p * d.g() + (1.0 - p) * c.g(),
                    p * d.b() + (1.0 - p) * c.b(),
                )
                .into(),
            );
        }

        index += 1;
    }
}
