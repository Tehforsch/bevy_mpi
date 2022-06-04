use std::collections::HashMap;

use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use mpi::topology::Rank;
use mpi::traits::Equivalence;
use uom::si::f64::*;
use uom::si::length::meter;
use uom::si::time::second;

use self::grid_data::CellIdentifier;
use self::grid_data::GridData;
use crate::mpi_world::MpiWorld;
use crate::quantities::number_density_unit;
use crate::Black;
use crate::Concentration;
use crate::HaloCell;
use crate::LocalCell;
use crate::Neighbours;
use crate::Position;
use crate::Red;
use crate::Source;

mod grid_data;

fn get_test_concentration(pos: Position) -> Concentration {
    let concentration = if pos.0 < Length::new::<meter>(10.0) {
        1.0
    } else {
        0.0
    };
    Concentration(concentration * number_density_unit())
}

fn get_grid_for_world_size_and_rank(world_size: i32, rank: Rank) -> GridData {
    GridData::new(60, 60, world_size, 1, rank, 0)
}

fn get_halo_rank(world: &MpiWorld, cell: &CellIdentifier) -> Option<Rank> {
    world.other_ranks().find(|rank| {
        cell.with_other_grid(get_grid_for_world_size_and_rank(world.size(), *rank))
            .is_local()
    })
}

pub fn create_grid_system(mut commands: Commands, world: Res<MpiWorld>) {
    let grid = get_grid_for_world_size_and_rank(world.size(), world.rank());
    let mut entities = HashMap::new();
    for cell in grid.iter_local_cells_and_haloes() {
        let concentration = get_test_concentration(cell.get_position());
        let mut entity_commands = commands.spawn();
        entity_commands
            .insert(concentration)
            .insert(Source(
                0.0 * number_density_unit() / Time::new::<second>(1.0),
            ))
            .insert(cell.get_position());
        if cell.is_local() {
            entity_commands.insert(LocalCell);
        } else if cell.is_halo() {
            entity_commands.insert(HaloCell(get_halo_rank(&world, &cell).unwrap()));
        }
        entities.insert(cell, entity_commands.id());
    }
    for cell in grid.iter_local_grid_cells() {
        let mut entity = commands.entity(entities[&cell]);
        let neighbours = cell.get_neighbours();
        entity.insert(Neighbours(
            neighbours
                .into_iter()
                .map(|neighbour| entities[&neighbour])
                .collect(),
        ));
        if cell.is_even() {
            entity.insert(Red);
        } else {
            entity.insert(Black);
        }
    }
}

pub fn exchange_halo_information_system(
    mut commands: Commands,
    halo_cells: Query<(Entity, &HaloCell, &Position)>,
    world: Res<MpiWorld>,
) {
    for (entity, halo_cell, pos) in halo_cells.iter() {
        let rank = halo_cell.0;
        #[derive(Equivalence)]
        struct A {
            x: f64,
            y: f64,
        }
        world.send(rank, A { x: pos.0.value, y: pos.1.value });
    }
}
