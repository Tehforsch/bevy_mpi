use std::collections::HashMap;

use bevy::prelude::Commands;
use bevy::prelude::Res;
use uom::si::f64::*;
use uom::si::length::meter;
use uom::si::time::second;

use crate::mpi_world::MpiWorld;
use crate::quantities::number_density_unit;
use crate::Black;
use crate::Concentration;
use crate::Neighbours;
use crate::Position;
use crate::Red;
use crate::Source;

#[derive(Clone, PartialEq, Eq, Hash)]
struct GridData {
    grid_size_x: i32,
    grid_size_y: i32,
    num_ranks_x: i32,
    num_ranks_y: i32,
    this_rank_x: i32,
    this_rank_y: i32,
    local_grid_size_x: i32,
    local_grid_size_y: i32,
}

impl GridData {
    fn new(
        grid_size_x: i32,
        grid_size_y: i32,
        num_ranks_x: i32,
        num_ranks_y: i32,
        this_rank_x: i32,
        this_rank_y: i32,
    ) -> Self {
        assert_eq!(grid_size_x.rem_euclid(num_ranks_x), 0);
        assert_eq!(grid_size_y.rem_euclid(num_ranks_y), 0);
        let local_grid_size_x = grid_size_x / num_ranks_x;
        let local_grid_size_y = grid_size_y / num_ranks_y;
        Self {
            grid_size_x,
            grid_size_y,
            num_ranks_x,
            num_ranks_y,
            this_rank_x,
            this_rank_y,
            local_grid_size_x,
            local_grid_size_y,
        }
    }

    fn iter_grid_cells(&self) -> impl Iterator<Item = CellIdentifier> + '_ {
        (0..self.grid_size_x).flat_map(move |global_x| {
            (0..self.grid_size_y).map(move |global_y| CellIdentifier {
                global_x,
                global_y,
                grid_data: self.clone(),
            })
        })
    }

    fn iter_local_grid_cells(&self) -> impl Iterator<Item = CellIdentifier> + '_ {
        self.iter_grid_cells().filter(|cell| cell.is_local())
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct CellIdentifier {
    global_x: i32,
    global_y: i32,
    grid_data: GridData,
}

impl CellIdentifier {
    fn new_from_global(&self, global_x: i32, global_y: i32) -> Self {
        Self {
            grid_data: self.grid_data.clone(),
            global_x,
            global_y,
        }
    }

    fn wrap(&self) -> Self {
        self.new_from_global(
            self.global_x.rem_euclid(self.grid_data.grid_size_x),
            self.global_y.rem_euclid(self.grid_data.grid_size_y),
        )
    }

    fn get_position(&self) -> Position {
        let length = Length::new::<meter>(1.0);
        Position(
            (self.global_x as f64) * length,
            (self.global_y as f64) * length,
        )
    }

    fn get_neighbours(&self) -> Vec<CellIdentifier> {
        vec![
            self.new_from_global(self.global_x - 1, self.global_y - 1)
                .wrap(),
            self.new_from_global(self.global_x + 1, self.global_y - 1)
                .wrap(),
            self.new_from_global(self.global_x - 1, self.global_y + 1)
                .wrap(),
            self.new_from_global(self.global_x + 1, self.global_y + 1)
                .wrap(),
        ]
    }

    fn is_even(&self) -> bool {
        (self.global_y * self.grid_data.grid_size_x + self.global_x).rem_euclid(2) == 0
    }

    fn is_local(&self) -> bool {
        ((self.grid_data.local_grid_size_x * self.grid_data.this_rank_x)
            ..(self.grid_data.local_grid_size_x * (self.grid_data.this_rank_x + 1)))
            .contains(&self.global_x)
            && ((self.grid_data.local_grid_size_y * self.grid_data.this_rank_y)
                ..(self.grid_data.local_grid_size_y * (self.grid_data.this_rank_y + 1)))
                .contains(&self.global_y)
    }
}

pub fn create_grid_system(mut commands: Commands, world: Res<MpiWorld>) {
    let grid = GridData::new(60, 60, world.size(), 1, world.rank(), 0);
    let mut entities = HashMap::new();
    for cell in grid.iter_grid_cells() {
        let concentration = if cell.global_x <= 10 { 1.0 } else { 0.0 };
        entities.insert(
            cell.clone(),
            commands
                .spawn()
                .insert(Concentration(concentration * number_density_unit()))
                .insert(Source(
                    0.0 * number_density_unit() / Time::new::<second>(1.0),
                ))
                .insert(cell.get_position())
                .id(),
        );
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
