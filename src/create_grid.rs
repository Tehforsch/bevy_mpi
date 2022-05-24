use bevy::prelude::Commands;
use uom::si::f64::*;
use uom::si::length::meter;
use uom::si::time::second;

use crate::quantities::number_density_unit;
use crate::Black;
use crate::Concentration;
use crate::Neighbours;
use crate::Position;
use crate::Red;
use crate::Source;

pub fn create_grid_system(mut commands: Commands) {
    let nx = 20;
    let ny = 20;
    let wrap = |i: usize, j: usize| (i.rem_euclid(nx), j.rem_euclid(ny));
    let index = |i: usize, j: usize| i * ny + j;
    let mut entities = vec![];
    for i in 0..nx {
        for j in 0..ny {
            let concentration = if i <= 10 { 1.0 } else { 0.0 };
            let length = Length::new::<meter>(1.0);
            let pos = Position((i as f64) * length, (j as f64) * length);
            entities.push(
                commands
                    .spawn()
                    .insert(Concentration(concentration * number_density_unit()))
                    .insert(Source(
                        0.0 * number_density_unit() / Time::new::<second>(1.0),
                    ))
                    .insert(pos)
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
