use bevy::prelude::Component;
use uom::si::f64::Length;

#[derive(Component, Debug, PartialEq)]
pub struct Position(pub Length, pub Length);
