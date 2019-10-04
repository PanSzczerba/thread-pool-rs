use std::sync::{Arc, Barrier};
use std::thread;
use std::time;

pub const JOIN_WAIT_TIME: time::Duration = time::Duration::from_millis(5);

pub fn barrier_wait(barrier: &Arc<Barrier>) {
    barrier.wait();
    thread::sleep(JOIN_WAIT_TIME);
}

pub fn barriers_wait(barriers: &[Arc<Barrier>]) {
    for barrier in barriers {
        barrier.wait();
    }
    thread::sleep(JOIN_WAIT_TIME);
}
