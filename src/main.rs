mod copy;
mod dir_tree;
mod pool;
mod test_gen;

use crate::copy::{copy_dir_recursive, copy_dir_recursive_single_thread};
use crate::pool::ThreadPool;
use crate::test_gen::TestDirGenerator;
use clap::{Parser, Subcommand};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};
use crate::dir_tree::{DirNode, SharedNodeRef};

#[derive(Subcommand, Debug)]
enum TestCommand {
    /// Generate test folder in path
    GenTestFolder {
        #[clap(value_parser)]
        path: String,

        #[clap(value_parser, short = 'd', long)]
        max_depth: Option<u32>,

        #[clap(value_parser, long)]
        threshold: Option<f64>,
    },
}

impl TestCommand {
    fn exec(&self) {
        match self {
            TestCommand::GenTestFolder {
                path,
                max_depth,
                threshold,
            } => {
                let path = Path::new(path);
                println!(
                    "path: {:?}, exists: {:?}, is_dir: {:?}",
                    path,
                    path.exists(),
                    path.is_dir()
                );

                if !path.exists() {
                    create_dir_all(path).unwrap();
                }

                let mut g = TestDirGenerator::builder();
                g = g.upper_path(path);

                if let Some(md) = max_depth {
                    g = g.max_depth(*md);
                }

                if let Some(t) = threshold {
                    g = g.threshold(*t);
                }

                g.build().unwrap().gen().unwrap()
            }
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(value_parser)]
    from: Option<String>,

    #[clap(value_parser)]
    to: Option<String>,

    #[clap(short, long, value_parser, default_value_t = 4)]
    thread: usize,

    #[clap(short, long, value_parser, default_value_t = false)]
    single_thread: bool,

    // #[clap(short, long, value_parser)]
    // name: String,
    #[clap(subcommand)]
    test_command: Option<TestCommand>,
}

fn multi_threads_copy(from: &str, to: &str, thread_number: usize) {
    let pool = ThreadPool::new(thread_number);
    let sender = pool.sender.clone();
    let now = Instant::now();
    let root = SharedNodeRef::new(DirNode::new(PathBuf::from(to)));
    copy_dir_recursive(
        PathBuf::from(from),
        PathBuf::from(to),
        PathBuf::new(),
        sender,
        root.clone()
    )
        .expect("Copy failed");

    println!("Waiting copy stop...");
    loop {
        println!("in loop");
        match root.0.try_read() {
            Ok(reader) => {
                if reader.is_copied() {
                    println!("Copy complete.");
                    break;
                }
            }
            Err(e) => {
                println!("{:?}", e);
            },
        }
        thread::sleep(Duration::from_millis(50));
    }
    let elapsed_time = now.elapsed();
    println!("Copy action took {} milliseconds", elapsed_time.as_millis());
}

fn single_thread_copy(from: &str, to: &str) {
    let now = Instant::now();
    copy_dir_recursive_single_thread(Path::new(from), Path::new(to), &PathBuf::new())
        .expect("Copy failed");
    let elapsed_time = now.elapsed();
    println!("Copy action took {} milliseconds", elapsed_time.as_millis());
}


fn run() {
    let args = Args::parse();
    if let Some(subcommand) = &args.test_command {
        subcommand.exec();
    }

    if let Some(from) = &args.from {
        println!("from: {}", from);
        match &args.to {
            Some(to) => {
                println!("to: {}", to);
                if args.single_thread {
                    single_thread_copy(from, to);
                } else {
                    let threads_number = args.thread;
                    multi_threads_copy(from, to, threads_number);
                }
            }
            None => {
                println!("Not set target");
            }
        }
    }
}

fn main() {
    run();
}
