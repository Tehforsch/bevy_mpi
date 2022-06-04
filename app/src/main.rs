// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]

mod communication;
mod config;
mod create_grid;
mod mpi_world;
mod quantities;
mod visualization;

use bevy::prelude::*;
use communication::send_queues::SendQueues;
use config::DIFFUSION_CONSTANT;
use create_grid::create_grid_system;
use mpi::topology::Rank;
use mpi_world::MpiWorld;
use quantities::NumberDensity;
use quantities::NumberDensityPerTime;
use quantities::TimeQuantity;
use uom::si::f64::Length;
use uom::si::time::second;
use uom_derive::UomEquivalence;
use visualization::setup_camera_system;
use visualization::spawn_sprites_system;
use visualization::update_cells_visually_system;

use crate::quantities::number_density_unit;

#[derive(Component, Debug, UomEquivalence)]
pub struct Position(Length, Length);
#[derive(Component, Debug, Clone)]
pub struct Concentration(NumberDensity);
#[derive(Component, Debug)]
struct Source(NumberDensityPerTime);
#[derive(Component, Debug)]
struct Neighbours(Vec<Entity>);

struct Timestep(TimeQuantity);

#[derive(Component, Debug)]
struct Red;
#[derive(Component, Debug)]
struct Black;

/// A cell that has its values updated by this rank
#[derive(Component, Debug)]
struct LocalCell;

/// A cell that is only used to update local cells
/// but whose values correspond to those of a local cell
/// on another rank
#[derive(Component, Debug)]
pub struct HaloCell(Rank);

/// A local cell which has information that another rank will need
/// (that is, it has a halo cell corresponding to it on another rank)
#[derive(Component, Debug)]
struct ExchangeCell {
    rank: Rank,
    /// The entity index on the target rank
    entity: Entity,
}

fn initialize_mpi_and_add_world_resource(app: &mut bevy::prelude::App) -> Rank {
    let mpi_world = MpiWorld::new();
    let rank = mpi_world.rank();
    app.insert_resource(SendQueues::<(Entity, Concentration)>::from_mpi_world(
        &mpi_world,
    ));
    app.insert_resource(mpi_world);
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
        .add_system(diffusion_system::<Red>.after(source_system))
        .add_system(diffusion_system::<Black>.after(diffusion_system::<Red>))
        .add_system(exchange_system.after(diffusion_system::<Black>))
        .add_system(SendQueues::<Concentration>::send_all_system)
        .add_system(print_total_concentration_system)
        .insert_resource(Timestep(TimeQuantity::new::<second>(1.0)))
        .run();
}

fn source_system(mut cells: Query<(&mut Concentration, &Source)>, timestep: Res<Timestep>) {
    for (mut concentration, source) in cells.iter_mut() {
        concentration.0 += source.0 * timestep.0;
    }
}

fn diffusion_system<T>(
    mut cells1: Query<(&mut Concentration, &Neighbours), (With<LocalCell>, With<T>)>,
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

fn exchange_system(
    cells: Query<(&Concentration, &ExchangeCell)>,
    mut queues: ResMut<SendQueues<(Entity, Concentration)>>,
) {
    for (concentration, cell) in cells.iter() {
        queues.push(cell.rank, (cell.entity, concentration.clone()));
    }
}

fn print_total_concentration_system(
    cells: Query<&Concentration, With<LocalCell>>,
    world: Res<MpiWorld>,
) {
    println!(
        "{}: Total: {}",
        world.rank(),
        cells
            .iter()
            .map(|x| (x.0 / number_density_unit()).value)
            .sum::<f64>()
    );
}
