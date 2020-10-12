use bevy::prelude::*;
use rand::Rng;
use bevy::input::mouse::{MouseMotion, MouseButtonInput};

pub struct CursorMovePlugin;

impl Plugin for CursorMovePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .init_resource::<CursorMoveState>()
            .add_system(cursor_move_system.system());
    }
}

#[derive(Default)]
pub struct CursorMoveState {
    cursor_moved_event_reader: EventReader<CursorMoved>,
    pub(crate) pos: Vec2,
}

fn cursor_move_system(
    mut state: ResMut<CursorMoveState>,
    cursor_moved_events: Res<Events<CursorMoved>>,
) {
    for event in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        state.pos = event.position;
    }
}