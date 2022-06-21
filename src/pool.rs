use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub type Task = Box<dyn FnOnce() + Send + 'static>;

pub enum Message {
    NewTask(Task),
    Terminate,
}

pub struct ThreadPool {
    sender: Sender<Message>,
    handlers: Vec<Option<thread::JoinHandle<()>>>,
}

impl ThreadPool {
    pub fn new(number: usize) -> Self {
        let (tx, rx) = channel::<Message>();
        let mut handlers = vec![];

        let lock = Arc::new(Mutex::new(rx));

        for _ in 0..number {
            let lock = lock.clone();
            let handle = thread::spawn(move || loop {
                let task = lock.lock().unwrap().recv().unwrap();
                match task {
                    Message::NewTask(task) => {
                        task();
                    }
                    Message::Terminate => {
                        break;
                    }
                }
            });

            handlers.push(Some(handle));
        }

        ThreadPool {
            sender: tx,
            handlers,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Box::new(f);
        self.sender.send(Message::NewTask(task)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.handlers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for j in &mut self.handlers {
            if let j = j.take().unwrap() {
                j.join().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn print_test() {
        let p = ThreadPool::new(32);
        for i in 0..128 {
            p.sender
                .clone()
                .send(Message::NewTask(Box::new(move || {
                    println!("task id {}", i);
                    thread::sleep(Duration::from_secs(1));
                })))
                .unwrap();
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
