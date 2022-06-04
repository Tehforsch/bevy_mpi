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
use crate::HaloCell;
use crate::LocalCell;
use crate::Neighbours;
use crate::Position;
use crate::Red;
use crate::Source;

#[derive(Clone, PartialEq, Eq, Hash)]
struct GridData {
    size_x: i32,
    size_y: i32,
    num_ranks_x: i32,
    num_ranks_y: i32,
    this_rank_x: i32,
    this_rank_y: i32,
    local_size_x: i32,
    local_size_y: i32,
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
        assert_eq!(grid_size_x.rem_euclid(2), 0);
        assert_eq!(grid_size_y.rem_euclid(2), 0);
        let local_grid_size_x = grid_size_x / num_ranks_x;
        let local_grid_size_y = grid_size_y / num_ranks_y;
        Self {
            size_x: grid_size_x,
            size_y: grid_size_y,
            num_ranks_x,
            num_ranks_y,
            this_rank_x,
            this_rank_y,
            local_size_x: local_grid_size_x,
            local_size_y: local_grid_size_y,
        }
    }

    fn iter_grid_cells(&self) -> impl Iterator<Item = CellIdentifier> + '_ {
        (0..self.size_x).flat_map(move |global_x| {
            (0..self.size_y).map(move |global_y| CellIdentifier {
                x: global_x,
                y: global_y,
                grid_data: self.clone(),
            })
        })
    }

    fn iter_local_grid_cells(&self) -> impl Iterator<Item = CellIdentifier> + '_ {
        self.iter_grid_cells().filter(|cell| cell.is_local())
    }

    fn iter_local_cells_and_haloes(&self) -> impl Iterator<Item = CellIdentifier> + '_ {
        self.iter_grid_cells()
            .filter(|cell| cell.is_local() || cell.is_halo())
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct CellIdentifier {
    x: i32,
    y: i32,
    grid_data: GridData,
}

impl CellIdentifier {
    fn new_from_global(&self, global_x: i32, global_y: i32) -> Self {
        Self {
            grid_data: self.grid_data.clone(),
            x: global_x,
            y: global_y,
        }
    }

    fn wrap(&self) -> Self {
        self.new_from_global(
            self.x.rem_euclid(self.grid_data.size_x),
            self.y.rem_euclid(self.grid_data.size_y),
        )
    }

    fn get_position(&self) -> Position {
        let length = Length::new::<meter>(1.0);
        Position(
            (self.x as f64) * length,
            (self.y as f64) * length,
        )
    }

    fn get_neighbours(&self) -> Vec<CellIdentifier> {
        vec![
            self.new_from_global(self.x - 1, self.y - 1)
                .wrap(),
            self.new_from_global(self.x + 1, self.y - 1)
                .wrap(),
            self.new_from_global(self.x - 1, self.y + 1)
                .wrap(),
            self.new_from_global(self.x + 1, self.y + 1)
                .wrap(),
        ]
    }

    fn is_even(&self) -> bool {
        (self.y * self.grid_data.size_x + self.x).rem_euclid(2) == 0
    }

    fn is_local(&self) -> bool {
        let local_x = self.x - self.grid_data.local_size_x * self.grid_data.this_rank_x;
        let local_y = self.y - self.grid_data.local_size_y * self.grid_data.this_rank_y;
        (0..self.grid_data.local_size_x).contains(&local_x)
            && (0..self.grid_data.local_size_y).contains(&local_y)
    }

    fn is_halo(&self) -> bool {
        let local_x = (self.x
            - self.grid_data.local_size_x * self.grid_data.this_rank_x)
            .rem_euclid(self.grid_data.size_x);
        let local_y = (self.y
            - self.grid_data.local_size_y * self.grid_data.this_rank_y)
            .rem_euclid(self.grid_data.size_y);
        let is_on_x_border = (local_x + 1).rem_euclid(self.grid_data.size_x) == 0
            || local_x == self.grid_data.local_size_x;
        let is_on_y_border = (local_y + 1).rem_euclid(self.grid_data.size_y) == 0
            || local_y == self.grid_data.local_size_y;
        is_on_x_border ^ is_on_y_border
    }
}

pub fn create_grid_system(mut commands: Commands, world: Res<MpiWorld>) {
    let grid = GridData::new(30, 30, world.size(), 1, world.rank(), 0);
    let mut entities = HashMap::new();
    for cell in grid.iter_local_cells_and_haloes() {
        let concentration = if cell.x <= 10 { 1.0 } else { 0.0 };
        let mut entity_commands = commands.spawn();
        entity_commands
            .insert(Concentration(concentration * number_density_unit()))
            .insert(Source(
                0.0 * number_density_unit() / Time::new::<second>(1.0),
            ))
            .insert(cell.get_position());
        if cell.is_local() {
            entity_commands.insert(LocalCell);
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

#[cfg(test)]
mod tests {
    use super::CellIdentifier;
    use super::GridData;

    fn assert_is_halo(grid_data: &GridData, x: i32, y: i32) {
        assert!(CellIdentifier {
            x,
            y,
            grid_data: grid_data.clone(),
        }
        .is_halo())
    }

    #[test]
    fn halo_check_x() {
        let size_x = 20;
        let size_y = 40;
        let grid = GridData::new(size_x, size_y, 2, 1, 0, 0);
        for y in 1..size_y - 1 {
            assert_is_halo(&grid, size_x / 2, y);
            assert_is_halo(&grid, size_x - 1, y);
        }
        let grid = GridData::new(size_x, size_y, 2, 1, 1, 0);
        for y in 1..size_y - 1 {
            assert_is_halo(&grid, 0, y);
            assert_is_halo(&grid, 9, y);
        }
    }

    #[test]
    fn halo_check_y() {
        let size_x = 20;
        let size_y = 40;
        let grid = GridData::new(size_x, size_y, 1, 4, 0, 1);
        for x in 1..size_x - 1 {
            assert_is_halo(&grid, x, 9);
            assert_is_halo(&grid, x, 20);
        }
    }
}
