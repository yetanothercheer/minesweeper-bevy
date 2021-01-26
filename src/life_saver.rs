use bevy::{prelude::*, window::WindowResized};

pub fn SimpleText(value: &str, size: u32, color: Color, font: Handle<Font>, style: Style) -> TextBundle {
    TextBundle {
        style,
        text: Text {
            value: value.to_string(),
            font,
            style: TextStyle {
                font_size: size as f32,
                color,
                ..Default::default()
            },
        },
        ..Default::default()
    }
}
