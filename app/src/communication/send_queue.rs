use mpi::topology::Rank;

pub struct SendQueue<T> {
    items: Vec<T>,
    rank: Rank,
}

impl<T> SendQueue<T> {
    pub fn new(rank: Rank) -> Self {
        Self {
            items: vec![],
            rank,
        }
    }

    pub fn push(&mut self, data: T) {
        self.items.push(data)
    }

    pub fn send_all(&mut self) {
        todo!()
    }
}
