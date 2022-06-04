use uom::si::f64::Length;
use uom::si::f64::Time;
use uom::si::length::meter;
use uom::si::Quantity;
use uom::si::ISQ;
use uom::si::SI;
use uom::typenum::*;

pub type NumberDensity = Quantity<ISQ<N3, Z0, Z0, Z0, Z0, Z0, Z0>, SI<f64>, f64>;
pub type NumberDensityPerTime = Quantity<ISQ<N3, Z0, N1, Z0, Z0, Z0, Z0>, SI<f64>, f64>;
// Because bevy shadows Time
pub type TimeQuantity = Time;

pub fn number_density_unit() -> NumberDensity {
    1.0 / (Length::new::<meter>(1.0) * Length::new::<meter>(1.0) * Length::new::<meter>(1.0))
}
