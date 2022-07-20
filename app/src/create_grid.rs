use std::collections::HashMap;

use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::With;
use mpi::point_to_point::Status;
use mpi::topology::Communicator;
use mpi::topology::Rank;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use uom::si::f64::*;
use uom::si::length::meter;
use uom::si::time::second;

use self::grid_data::CellIdentifier;
use self::grid_data::GridData;
use crate::mpi_world::MpiWorld;
use crate::position::Position;
use crate::quantities::number_density_unit;
use crate::Black;
use crate::Concentration;
use crate::ExchangeCell;
use crate::HaloCell;
use crate::LocalCell;
use crate::Neighbours;
use crate::Red;

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
            .insert(crate::Source(
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

pub(super) fn setup_halo_exchange_system(
    mut commands: Commands,
    halo_cells: Query<(Entity, &HaloCell, &Position)>,
    local_cells: Query<(Entity, &Position), With<LocalCell>>,
    world: Res<MpiWorld>,
) {
    let pos_data: HashMap<i32, Vec<PositionData>> = world
        .other_ranks()
        .map(|i| {
            (
                i,
                halo_cells
                    .iter()
                    .filter(|(_, cell, _)| cell.0 == i)
                    .map(|(entity, _, pos)| PositionData::new(pos, &entity))
                    .collect(),
            )
        })
        .collect();
    for (rank, data) in pos_data.iter() {
        world.world().process_at_rank(*rank).send(&data[..]);
    }
    for rank in world.other_ranks() {
        let (msg, status): (Vec<PositionData>, Status) =
            world.world().process_at_rank(rank).receive_vec();
        for pos in msg.iter() {
            let rank = status.source_rank();
            let target_pos = pos.pos();
            let local_cell = local_cells
                .iter()
                .find(|(_, pos)| pos == &&target_pos)
                .unwrap()
                .0;
            commands.entity(local_cell).insert(ExchangeCell {
                rank,
                entity: Entity::from_bits(pos.entity),
            });
        }
    }
}

pub(super) fn halo_exchange_system(
    mut halo_cells: Query<&mut Concentration, With<HaloCell>>,
    exchange_cells: Query<(&Concentration, &ExchangeCell)>,
    world: Res<MpiWorld>,
) {
    let data: HashMap<i32, Vec<ConcentrationData>> = world
        .other_ranks()
        .map(move |rank| {
            (
                rank,
                exchange_cells
                    .iter()
                    .filter(|(_, cell)| cell.rank == rank)
                    .map(|(concentration, cell)| ConcentrationData {
                        concentration: concentration.0.value,
                        entity: cell.entity.to_bits(),
                    })
                    .collect(),
            )
        })
        .collect();
    for rank in world.other_ranks() {
        world.world().process_at_rank(rank).send(&data[&rank]);
    }
    for rank in world.other_ranks() {
        let (msg, _): (Vec<ConcentrationData>, _) =
            world.world().process_at_rank(rank).receive_vec();
        for cell in msg {
            halo_cells
                .get_mut(Entity::from_bits(cell.entity))
                .unwrap()
                .0 = number_density_unit() * cell.concentration;
        }
    }
}

#[derive(Equivalence, Debug)]
pub struct PositionData {
    pos: (f64, f64),
    entity: u64,
}

impl PositionData {
    fn new(x: &Position, entity: &Entity) -> PositionData {
        PositionData {
            pos: (x.0.value, x.1.value),
            entity: entity.to_bits(),
        }
    }

    fn pos(&self) -> Position {
        Position(
            Length::new::<meter>(self.pos.0),
            Length::new::<meter>(self.pos.1),
        )
    }
}

#[derive(Equivalence)]
struct ConcentrationData {
    concentration: f64,
    entity: u64,
}
