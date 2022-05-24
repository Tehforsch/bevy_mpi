use bevy::{ecs::query::WorldQuery, prelude::*};
use derive_more::{Deref, DerefMut};

#[derive(Component, Deref, DerefMut)]
struct Concentration(f64);
#[derive(Component, Deref, DerefMut)]
struct Source(f64);
#[derive(Component)]
struct Neighbours(Vec<Entity>);

#[derive(Component)]
struct Red;
#[derive(Component)]
struct Black;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_startup_system(create_grid_system)
        .add_system(source_system)
        .add_system(diffusion_system::<Red>)
        .add_system(diffusion_system::<Black>)
        .run();
}

fn source_system(mut cells: Query<(&mut Concentration, &Source)>) {
    for (mut concentration, source) in cells.iter_mut() {
        **concentration += source.0;
    }
}

fn diffusion_system<T1>(
    red_cells: Query<(&mut Concentration, &Source), With<T1>>,
    mut black_cells: Query<(&mut Concentration, &Source), Without<T1>>,
) where
    T1: Component,
{
}

fn create_grid_system(mut commands: Commands) {
    let nx = 4;
    let ny = 4;
    let wrap = |i: usize, j: usize| (i.rem_euclid(nx), j.rem_euclid(ny));
    let index = |i: usize, j: usize| i * ny + j;
    let mut entities = vec![];
    for _ in 0..nx {
        for _ in 0..ny {
            entities.push(
                commands
                    .spawn()
                    .insert(Concentration(0.0))
                    .insert(Source(0.0))
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
