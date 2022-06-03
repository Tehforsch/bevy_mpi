use mpi::environment::Universe;
use mpi::topology::Communicator;
use mpi::topology::SystemCommunicator;

pub struct MpiWorld {
    universe: Universe,
}

impl MpiWorld {
    pub fn new() -> Self {
        let universe = mpi::initialize().unwrap();
        Self { universe }
    }

    pub fn rank(&self) -> i32 {
        self.world().rank()
    }

    pub fn world(&self) -> SystemCommunicator {
        self.universe.world()
    }
}
