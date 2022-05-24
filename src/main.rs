mod create_grid;
mod visualization;

use bevy::prelude::*;
use create_grid::create_grid_system;
use visualization::update_cells_visually_system;

#[derive(Component)]
pub struct Concentration(f64);
#[derive(Component)]
pub struct Source(f64);
#[derive(Component)]
pub struct Neighbours(Vec<Entity>);

#[derive(Component)]
pub struct Red;
#[derive(Component)]
pub struct Black;

const DIFFUSION_CONSTANT: f64 = 0.1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(create_grid_system)
        .add_startup_system(setup_camera_system)
        .add_system(source_system)
        .add_system(diffusion_system::<Red>)
        .add_system(diffusion_system::<Black>)
        .add_system(update_cells_visually_system)
        .run();
}

fn source_system(mut cells: Query<(&mut Concentration, &Source)>) {
    for (mut concentration, source) in cells.iter_mut() {
        concentration.0 += source.0;
    }
}

fn diffusion_system<T1>(
    mut cells1: Query<(&mut Concentration, &Neighbours), With<T1>>,
    cells2: Query<&Concentration, Without<T1>>,
) where
    T1: Component,
{
    for (mut concentration, neighbours) in cells1.iter_mut() {
        for neighbour in neighbours.0.iter() {
            let neighbour_concentration = cells2.get(*neighbour).unwrap();
            concentration.0 +=
                0.5 * DIFFUSION_CONSTANT * (neighbour_concentration.0 - concentration.0);
        }
    }
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
