use mpi::environment::Universe;
use mpi::topology::Communicator;
use mpi::topology::Rank;
use mpi::topology::SystemCommunicator;
use mpi::traits::Destination;
use mpi::traits::Equivalence;

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

    pub fn size(&self) -> i32 {
        self.world().size()
    }

    pub fn world(&self) -> SystemCommunicator {
        self.universe.world()
    }

    pub fn send<T: Equivalence>(&self, rank: Rank, data: T) {
        self.world().process_at_rank(rank).send(&data)
    }

    pub fn other_ranks(&self) -> impl Iterator<Item = i32> + '_ {
        (0..self.size()).filter(|rank| *rank != self.rank())
    }
}
