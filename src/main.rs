mod config;
mod create_grid;
mod mpi_world;
mod quantities;
mod visualization;

use bevy::prelude::*;
use config::DIFFUSION_CONSTANT;
use create_grid::create_grid_system;
use mpi_world::MpiWorld;
use quantities::NumberDensity;
use quantities::NumberDensityPerTime;
use quantities::TimeQuantity;
use uom::si::f64::Length;
use uom::si::time::second;
use visualization::setup_camera_system;
use visualization::spawn_sprites_system;
use visualization::update_cells_visually_system;

#[derive(Component, Debug)]
pub struct Position(Length, Length);
#[derive(Component, Debug)]
pub struct Concentration(NumberDensity);
#[derive(Component, Debug)]
pub struct Source(NumberDensityPerTime);
#[derive(Component, Debug)]
pub struct Neighbours(Vec<Entity>);

pub struct Timestep(TimeQuantity);

#[derive(Component, Debug)]
pub struct Red;
#[derive(Component, Debug)]
pub struct Black;

pub fn initialize_mpi_and_add_world_resource(app: &mut bevy::prelude::App) -> i32 {
    let mpi_world = MpiWorld::new();
    let rank = mpi_world.rank();
    app.insert_non_send_resource(mpi_world);
    rank
}

fn main() {
    let mut app = App::new();
    let rank = initialize_mpi_and_add_world_resource(&mut app);
    if rank == 0 {
        app.add_plugins(DefaultPlugins)
            .add_startup_system(setup_camera_system)
            .add_startup_system_to_stage(StartupStage::PostStartup, spawn_sprites_system)
            .add_system(update_cells_visually_system);
    } else {
        app.add_plugins(MinimalPlugins);
    }
    app.add_startup_system(create_grid_system)
        .add_system(source_system)
        .add_system(diffusion_system::<Red>)
        .add_system(diffusion_system::<Black>)
        .insert_resource(Timestep(TimeQuantity::new::<second>(1.0)))
        .run();
}

fn source_system(mut cells: Query<(&mut Concentration, &Source)>, timestep: Res<Timestep>) {
    for (mut concentration, source) in cells.iter_mut() {
        concentration.0 += source.0 * timestep.0;
    }
}

fn diffusion_system<T>(
    mut cells1: Query<(&mut Concentration, &Neighbours), With<T>>,
    cells2: Query<&Concentration, Without<T>>,
) where
    T: Component,
{
    for (mut concentration, neighbours) in cells1.iter_mut() {
        for neighbour in neighbours.0.iter() {
            let neighbour_concentration = cells2.get(*neighbour).unwrap();
            let flux = 0.5 * DIFFUSION_CONSTANT * (neighbour_concentration.0 - concentration.0);
            concentration.0 += flux;
        }
    }
}
