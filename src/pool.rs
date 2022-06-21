use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;

pub trait FnBox {
    fn call_box(self: Box<Self>);
}

impl <F: FnOnce()> FnBox for F {
    fn call_box(self: Box<Self>) {
        (*self)();
    }
}

pub type Task = Box<dyn FnBox + Send>;

pub struct ThreadPool {
    tx: Option<Sender<Task>>,
    handlers: Option<Vec<thread::JoinHandle<()>>>
}

impl ThreadPool {
    pub fn new(number: usize) -> Self {
        let (tx, rx) = channel::<Task>();
        let mut handlers = vec![];

        let lock = Arc::new(Mutex::new(rx));

        for _ in 0..number {
            let lock = lock.clone();
            let handle = thread::spawn(move || {
                while let Ok(task) = lock.lock().unwrap().recv() {
                    println!("{:?}", lock );
                    task.call_box();
                }
            });

            handlers.push(handle);
        }

        ThreadPool {
            tx: Some(tx),
            handlers: Some(handlers)
        }
    }
}


#[cfg(test)]
mod test {
    use std::time::Duration;
    use super::*;

    #[test]
    fn print_test() {
        let p = ThreadPool::new(32);
        for i in 0..128 {
            p.tx.clone().unwrap().send(Box::new(move || {
                println!("task id {}", i);
                thread::sleep(Duration::from_secs(1));
            })).unwrap();
        }

        for h in p.handlers.unwrap() {
            h.join().unwrap();
        }
    }

    #[test]
    fn mutex_test() {
        let l = Arc::new(Mutex::new(0));
        let mut ta = vec![];
        for _ in 0..32 {
            let l = l.clone();
            let j = thread::spawn(move || {
                let mut data = l.lock().unwrap();
                *data += 1;
                println!("data is {}, sleep", data);
                drop(data);
                thread::sleep(Duration::from_secs(1));
                println!("done");
            });
            ta.push(j);
        }
        for j in ta {
            j.join().unwrap();
        }
    }
}
