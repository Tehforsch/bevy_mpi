use mpi::topology::Communicator;

struct MpiWorld {
    size: i32,
    rank: i32,
}

pub fn initialize_mpi_and_add_world_resource(app: &mut bevy::prelude::App) -> i32 {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let mpi_world = MpiWorld {
        size: world.size(),
        rank: world.rank(),
    };
    app.insert_resource(mpi_world);
    mpi_world.rank
}
