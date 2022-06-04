use uom::si::f64::*;
use uom::si::length::meter;

use crate::Position;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GridData {
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
    pub fn new(
        size_x: i32,
        size_y: i32,
        num_ranks_x: i32,
        num_ranks_y: i32,
        this_rank_x: i32,
        this_rank_y: i32,
    ) -> Self {
        assert_eq!(size_x.rem_euclid(num_ranks_x), 0);
        assert_eq!(size_y.rem_euclid(num_ranks_y), 0);
        assert_eq!(size_x.rem_euclid(2), 0);
        assert_eq!(size_y.rem_euclid(2), 0);
        let local_size_x = size_x / num_ranks_x;
        let local_size_y = size_y / num_ranks_y;
        Self {
            size_x,
            size_y,
            num_ranks_x,
            num_ranks_y,
            this_rank_x,
            this_rank_y,
            local_size_x,
            local_size_y,
        }
    }

    pub fn iter_grid_cells(&self) -> impl Iterator<Item = CellIdentifier> + '_ {
        (0..self.size_x).flat_map(move |global_x| {
            (0..self.size_y).map(move |global_y| CellIdentifier {
                x: global_x,
                y: global_y,
                grid_data: self.clone(),
            })
        })
    }

    pub fn iter_local_grid_cells(&self) -> impl Iterator<Item = CellIdentifier> + '_ {
        self.iter_grid_cells().filter(|cell| cell.is_local())
    }

    pub fn iter_local_cells_and_haloes(&self) -> impl Iterator<Item = CellIdentifier> + '_ {
        self.iter_grid_cells()
            .filter(|cell| cell.is_local() || cell.is_halo())
    }

    fn wrap_x(&self, x: i32) -> i32 {
        x.rem_euclid(self.size_x)
    }

    fn wrap_y(&self, y: i32) -> i32 {
        y.rem_euclid(self.size_y)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct CellIdentifier {
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

    pub fn with_other_grid(&self, grid: GridData) -> Self {
        Self {
            x: self.x,
            y: self.y,
            grid_data: grid,
        }
    }

    fn wrap(&self) -> Self {
        self.new_from_global(self.grid_data.wrap_x(self.x), self.grid_data.wrap_y(self.y))
    }

    pub fn get_position(&self) -> Position {
        let length = Length::new::<meter>(1.0);
        Position((self.x as f64) * length, (self.y as f64) * length)
    }

    pub fn get_neighbours(&self) -> Vec<CellIdentifier> {
        vec![
            self.new_from_global(self.x - 1, self.y - 1).wrap(),
            self.new_from_global(self.x + 1, self.y - 1).wrap(),
            self.new_from_global(self.x - 1, self.y + 1).wrap(),
            self.new_from_global(self.x + 1, self.y + 1).wrap(),
        ]
    }

    pub fn is_even(&self) -> bool {
        (self.y * self.grid_data.size_x + self.x).rem_euclid(2) == 0
    }

    fn local_x(&self) -> i32 {
        self.x - self.grid_data.local_size_x * self.grid_data.this_rank_x
    }

    fn local_y(&self) -> i32 {
        self.y - self.grid_data.local_size_y * self.grid_data.this_rank_y
    }

    pub fn is_local(&self) -> bool {
        (0..self.grid_data.local_size_x).contains(&self.local_x())
            && (0..self.grid_data.local_size_y).contains(&self.local_y())
    }

    pub fn is_halo(&self) -> bool {
        // World's laziest calculation
        self.get_neighbours()
            .into_iter()
            .any(|neigh| neigh.is_local())
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
