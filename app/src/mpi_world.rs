use mpi::environment::Universe;
use mpi::point_to_point::Status;
use mpi::topology::Communicator;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Destination;
use mpi::traits::Equivalence;
use mpi::traits::Source;
use mpi::Threading;

pub struct MpiWorld {
    universe: Universe,
}

impl MpiWorld {
    pub fn new() -> Self {
        let threading = Threading::Multiple;
        let universe = mpi::initialize().unwrap();
        Self { universe }
    }

    pub fn rank(&self) -> i32 {
        self.world().rank()
    }

    pub fn size(&self) -> i32 {
        self.world().size()
    }

    pub fn world(&self) -> SystemCommunicator {
        self.universe.world()
    }

    pub fn other_ranks(&self) -> impl Iterator<Item = i32> + '_ {
        (0..self.size()).filter(|rank| *rank != self.rank())
    }
}
