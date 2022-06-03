killall bevy_mpi
cargo build
mpirun -n 3 ~/.cargo-target/debug/bevy_mpi
killall bevy_mpi
