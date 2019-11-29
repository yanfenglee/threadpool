use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

fn long_task(name: String, sec: u64) {
    println!("begin long task: {}", name);
    thread::sleep(Duration::from_secs(sec));
    println!("end long task: {}", name);
}

fn main() {
    let pool = ThreadPool::new(10);

    for i in 1..=4 {
        pool.execute(move ||{
            long_task(format!("task {}",i), i);
        });
    }
}
