use crate::{Black, Concentration, Neighbours, Red, Source};
use bevy::prelude::*;

const GRID_SIZE: f32 = 40.0;

pub fn create_grid_system(mut commands: Commands) {
    let nx = 20;
    let ny = 20;
    let wrap = |i: usize, j: usize| (i.rem_euclid(nx), j.rem_euclid(ny));
    let index = |i: usize, j: usize| i * ny + j;
    let mut entities = vec![];
    for i in 0..nx {
        for j in 0..ny {
            let concentration = if i <= 10 { 1.0 } else { 0.0 };
            entities.push(
                commands
                    .spawn()
                    .insert(Concentration(concentration))
                    .insert(Source(0.0))
                    .insert_bundle(SpriteBundle {
                        transform: Transform {
                            translation: Vec3::new(
                                GRID_SIZE * (i as f32),
                                GRID_SIZE * (j as f32),
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
                    })
                    .id(),
            );
        }
    }
    for i in 0..nx {
        for j in 0..ny {
            let neighbours = vec![
                wrap(i + (nx - 1), j + (ny - 1)),
                wrap(i + 1, j + (ny - 1)),
                wrap(i + (nx - 1), j + 1),
                wrap(i + 1, j + 1),
            ]
            .into_iter()
            .map(|(i, j)| entities[index(i, j)])
            .collect();
            let mut entity = commands.entity(entities[index(i, j)]);
            entity.insert(Neighbours(neighbours));
            if index(i, j).rem_euclid(2) == 0 {
                entity.insert(Red);
            } else {
                entity.insert(Black);
            }
        }
    }
}
