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
            .add_plugin(CursorMovePlugin)
            .add_resource(Minesweeper::new())
            .add_startup_system(setup.system())
            .add_system(sweeper.system());
    }
}

struct Box {
    x: i32,
    y: i32,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut asset_server: Res<AssetServer>,
) {
    let font_handle = asset_server.load("assets/fonts/IBMPlexMono-Light.ttf").unwrap();
    for x in -5..5 {
        for y in -5..5 {
            commands.spawn(SpriteComponents {
                material: materials.add(Color::rgb(1.0, 1.0, 1.0).into()),
                transform: Transform::from_translation(Vec3::new(x as f32 * 20.0, y as f32 * 20.0, 0.0)),
                sprite: Sprite {
                    size: Vec2::new(19.0, 19.0),
                    resize_mode: SpriteResizeMode::Automatic,
                },
                ..Default::default()
            }).with(Box { x, y });
            commands.spawn(TextComponents {
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
                    font: font_handle,
                    style: TextStyle {
                        font_size: 30.0,
                        color: Color::BLUE,
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
        let x = ((state.pos.x() - 370.0) / 20.0).floor() as i32;
        let y = ((state.pos.y() - 160.0) / 20.0).floor() as i32;
        // println!("{}", state.pos);
        // println!("{} {}", x, y);

        // Restart Game
        if x < 0 || y < 0 || x > 9 || y > 9 {
            for (mut text, b) in &mut t.iter() { text.value = " ".into(); }
            mines.generate_state = false;
            mines.game_over = false;
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
                for (mut text, b) in &mut t.iter() {
                    if b.x == x - 5 && b.y == y - 5 {
                        if mines.test(x, y) {
                            text.value = "X".into();
                            text.style.color = Color::RED;
                            println!("You Lose It!");
                            mines.game_over = true;
                        } else {
                            let n = mines.neighbor(x, y);
                            text.value = n.to_string().into();
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
                let index = rand::thread_rng().gen_range(0, 100) as usize;
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
