use std::collections::HashMap;

use bevy::prelude::Commands;
use bevy::prelude::Res;
use uom::si::f64::*;
use uom::si::length::meter;
use uom::si::time::second;

use self::grid_data::GridData;
use crate::mpi_world::MpiWorld;
use crate::quantities::number_density_unit;
use crate::Black;
use crate::Concentration;
use crate::ExchangeCell;
use crate::HaloCell;
use crate::LocalCell;
use crate::Neighbours;
use crate::Position;
use crate::Red;
use crate::Source;

mod grid_data;

fn get_test_concentration(pos: Position) -> Concentration {
    let concentration = if pos.0 < Length::new::<meter>(1.0) {
        1.0
    } else {
        0.0
    };
    Concentration(concentration * number_density_unit())
}

fn get_grid_for_world_size_and_rank(world_size: i32, rank: i32) -> GridData {
    GridData::new(60, 60, world_size, 1, rank, 0)
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
            for rank in world.other_ranks() {
                if cell
                    .with_other_grid(&get_grid_for_world_size_and_rank(world.size(), rank))
                    .is_halo()
                {
                    entity_commands.insert(ExchangeCell);
                }
            }
        } else if cell.is_halo() {
            entity_commands.insert(HaloCell);
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
