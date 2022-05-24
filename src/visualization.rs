use super::Concentration;
use bevy::prelude::*;

pub fn update_cells_visually_system(mut cells: Query<(&mut Sprite, &Concentration)>) {
    for (mut sprite, concentration) in cells.iter_mut() {
        sprite.color = Color::rgb(concentration.0 as f32, 0.0, 0.0);
    }
}
