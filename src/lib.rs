
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc,Mutex};
use std::ops::FnOnce;

pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

pub enum PoolMsg {
    NewJob(Job),
    Terminate,
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<PoolMsg>>>) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let msg = receiver.lock().unwrap().recv().unwrap();
                match msg {
                    PoolMsg::NewJob(job) => {
                        println!("thread {} recive job", id);
                        job.call_box();
                    },
                    PoolMsg::Terminate => {
                        println!("thread {} recive terminate", id);
                        break;
                    }
                }
                
            }
            
        });

        Worker{id, thread: Some(thread),}
    }
}



pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<PoolMsg>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for i in 0..size {
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }


        ThreadPool {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.send(PoolMsg::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.workers {
            self.sender.send(PoolMsg::Terminate).unwrap();
        }
        
        println!("shutdown all workers!!!");

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}