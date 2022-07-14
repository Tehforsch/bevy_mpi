use bevy::prelude::Component;
use uom::si::f64::Length;

#[derive(Component, Debug)]
pub struct Position(pub Length, pub Length);
