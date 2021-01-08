#![allow(unused_imports)]
#![allow(dead_code)]

use bevy::prelude::*;
use rand::Rng;
use bevy::input::mouse::{MouseMotion, MouseButtonInput};
use crate::cursor_move::{CursorMoveState, CursorMovePlugin};

pub struct MinesweeperPlugin;

impl Plugin for MinesweeperPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app

            .init_resource::<ButtonMaterials>()
            .add_system(button_system.system())

            .add_plugin(CursorMovePlugin)
            .add_resource(Minesweeper::new())
            .add_startup_system(setup.system())
            .add_system(sweeper.system());
    }
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut mines: ResMut<Minesweeper>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Mutated<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text, With<RestartButton>>,
    mut t: Query<(&mut Text, &Box)>,
) {
    for (interaction, mut material, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                if mines.game_over {
                    for (mut text, b) in t.iter_mut() { text.value = " ".into(); }
                    mines.generate_state = false;
                    mines.game_over = false;
                }
                *material = button_materials.pressed.clone();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

struct Box {
    x: i32,
    y: i32,
}

struct RestartButton;

fn setup(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
        // ui camera
        .spawn(CameraUiBundle::default())
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                position: Rect {
                    ..Default::default()
                },
                // center button
                // margin: Rect::all(Val::Auto),

                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    value: "RESTART".to_string(),
                    font: asset_server.load("fonts/IBMPlexMono-Light.ttf"),
                    style: TextStyle {
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..Default::default()
                    },
                },
                ..Default::default()
            }).with(RestartButton);
        });

    for x in -5..5 {
        for y in -5..5 {
            commands.spawn(SpriteBundle {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(x as f32 * 20.0, y as f32 * 20.0, 0.0)),
                sprite: Sprite {
                    size: Vec2::new(19.0, 19.0),
                    resize_mode: SpriteResizeMode::Automatic,
                },
                ..Default::default()
            }).with(Box { x, y });
            commands.spawn(TextBundle {
                style: Style {
                    size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                    position_type: PositionType::Absolute,
                    position: Rect {
                        left: Val::Px(960.0 / 2.0 + x as f32 * 20.0 - 7.0),
                        top: Val::Px(540.0 / 2.0 - y as f32 * 20.0 - 5.0),
                        ..Default::default()
                    },
                    border: Rect::all(Val::Px(2.0)),
                    ..Default::default()
                },
                text: Text {
                    value: " ".to_string(),
                    font: asset_server.load("fonts/IBMPlexMono-Light.ttf"),
                    style: TextStyle {
                        font_size: 30.0,
                        color: Color::BLUE,
                        ..Default::default()
                    },
                },
                ..Default::default()
            }).with(Box { x, y });
        }
    }
}

fn sweeper(
    mut state: ResMut<CursorMoveState>,
    mut mines: ResMut<Minesweeper>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut t: Query<(&mut Text, &Box)>,
) {
    if mouse_button_input.just_released(MouseButton::Left) {
        let x = ((state.pos.x - 370.0) / 20.0).floor() as i32;
        let y = ((state.pos.y - 160.0) / 20.0).floor() as i32;
        // println!("{}", state.pos);
        // println!("{} {}", x, y);

        if x < 0 || y < 0 || x > 9 || y > 9 {
            return;
        }

        if mines.game_over {
            return;
        }

        let mut to_sweep = vec![];
        to_sweep.push((x, y));

        while !to_sweep.is_empty() {
            let (x, y) = to_sweep.pop().unwrap();

            if !mines.generate_state {
                if x >= 0 && x < 10 && y >= 0 && y < 10 {
                    println!("Generate");
                    mines.generate_mines(10, x, y);
                } else {
                    return;
                }
            }

            println!("Test {} {}", x, y);

            if !mines.revealed(x, y) {
                mines.mark_as_revealed(x, y);
                for (mut text, b) in t.iter_mut() {
                    if b.x == x - 5 && b.y == y - 5 {
                        if mines.test(x, y) {
                            text.value = "X".into();
                            text.style.color = Color::RED;
                            println!("You Lose It!");
                            mines.game_over = true;
                        } else {
                            let n = mines.neighbor(x, y);
                            text.value = n.to_string().into();
                            text.style.color = Color::BLUE;
                            if mines.check_win() {
                                println!("You Win!");
                                mines.game_over = true;
                            }
                            if n == 0 {
                                to_sweep.push((x + 1, y - 1));
                                to_sweep.push((x + 1, y));
                                to_sweep.push((x + 1, y + 1));
                                to_sweep.push((x - 1, y - 1));
                                to_sweep.push((x - 1, y));
                                to_sweep.push((x - 1, y + 1));
                                to_sweep.push((x, y - 1));
                                to_sweep.push((x, y + 1));
                            }
                        }
                    }
                }
            }
        }
    }
}

struct Minesweeper {
    game_over: bool,
    generate_state: bool,
    mines: Vec<bool>,
    mines_state: Vec<bool>,
}

impl Minesweeper {
    fn new() -> Minesweeper {
        Minesweeper {
            game_over: false,
            generate_state: false,
            mines: Vec::default(),
            mines_state: Vec::default(),
        }
    }

    fn check_win(&self) -> bool {
        for i in 0..100 {
            if !self.mines_state[i] && !self.mines[i] {
                return false;
            }
        }
        return true;
    }

    fn test(&self, x: i32, y: i32) -> bool {
        self.mines[(x * 10 + y) as usize]
    }

    fn neighbor(&self, x: i32, y: i32) -> i32 {
        let value = |x: i32, y: i32| -> i32 {
            if x < 0 || x > 9 || y < 0 || y > 9 {
                0
            } else {
                if self.mines[(x * 10 + y) as usize] { 1 } else { 0 }
            }
        };
        value(x + 1, y) + value(x - 1, y) + value(x, y + 1) + value(x, y - 1) + value(x - 1, y - 1) + value(x - 1, y + 1) + value(x + 1, y + 1) + value(x + 1, y - 1)
    }

    fn generate_mines(&mut self, num_mines: i32, exclude_x: i32, exclude_y: i32) {
        self.mines = vec![false; 100 as usize];
        let exclude = (exclude_x * 10 + exclude_y) as usize;
        (0..num_mines).for_each(|_| {
            loop {
                let index = rand::thread_rng().gen_range(0..100) as usize;
                if index != exclude && !self.mines[index] {
                    self.mines[index] = true;
                    break;
                }
            }
        });
        self.mines_state = vec![false; 100 as usize];
        self.generate_state = true;
    }

    fn revealed(&self, x: i32, y: i32) -> bool {
        if x >= 0 && x < 10 && y >= 0 && y < 10 {
            self.mines_state[(x * 10 + y) as usize]
        } else {
            true
        }
    }

    fn mark_as_revealed(&mut self, x: i32, y: i32) {
        if x >= 0 && x < 10 && y >= 0 && y < 10 {
            self.mines_state[(x * 10 + y) as usize] = true;
        }
    }
}
