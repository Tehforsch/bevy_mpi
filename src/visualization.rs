use bevy::prelude::*;
use uom::si::f64::Length;
use uom::si::length::meter;

use super::Concentration;
use crate::quantities::number_density_unit;
use crate::Position;

const GRID_SIZE: f32 = 20.0;

pub fn update_cells_visually_system(mut cells: Query<(&mut Sprite, &Concentration)>) {
    for (mut sprite, concentration) in cells.iter_mut() {
        sprite.color = Color::rgb(
            (concentration.0 / number_density_unit()).value as f32,
            0.0,
            0.0,
        );
    }
}

pub fn spawn_sprites_system(mut commands: Commands, cells: Query<(Entity, &Position)>) {
    for (entity, pos) in cells.iter() {
        commands
            .entity(entity)
            .insert_bundle(get_sprite_at_position(pos.0, pos.1));
    }
}

fn get_sprite_at_position(x: Length, y: Length) -> SpriteBundle {
    let x = x / Length::new::<meter>(1.0);
    let y = y / Length::new::<meter>(1.0);
    SpriteBundle {
        transform: Transform {
            translation: Vec3::new(
                GRID_SIZE * x.value as f32 - 400.0,
                GRID_SIZE * y.value as f32 - 400.0,
                0.0,
            ),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(GRID_SIZE, GRID_SIZE)),
            ..default()
        },
        ..default()
    }
}

pub fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
