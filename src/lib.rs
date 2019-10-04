#[cfg(test)]
pub mod test_helpers;

#[cfg(test)]
mod tests;

mod spmc;

use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, atomic::AtomicUsize, atomic::Ordering};

use spmc::{Sender, Receiver};

type Job = Box<dyn FnOnce() -> () + Send + 'static>;
type WorkerHandle = JoinHandle<()>;
type JobSender = Sender<Job>;

pub struct ThreadPool {
    worker_handles: Vec<WorkerHandle>,
    max_queue_depth: usize,
    job_sender: JobSender,
    executing_jobs: Arc<AtomicUsize>
}

type EnqueueResult = Result<(), EnqueueError>;

#[derive(Debug)]
pub struct EnqueueError {
    max_queue_depth: usize,
}

impl Error for EnqueueError {}

impl Display for EnqueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tried to submit more jobs than queue depth({})",
            self.max_queue_depth
        )
    }
}

impl ThreadPool {
    pub fn new(pool_size: usize, max_queue_depth: usize) -> ThreadPool {
        assert!(pool_size > 0);

        let mut worker_handles = Vec::new();
        let executing_jobs = Arc::new(AtomicUsize::new(0));

        worker_handles.reserve(pool_size);

        let (job_sender, job_receiver): (Sender<Job>, Receiver<Job>) = spmc::channel();

        for _ in 0..pool_size {
            let job_receiver = Receiver::clone(&job_receiver);
            let executing_jobs = Arc::clone(&executing_jobs);
            worker_handles.push(thread::spawn(move || {
                for job in job_receiver {
                    executing_jobs.fetch_add(1, Ordering::Relaxed);
                    job();
                    executing_jobs.fetch_sub(1, Ordering::Relaxed);
                }
            }));
        }

        ThreadPool {
            worker_handles,
            max_queue_depth: max_queue_depth * pool_size,
            job_sender,
            executing_jobs
        }
    }

    pub fn execute<F>(&self, job: F) -> EnqueueResult
        where
            F: FnOnce() -> (),
            F: Send + 'static
    {
        if self.queued_jobs() + self.executing_jobs() != self.max_queue_depth {
            self.job_sender.send(Box::new(job));
            Ok(())
        } else {
            Err(EnqueueError { max_queue_depth: self.max_queue_depth })
        }
    }

    pub fn executing_jobs(&self) -> usize {
        self.executing_jobs.load(Ordering::Relaxed)
    }

    pub fn queued_jobs(&self) -> usize {
        self.job_sender.queued()
    }

    fn join_all(self) {
        drop(self.job_sender);
        for worker_handle in self.worker_handles {
            worker_handle.join().unwrap();
        }
    }
}
