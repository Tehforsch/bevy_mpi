use std::collections::HashMap;

use bevy::prelude::ResMut;
use mpi::topology::Rank;

use super::send_queue::SendQueue;
use crate::mpi_world::MpiWorld;

pub struct SendQueues<T> {
    queues: HashMap<Rank, SendQueue<T>>,
}

impl<T> SendQueues<T> {
    pub fn from_mpi_world(world: &MpiWorld) -> Self {
        Self {
            queues: world
                .other_ranks()
                .into_iter()
                .map(|rank| (rank, SendQueue::<T>::new(rank)))
                .collect(),
        }
    }
    pub fn push(&mut self, rank: Rank, data: T) {
        self.queues.get_mut(&rank).unwrap().push(data)
    }

    pub fn send_all_system(mut queues: ResMut<SendQueues<T>>)
    where
        T: Sync + Send + 'static,
    {
        for queue in queues.queues.values_mut() {
            queue.send_all()
        }
    }
}
