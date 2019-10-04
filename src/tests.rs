use super::test_helpers::*;
use super::*;

use std::sync::{Arc, Barrier};
use std::time::Duration;

fn add_idle_jobs(pool: &ThreadPool, idle_jobs_count: usize) -> Vec<Arc<Barrier>> {
    let mut barriers = Vec::new();

    for _ in 0..idle_jobs_count {
        let barrier = Arc::new(Barrier::new(2));
        let thread_barrier = Arc::clone(&barrier);

        assert!(pool
            .execute(move || {
                thread_barrier.wait();
            })
            .is_ok());

        barriers.push(barrier);
    }
    thread::sleep(Duration::from_millis(1));

    barriers
}

#[test]
fn one_job_pool() {
    let pool = ThreadPool::new(4, 1);
    let barrier = add_idle_jobs(&pool, 1).remove(0);

    assert_eq!(pool.executing_jobs(), 1);
    assert_eq!(pool.queued_jobs(), 0);

    barrier_wait(&barrier);
}

#[test]
fn multiple_jobs_in_pool() {
    let pool = ThreadPool::new(4, 1);
    let barriers = add_idle_jobs(&pool, 2);

    assert_eq!(pool.executing_jobs(), 2);
    assert_eq!(pool.queued_jobs(), 0);

    barriers_wait(&barriers);
}

#[test]
fn full_queue() {
    let pool = ThreadPool::new(4, 2);
    let barriers = add_idle_jobs(&pool, 4);

    assert!(pool.execute(|| ()).is_ok());
    assert_eq!(pool.executing_jobs(), 4);
    assert_eq!(pool.queued_jobs(), 1);

    barriers_wait(&barriers);
}

#[test]
fn queued_job_starts_after_one_job_end() {
    let pool = ThreadPool::new(4, 2);
    let mut barriers = add_idle_jobs(&pool, 3);
    assert_eq!(pool.executing_jobs(), 3);
    assert_eq!(pool.queued_jobs(), 0);

    barriers.append(&mut add_idle_jobs(&pool, 1));
    assert_eq!(pool.executing_jobs(), 4);
    assert_eq!(pool.queued_jobs(), 0);

    barriers.append(&mut add_idle_jobs(&pool, 1));
    assert_eq!(pool.executing_jobs(), 4);
    assert_eq!(pool.queued_jobs(), 1);

    barrier_wait(&barriers.remove(0));
    assert_eq!(pool.executing_jobs(), 4);
    assert_eq!(pool.queued_jobs(), 0);

    barriers_wait(&barriers);
}

#[test]
fn queue_job_over_queue_limit() {
    let pool = ThreadPool::new(4, 1);
    let barriers = add_idle_jobs(&pool, 4);

    assert!(pool.execute(|| ()).is_err());

    barriers_wait(&barriers);
}
