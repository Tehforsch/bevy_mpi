use bevy::prelude::*;
use mpi::topology::Communicator;

struct MpiPlugin;

struct MpiWorld {
    size: i32,
    rank: i32,
}

impl Plugin for MpiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let universe = mpi::initialize().unwrap();
        let world = universe.world();
        let mpi_world = MpiWorld {
            size: world.size(),
            rank: world.rank(),
        };
        app.insert_resource(mpi_world);
    }
}
