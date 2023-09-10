use std::{
    sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex},
    thread::{spawn, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<Sender<Job>>,
}

struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, rcv: Arc<Mutex<Receiver<Job>>>) -> Self {
        Worker {
            id,
            handle: Some(spawn(move || loop {
                let job = rcv.lock().unwrap().recv();
                if let Err(_)= job {
                    println!("Worker {id} disconnected and shutting down.");
                    break;
                }
                let job = job.unwrap();
                println!("Worker {id} is executing job...");
                job();
            })),
        }
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The 'new' function will panic if it's size is not higher than 0.
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (rx, tx) = channel();
        let tx = Arc::new(Mutex::new(tx));
        ThreadPool { 
            workers: (0..size).map(|id| Worker::new(id, Arc::clone(&tx))).collect(), 
            sender: Some(rx) }
    }

    /// Executes a job for the thread pool
    ///
    /// task is the closure to run in a thread.
    ///
    /// # Panics
    ///
    /// The 'execute' function panics if sending fails.
    pub fn execute<F>(&self, task: F) 
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.as_ref().unwrap().send(Box::new(task)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}
