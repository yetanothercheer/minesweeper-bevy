use std::collections::HashMap;

use bevy::{input::system::exit_on_esc_system, prelude::*};

mod minesweeper;
use minesweeper::*;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Prologue,
    Playing,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 960.0,
            height: 540.0,
            title: "minesweeper-bevy".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_system(exit_on_esc_system)
        // Starts here
        .add_startup_system(startup)
        .add_state(GameState::Prologue)
        .add_system_set(SystemSet::on_update(GameState::Prologue).with_system(prologue))
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(minesweeper_setup))
        .add_system_set(SystemSet::on_update(GameState::Playing).with_system(minesweeper))
        .run()
}

struct GameRes {
    font_m: Handle<Font>,
}

fn startup(mut c: Commands, a: Res<AssetServer>) {
    c.spawn_bundle(UiCameraBundle::default());
    c.spawn_bundle(OrthographicCameraBundle::new_2d());
    c.insert_resource(GameRes {
        font_m: a.load("fonts/FiraMono-Medium.ttf"),
    });
}

fn prologue(
    mut c: Commands,
    mut frame_count: Local<usize>,
    mut query: Query<(Entity, &mut Text)>,
    mut state: ResMut<State<GameState>>,
    gr: Res<GameRes>,
) {
    *frame_count += 1;

    if *frame_count == 1 {
        c.spawn_bundle(Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "Minesweeper!".to_string(),
                    style: TextStyle {
                        font: gr.font_m.clone(),
                        font_size: 60.0,
                        color: Color::BLACK,
                    },
                }],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            },
            ..Default::default()
        });
    } else if *frame_count < 150 {
        let mut text: Mut<Text> = query.single_mut().1;
        text.sections[0].style.color = Color::rgba(1.0, 1.0, 1.0, (*frame_count as f32) / 150.0);
    } else if *frame_count == 150 {
        let e: Entity = query.single_mut().0;
        c.entity(e).despawn();
        // State change here:
        // GameState::Prologue => GameState::Playing
        state.overwrite_set(GameState::Playing).unwrap();
    }
}

#[derive(Component)]
struct Mine;

fn minesweeper_setup(mut c: Commands, gr: Res<GameRes>) {
    let scale_factor = 35.0;

    for i in 0..10 {
        for j in 0..10 {
            let pivot = Transform::from_translation(Vec3::new(
                (i as f32 - 4.5) * scale_factor,
                (j as f32 - 4.5) * scale_factor,
                0.0,
            ));

            let mut pivot_forward = pivot;
            pivot_forward.translation.z = 0.1;

            c.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(33.0, 33.0)),
                    ..Default::default()
                },
                transform: pivot,
                ..Default::default()
            })
            .insert(Mine);

            c.spawn_bundle(Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            font: gr.font_m.clone(),
                            font_size: 40.0,
                            color: Color::NONE,
                        },
                    }],
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                },
                transform: pivot_forward,
                ..Default::default()
            })
            .insert(Mine);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PlayingState {
    First,
    Playing,
    Finished,
}

impl Default for PlayingState {
    fn default() -> Self {
        return PlayingState::First;
    }
}

fn minesweeper(
    time: Res<Time>,
    // Input handling
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_moved_event_reader: EventReader<CursorMoved>,
    mut cursor_position: Local<Vec2>,
    // Game states
    mut mines: Local<Mines>,
    mut state: Local<PlayingState>,
    // Ugly type: (start_time: f64, end_time: f64, start:color: Color, end_color: Color)
    mut colors: Local<HashMap<usize, (f64, f64, Color, Color)>>,
    // Query
    mut sprites: Query<&mut Sprite, With<Mine>>,
    mut texts: Query<&mut Text, With<Mine>>,
) {
    if let Some(cursor_moved) = cursor_moved_event_reader.iter().last() {
        *cursor_position = cursor_moved.position;
    }
    if mouse_button_input.just_released(MouseButton::Left) {
        let x = (cursor_position.x - 960.0 / 2.0 + 35.0 * 5.0) / 35.0;
        let y = (cursor_position.y - 540.0 / 2.0 + 35.0 * 5.0) / 35.0;
        let click_outside = x <= 0.0 || y <= 0.0 || x > 10.0 || y > 10.0;

        match *state {
            PlayingState::First => {
                if !click_outside {
                    mines.generate(5, (y.ceil() as usize, x.ceil() as usize));
                    mines.reveal(y.ceil() as usize, x.ceil() as usize);
                    *state = PlayingState::Playing;
                }
            }
            PlayingState::Playing => {
                if !click_outside {
                    mines.reveal(y.ceil() as usize, x.ceil() as usize);
                    if mines.status() != Status::Unfinished {
                        println!("{:?}", mines.status());
                        *state = PlayingState::Finished;
                    }
                }
            }
            PlayingState::Finished => {
                if click_outside {
                    *mines = Mines::new(10, 10);
                    colors.clear();
                    for mut sprite in sprites.iter_mut() {
                        sprite.color = Color::WHITE;
                    }
                    for mut text in texts.iter_mut() {
                        text.sections[0].style.color = Color::NONE;
                    }
                    *state = PlayingState::First;
                }
            }
        }
    }

    let mut index = 0;

    for mut sprite in sprites.iter_mut() {
        let state = mines.state.get_mut(index).unwrap();

        if state.bomb && state.reveal() && !colors.contains_key(&index) {
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

        if !state.bomb && state.reveal() && !colors.contains_key(&index) {
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
            sprite.color = Color::rgb(
                p * d.r() + (1.0 - p) * c.r(),
                p * d.g() + (1.0 - p) * c.g(),
                p * d.b() + (1.0 - p) * c.b(),
            );
        }

        index += 1;
    }

    index = 0;
    for mut text in texts.iter_mut() {
        let state = mines.state.get_mut(index).unwrap();

        if !state.bomb && state.reveal() && state.surrounds != 0 {
            text.sections[0].value = format!("{}", state.surrounds);
            text.sections[0].style.color = Color::BLACK;
        }

        index += 1;
    }
}
