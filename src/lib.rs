use core::fmt;
use std::{error::Error, io, sync::{mpsc::{self}, Arc, Mutex}, thread::{self}};
type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
#[derive(Debug)]
pub enum PoolCreationError { NegativePoolSize, ThreadSpawnError }
impl Error for PoolCreationError {}
impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Negative thread pool is invalid!")
    }
}
impl From<io::Error> for PoolCreationError {
    fn from(_: io::Error) -> Self {
        PoolCreationError::ThreadSpawnError
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `build` function will panic if the size is zero.
    pub fn build(size: usize) ->  Result<ThreadPool, PoolCreationError> {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::build(id, Arc::clone(&receiver))?)
        }
        

        Ok(ThreadPool{workers, sender: Some(sender)})
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}
impl Worker {
    pub fn build(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Result<Worker, PoolCreationError> {
        let builder = thread::Builder::new();
        let thread = builder.spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        })?;
        Ok(Worker {id, thread: Some(thread)})
    }
}